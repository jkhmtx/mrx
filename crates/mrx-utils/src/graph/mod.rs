use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    path::{
        Path,
        PathBuf,
    },
};

mod error;
use error::GraphError;

use crate::{
    Config,
    ast::{
        NixAst,
        NixAstNodes,
        NixAstNodesError,
    },
    attr::Attrname,
    find_nix_path_attrset,
    fs::{
        AbsolutePathBuf,
        AbsolutePathBufError,
    },
};

#[derive(Clone, Debug)]
pub struct GraphNode {
    pub path: AbsolutePathBuf,
    pub derivation: Option<Attrname>,
}

impl GraphNode {
    #[must_use]
    pub fn as_path(&self) -> &AbsolutePathBuf {
        &self.path
    }
}

impl From<AbsolutePathBuf> for GraphNode {
    fn from(path: AbsolutePathBuf) -> Self {
        GraphNode {
            path,
            derivation: None,
        }
    }
}

fn get_idx_or_create_node(
    lookup: &HashMap<NodeId, usize>,
    parent: PathBuf,
    node: &NixAst,
) -> Result<Option<GraphNodeOrIdx>, GraphError> {
    match node {
        NixAst::ImportOwnNameModuleExpression => Ok(None),
        NixAst::SimplePath { path } => {
            let path = PathBuf::from(path);

            let path = AbsolutePathBuf::try_from_relative(&path, &parent)?;

            let id = NodeId::Path(path.clone());
            if let Some(idx) = lookup.get(&id) {
                Ok(Some(GraphNodeOrIdx::Idx(*idx)))
            } else {
                Ok(Some(GraphNodeOrIdx::GraphNode(GraphNode::from(path))))
            }
        }
        NixAst::NixDirectoryPath { path } => {
            if let Some(stripped) = path.strip_suffix(".") {
                let relative = AbsolutePathBuf::try_from_relative(Path::new(&stripped), &parent)?;

                if relative.join("default.nix").is_file() {
                    get_idx_or_create_node(
                        lookup,
                        parent,
                        &NixAst::SimplePath {
                            path: stripped.to_string() + "default.nix",
                        },
                    )
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        NixAst::MrxDerivation { name } => {
            let attrname = Attrname::try_from(name.as_str())
                .map_err(|_| GraphError::InvalidNode(name.clone()))?;

            if attrname.is_internal() {
                Ok(None)
            } else if let Some(idx) = {
                let id = NodeId::Attrname(attrname.clone());
                // The setup we do in 'new' that finds all known attrnames ensures there is 'Some'
                // for valid nodes, and 'None' for out-of-date/invalid nodes (e.g. a node was
                // deleted from the dependency graph but another node still erroneously depends on it)
                lookup.get(&id)
            } {
                Ok(Some(GraphNodeOrIdx::Idx(*idx)))
            } else {
                Ok(None)
            }
        }
    }
}

fn set_dependencies<'deps, 'graph>(
    dependencies: &'deps mut HashMap<usize, &'graph GraphNode>,
    visited: &mut HashSet<usize>,
    graph: &'graph Graph,
    idx: usize,
) -> Option<Vec<usize>>
where
    'graph: 'deps,
{
    if visited.contains(&idx) {
        None
    } else {
        let mut next = vec![];
        for (home_idx, depends_on_idx) in &graph.edges {
            if home_idx == &idx {
                let node = &graph.nodes[*depends_on_idx];
                dependencies.insert(*depends_on_idx, node);
                next.push(*depends_on_idx);
            }
        }
        visited.insert(idx);

        Some(next)
    }
}

fn set_dependencies_r<'graph>(
    parents: &mut HashMap<usize, &'graph GraphNode>,
    visited: &mut HashSet<usize>,
    graph: &'graph Graph,
    idx: usize,
) {
    if let Some(next) = set_dependencies(parents, visited, graph, idx) {
        for idx in &next {
            set_dependencies_r(parents, visited, graph, *idx);
        }
    }
}

#[derive(Debug)]
pub struct Edge(pub GraphNode, pub GraphNode);

enum GraphNodeOrIdx {
    GraphNode(GraphNode),
    Idx(usize),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NodeId {
    Attrname(Attrname),
    Path(AbsolutePathBuf),
}

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<GraphNode>,
    edges: Vec<(usize, usize)>,
}

impl Graph {
    /// # Errors
    /// TODO
    pub fn new(config: &Config) -> Result<Self, GraphError> {
        let entrypoint = config.get_entrypoint().ok_or(GraphError::NoEntrypoint)?;
        let path = AbsolutePathBuf::try_from(entrypoint.as_ref()).map_err(|e| match e {
            AbsolutePathBufError::NotFound(_) => GraphError::NoEntrypoint,
            AbsolutePathBufError::NotSupported(path) => {
                GraphError::InvalidNode(path.to_string_lossy().to_string())
            }
            AbsolutePathBufError::Io(_, e) => GraphError::Io(e),
        })?;

        let mut graph = Self {
            edges: Vec::default(),
            nodes: Vec::default(),
        };

        let mut lookup = HashMap::default();

        graph.add_node(&mut lookup, GraphNode::from(path.clone()));

        let known_attrs = find_nix_path_attrset(config);

        let known_nodes = known_attrs.iter().map(|(attrname, p)| {
            AbsolutePathBuf::try_from(p).map(|path| GraphNode {
                derivation: Some(attrname.clone()),
                path,
            })
        });

        for node in known_nodes {
            match node {
                Ok(node) => {
                    graph.add_node(&mut lookup, node);
                    Ok(())
                }
                Err(AbsolutePathBufError::NotFound(_)) => Ok(()),
                Err(e) => Err(e),
            }?;
        }

        let mut visited = HashSet::default();

        for idx in 0..graph.nodes.len() {
            graph.process(&mut lookup, &mut visited, idx)?;
        }

        Ok(graph)
    }

    #[must_use]
    pub fn to_nodes(&self) -> Vec<&AbsolutePathBuf> {
        self.nodes.iter().map(|node| &node.path).collect()
    }

    #[must_use]
    pub fn to_edges(&self) -> Vec<Edge> {
        self.edges
            .iter()
            .map(|(a, b)| Edge(self.nodes[*a].clone(), self.nodes[*b].clone()))
            .collect()
    }

    fn add_node(&mut self, lookup: &mut HashMap<NodeId, usize>, node: GraphNode) -> usize {
        let current = self.nodes.len();

        if let Some(derivation) = &node.derivation {
            let attrname = NodeId::Attrname(derivation.clone());
            lookup.entry(attrname).or_insert(current);
        }

        let path = NodeId::Path(node.path.clone());

        if let Some(existing_idx) = lookup.get(&path) {
            *existing_idx
        } else {
            lookup.insert(path, current);
            self.nodes.push(node);

            current
        }
    }

    fn add_edge(&mut self, home_idx: usize, dependency_idx: usize) {
        self.edges.push((home_idx, dependency_idx));
    }

    fn process(
        &mut self,
        lookup: &mut HashMap<NodeId, usize>,
        visited: &mut HashSet<usize>,
        idx: usize,
    ) -> Result<(), GraphError> {
        let parent = {
            let node = &self.nodes[idx];
            &node.path.clone()
        };

        visited.insert(idx);

        if let Some(nodes) = match NixAstNodes::new(parent) {
            Ok(ast) => Ok(Some(ast)),
            Err(NixAstNodesError::NotNix(_)) => Ok(None),
            Err(e) => Err(e),
        }? {
            for ast_node in nodes.iter() {
                match get_idx_or_create_node(lookup, parent.to_path_buf(), ast_node)? {
                    Some(GraphNodeOrIdx::Idx(existing_idx)) => {
                        self.add_edge(idx, existing_idx);
                    }
                    Some(GraphNodeOrIdx::GraphNode(node)) => {
                        let curr_idx = self.nodes.len();

                        self.add_edge(idx, curr_idx);
                        self.add_node(lookup, node.clone());

                        self.process(lookup, visited, curr_idx)?;
                    }
                    None => {}
                }
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn find_node(&self, id: &NodeId) -> Option<(usize, &GraphNode)> {
        self.nodes.iter().enumerate().find(|pair| {
            let node = pair.1;

            match &id {
                NodeId::Attrname(attrname) => node
                    .derivation
                    .as_ref()
                    .is_some_and(|name| attrname == name),
                NodeId::Path(path) => node.path == *path,
            }
        })
    }

    #[must_use]
    pub fn find_dependencies_of(&self, idx: usize) -> HashMap<usize, &GraphNode> {
        let mut dependencies = HashMap::new();

        set_dependencies_r(&mut dependencies, &mut HashSet::default(), self, idx);

        dependencies
    }
}
