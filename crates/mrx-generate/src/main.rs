use mrx_generate::{generate, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    generate(config, options).unwrap();
}
