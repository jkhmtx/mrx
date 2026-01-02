use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    io::ErrorKind,
    path::{
        Path,
        PathBuf,
    },
};

mod error;
use error::GraphError;

use crate::{
    Config,
    attr::Attrname,
    find_nix_path_attrset,
    fs::{
        AbsoluteFilePathBuf,
        AbsoluteFilePathBufError,
    },
};

#[derive(Clone, Debug)]
pub struct Node {
    path: AbsoluteFilePathBuf,
    derivation: Option<Attrname>,
}

impl Node {
    #[must_use]
    pub fn as_path(&self) -> &AbsoluteFilePathBuf {
        &self.path
    }
}

impl From<AbsoluteFilePathBuf> for Node {
    fn from(path: AbsoluteFilePathBuf) -> Self {
        Node {
            path,
            derivation: None,
        }
    }
}

enum NodeReferenceKind {
    SimplePath,
    NixDirectoryPath,
    Derivation,
}

struct NodeReference {
    text: String,
    kind: NodeReferenceKind,
}

impl TryFrom<&rnix::SyntaxNode> for NodeReference {
    type Error = ();

    fn try_from(value: &rnix::SyntaxNode) -> Result<Self, Self::Error> {
        use rnix::SyntaxKind as Kind;

        match value.kind() {
            Kind::NODE_PATH => {
                let text = value.text();

                // If the last component in a path is the character '.',
                // it means it refers to a directory.
                // e.g. '.', '../.'
                let is_nix_directory_path = text
                    .len()
                    .checked_sub(1.into())
                    .and_then(|idx| text.char_at(idx))
                    .is_some_and(|c| c == '.');

                if is_nix_directory_path {
                    Some(NodeReference {
                        text: text.to_string(),
                        kind: NodeReferenceKind::NixDirectoryPath,
                    })
                } else {
                    Some(NodeReference {
                        text: text.to_string(),
                        kind: NodeReferenceKind::SimplePath,
                    })
                }
            }
            Kind::NODE_SELECT if value.first_child().is_some_and(|child| child.text() == "_") => {
                let text = value.text().to_string();

                Some(NodeReference {
                    text,
                    kind: NodeReferenceKind::Derivation,
                })
            }
            _ => None,
        }
        .ok_or(())
    }
}

fn walk(node: &rnix::SyntaxNode, references: &mut Vec<NodeReference>) {
    if let Ok(reference) = NodeReference::try_from(node) {
        references.push(reference);
    }

    for child in node.children() {
        walk(&child, references);
    }
}

fn references_within(path: &AbsoluteFilePathBuf) -> Result<Vec<NodeReference>, GraphError> {
    if !path.is_nix() {
        return Ok(vec![]);
    }

    let file = std::fs::read(path.as_path())
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => GraphError::MissingNode(path.to_string_lossy().to_string()),
            _ => GraphError::Io(e),
        })
        .and_then(|buf| {
            String::from_utf8(buf)
                .map_err(|_| GraphError::InvalidNode(path.to_string_lossy().to_string()))
        })?;

    let root = rnix::Root::parse(&file).syntax();
    let mut references = vec![];
    walk(&root, &mut references);

    Ok(references)
}

fn get_idx_or_create_node(
    lookup: &HashMap<NodeId, usize>,
    parent: PathBuf,
    reference: &NodeReference,
) -> Result<Option<NodeOrIdx>, GraphError> {
    match reference.kind {
        NodeReferenceKind::SimplePath => {
            let path = PathBuf::from(&reference.text);

            let path = AbsoluteFilePathBuf::try_from_relative(&path, &parent)?;

            let id = NodeId::Path(path.clone());
            if let Some(idx) = lookup.get(&id) {
                Ok(Some(NodeOrIdx::Idx(*idx)))
            } else {
                Ok(Some(NodeOrIdx::Node(Node::from(path))))
            }
        }
        NodeReferenceKind::NixDirectoryPath => {
            if let Some(stripped) = reference.text.strip_suffix(".") {
                let relative =
                    AbsoluteFilePathBuf::try_from_relative(Path::new(&stripped), &parent)?;

                if relative.join("default.nix").is_file() {
                    get_idx_or_create_node(
                        lookup,
                        parent,
                        &NodeReference {
                            // TODO: Test case where stripped = ""
                            text: stripped.to_string() + "default.nix",
                            kind: NodeReferenceKind::SimplePath,
                        },
                    )
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        NodeReferenceKind::Derivation => {
            let attrname = Attrname::try_from(reference.text.as_str())
                .map_err(|_| GraphError::InvalidNode(reference.text.clone()))?;

            if attrname.is_internal() {
                Ok(None)
            } else if let Some(idx) = {
                let id = NodeId::Attrname(attrname.clone());
                // The setup we do in 'new' that finds all known attrnames ensures there is 'Some'
                // for valid nodes, and 'None' for out-of-date/invalid nodes (e.g. a node was
                // deleted from the dependency graph but another node still erroneously depends on it)
                lookup.get(&id)
            } {
                Ok(Some(NodeOrIdx::Idx(*idx)))
            } else {
                Ok(None)
            }
        }
    }
}

#[derive(Debug)]
pub struct Edge(pub Node, pub Node);

enum NodeOrIdx {
    Node(Node),
    Idx(usize),
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum NodeId {
    Attrname(Attrname),
    Path(AbsoluteFilePathBuf),
}

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<(usize, usize)>,
}

impl Graph {
    /// # Errors
    /// TODO
    pub fn new(config: &Config) -> Result<Self, GraphError> {
        let entrypoint = config.get_entrypoint().ok_or(GraphError::NoEntrypoint)?;
        let path =
            AbsoluteFilePathBuf::try_from(entrypoint.as_path().as_path()).map_err(|e| match e {
                AbsoluteFilePathBufError::NotFound(_) => GraphError::NoEntrypoint,
                AbsoluteFilePathBufError::NotAFile(path) => {
                    GraphError::InvalidNode(path.to_string_lossy().to_string())
                }
                AbsoluteFilePathBufError::Io(_, e) => GraphError::Io(e),
            })?;

        let mut graph = Self {
            edges: Vec::default(),
            nodes: Vec::default(),
        };

        let mut lookup = HashMap::default();

        graph.add_node(&mut lookup, Node::from(path.clone()));

        let known_attrs = find_nix_path_attrset(config);

        let known_nodes = known_attrs.iter().map(|(attrname, p)| {
            AbsoluteFilePathBuf::try_from(p).map(|path| Node {
                derivation: Some(attrname.clone()),
                path,
            })
        });

        for node in known_nodes {
            graph.add_node(&mut lookup, node?);
        }

        let mut visited = HashSet::default();

        for idx in 0..graph.nodes.len() {
            graph.process(&mut lookup, &mut visited, idx)?;
        }

        Ok(graph)
    }

    #[must_use]
    pub fn to_nodes(&self) -> Vec<&AbsoluteFilePathBuf> {
        self.nodes.iter().map(|node| &node.path).collect()
    }

    #[must_use]
    pub fn to_edges(&self) -> Vec<Edge> {
        self.edges
            .iter()
            .map(|(a, b)| Edge(self.nodes[*a].clone(), self.nodes[*b].clone()))
            .collect()
    }

    fn add_node(&mut self, lookup: &mut HashMap<NodeId, usize>, node: Node) -> usize {
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

    fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((from, to));
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

        let references = references_within(parent)?;
        visited.insert(idx);

        for reference in &references {
            match get_idx_or_create_node(lookup, parent.to_path_buf(), reference)? {
                Some(NodeOrIdx::Idx(existing_idx)) => {
                    self.add_edge(idx, existing_idx);
                }
                Some(NodeOrIdx::Node(node)) => {
                    let curr_idx = self.nodes.len();

                    self.add_edge(idx, curr_idx);
                    self.add_node(lookup, node.clone());

                    self.process(lookup, visited, curr_idx)?;
                }
                None => {}
            }
        }

        Ok(())
    }
}
