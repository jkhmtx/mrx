use mrx_hook::{hook, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    hook(&config, &options);
}
