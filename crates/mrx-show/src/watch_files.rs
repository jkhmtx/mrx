use mrx_utils::{fs::AbsoluteFilePathBuf, graph::Graph, Config};

pub fn watch_files(config: Config) {
    let graph = Graph::try_from(config.get_entrypoint().unwrap()).unwrap();

    let generated_out_path =
        AbsoluteFilePathBuf::try_from(config.get_generated_out_path().to_path_buf()).unwrap();

    let mut bufs = graph
        .as_nodes()
        .iter()
        .map(|node| node.as_path())
        .filter(|path| **path != generated_out_path)
        .map(|path| path.as_path())
        .collect::<Vec<_>>();
    bufs.sort();
    for buf in bufs.iter() {
        println!("{}", buf.to_string_lossy());
    }
}
