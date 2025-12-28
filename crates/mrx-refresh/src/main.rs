use mrx_refresh::{refresh, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    refresh(&config, &options);
}
