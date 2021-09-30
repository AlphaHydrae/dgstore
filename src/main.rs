use ansi_term::Colour::Red;
use dgstore::Config;
use std::env;
use std::process;

fn main() {
    let config = Config::new(env::args().skip(1).collect()).unwrap_or_else(|err| {
        println!("{}", Red.paint(format!("{}", err)));
        process::exit(1);
    });

    if let Err(err) = dgstore::compute_and_store_digests(config) {
        println!("{}", Red.paint(format!("{}", err)));
        process::exit(2);
    }
}
