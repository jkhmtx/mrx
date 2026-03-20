use std::{
    collections::{
        HashMap,
        HashSet,
    },
    ffi::OsStr,
    fmt::Debug,
    path::{
        Path,
        PathBuf,
    },
};

use exn::{
    OptionExt,
    ResultExt,
};

use crate::{
    Config,
    NixAstNodesError,
    ast::{
        NixAst,
        NixAstNodes,
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

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum GraphError {
    #[error(
        "GraphError::GettingEntrypoint: custom entrypoint, 'flake.nix' or 'default.nix' not found"
    )]
    GettingEntrypoint,
    #[error("GraphError::GettingPathAttrset")]
    GettingPathAttrset,
    #[error("GraphError::InvalidNode")]
    InvalidNode,
    #[error(
        "GraphError::RelativePath: relative path from parent '{parent}' does not exist\n\npath: {path}\nparent: {parent}"
    )]
    RelativePath {
        path: String,
        parent: AbsolutePathBuf,
    },
}

type GraphResult<T> = Result<T, exn::Exn<GraphError>>;

impl std::fmt::Display for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(derivation) = &self.derivation {
            f.write_str(derivation)
        } else {
            let path = &self.path.display().to_string();
            f.write_str(path)
        }
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

fn from_relative(path: &Path, relative_to: &AbsolutePathBuf) -> Option<PathBuf> {
    let mut parent = relative_to
        .parent()
        .expect("This should only fail when 'relative_to' is the filesystem root '/'");
    let (up_traversing, components): (Vec<_>, Vec<_>) = path
        .components()
        .partition(|s| s.as_os_str() == ".." || s.as_os_str() == ".");
    for _ in up_traversing.iter().filter(|s| s.as_os_str() != ".") {
        parent = parent.parent()?;
    }

    let mut path = PathBuf::new();
    path.extend(components);

    Some(parent.join(path))
}

fn try_from_relative<T: ?Sized + AsRef<OsStr>>(
    path: &T,
    parent: &AbsolutePathBuf,
) -> GraphResult<AbsolutePathBuf> {
    let path = PathBuf::from(path);
    if let Some(path) = from_relative(&path, parent) {
        AbsolutePathBuf::try_from(path.as_path()).or_raise(|| GraphError::RelativePath {
            path: path.display().to_string(),
            parent: parent.clone(),
        })
    } else {
        Err(exn::Exn::from(GraphError::RelativePath {
            path: path.display().to_string(),
            parent: parent.clone(),
        }))
    }
}

fn get_idx_or_create_node(
    lookup: &HashMap<NodeId, usize>,
    parent: &AbsolutePathBuf,
    node: &NixAst,
) -> GraphResult<Option<GraphNodeOrIdx>> {
    match node {
        NixAst::ImportOwnNameModuleExpression => Ok(None),
        NixAst::SimplePath { path } => {
            let path = try_from_relative(&path, parent)?;

            let id = NodeId::Path(path.clone());

            Ok(Some(match lookup.get(&id) {
                Some(idx) => GraphNodeOrIdx::Idx(*idx),
                None => GraphNodeOrIdx::GraphNode(GraphNode::from(path)),
            }))
        }
        NixAst::NixDirectoryPath { path } => {
            if let Some(stripped) = path.strip_suffix(".") {
                let relative = try_from_relative(&stripped, parent)?;
                let default_nix = relative.join("default.nix");

                if default_nix.is_file() {
                    let default_nix = AbsolutePathBuf::File(default_nix);
                    let id = NodeId::Path(default_nix.clone());

                    return Ok(Some(match lookup.get(&id) {
                        Some(idx) => GraphNodeOrIdx::Idx(*idx),
                        None => GraphNodeOrIdx::GraphNode(GraphNode::from(default_nix)),
                    }));
                }
            }

            Ok(None)
        }
        NixAst::MrxDerivation { name } => {
            let attrname =
                Attrname::try_from(name.as_str()).or_raise(|| GraphError::InvalidNode)?;

            if attrname.is_internal() {
                return Ok(None);
            }

            if let Some(idx) = lookup.get(&NodeId::Attrname(attrname)) {
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
    pub fn new(config: &Config) -> GraphResult<Self> {
        let entrypoint = config
            .get_entrypoint()
            .ok_or_raise(|| GraphError::GettingEntrypoint)?;
        let path = AbsolutePathBuf::try_from(entrypoint.as_ref())
            .or_raise(|| GraphError::GettingEntrypoint)?;

        let mut graph = Self {
            edges: Vec::default(),
            nodes: Vec::default(),
        };

        let mut lookup = HashMap::default();

        graph.add_node(&mut lookup, GraphNode::from(path.clone()));

        let known_attrs =
            find_nix_path_attrset(config).or_raise(|| GraphError::GettingPathAttrset)?;

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
                Err(AbsolutePathBufError::NotFound) => Ok(()),
                Err(AbsolutePathBufError::Io(e)) => Err(e),
            }
            .or_raise(|| GraphError::InvalidNode)?;
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
    ) -> GraphResult<()> {
        let parent = {
            let node = &self.nodes[idx];
            &node.path.clone()
        };

        visited.insert(idx);

        if let Some(nodes) = match NixAstNodes::new(parent) {
            Ok(ast) => Ok(Some(ast)),
            Err(e) if matches!(*e, NixAstNodesError::NotNix(_)) => Ok(None),
            Err(e) => Err(e),
        }
        .or_raise(|| GraphError::InvalidNode)?
        {
            for ast_node in nodes.iter() {
                match get_idx_or_create_node(lookup, parent, ast_node)? {
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
