use mrx_build::{build, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    build(&config, &options).unwrap();
}
