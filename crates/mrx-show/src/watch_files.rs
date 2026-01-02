use mrx_utils::{
    Config,
    fs::AbsoluteFilePathBuf,
    graph::Graph,
};

pub(crate) fn watch_files(config: &Config) {
    let graph = Graph::new(config).unwrap();

    let generated_out_path =
        AbsoluteFilePathBuf::try_from(config.get_generated_out_path().as_path()).unwrap();

    let mut bufs = graph
        .to_nodes()
        .into_iter()
        .filter(|path| **path != generated_out_path)
        .map(|path| path.as_relative_to_parent(&config.dir()).unwrap())
        .collect::<Vec<_>>();
    bufs.sort();
    for buf in &bufs {
        println!("{}", buf.to_string_lossy());
    }
}
