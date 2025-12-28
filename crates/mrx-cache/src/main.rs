use mrx_cache::{cache, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    cache(&config, &options).unwrap();
}
