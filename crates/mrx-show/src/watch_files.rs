use mrx_utils::{
    Attrname,
    Config,
    fs::AbsoluteFilePathBuf,
    graph::{
        Graph,
        NodeId,
    },
};

use crate::cli::WatchFilesOptions;

pub(crate) fn watch_files(config: &Config, options: &WatchFilesOptions) -> Vec<String> {
    let graph = Graph::new(config).unwrap();
    let generated_out_path =
        AbsoluteFilePathBuf::try_from(config.get_generated_out_path().as_path()).unwrap();

    {
        let mut files = if options.derivations.is_empty() {
            graph.to_nodes()
        } else {
            let mut files: Vec<&AbsoluteFilePathBuf> = vec![];

            for (idx, node) in options.derivations.iter().filter_map(|derivation| {
                Attrname::try_from(derivation.as_str())
                    .map(NodeId::Attrname)
                    .ok()
                    .and_then(|id| graph.find_node(&id))
            }) {
                files.push(&node.path);

                files.extend(
                    graph
                        .find_dependencies_of(idx)
                        .values()
                        .map(|node| &node.path),
                );
            }

            files
        }
        .into_iter()
        .filter(|path| **path != generated_out_path)
        .map(|path| path.as_relative_to_parent(&config.dir()).unwrap())
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();

        files.dedup();
        files.sort();

        files
    }
}
