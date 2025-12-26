use mrx_show::{show, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    show(config, options);
}
