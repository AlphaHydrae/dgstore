use ansi_term::Colour::Red;
use clap::{App, Arg};
use dgstore::Config;
use std::env;
use std::process;

fn main() {
    let matches = App::new("dgstore")
        .version("0.1.0")
        .author("Simon Oulevay <rust@alphahydrae.com>")
        .about("Hash files and store the digests next to the files for future comparison")
        .arg(
            Arg::new("write")
                .short('w')
                .long("write")
                .about("Whether to save digest files")
                .takes_value(true),
        )
        .arg(
            Arg::new("files")
                .about("The files to hash")
                .required(true)
                .min_values(1),
        )
        .get_matches_from(env::args());

    let patterns: Vec<String> = matches
        .values_of_t("files")
        .unwrap_or_else(|err| err.exit());
    let write: bool = matches.value_of_t("write").unwrap_or(true);

    let config = Config::new(patterns, write).unwrap_or_else(|err| {
        println!("{}", Red.paint(format!("{}", err)));
        process::exit(1);
    });

    if let Err(err) = dgstore::compute_and_store_digests(config) {
        println!("{}", Red.paint(format!("{}", err)));
        process::exit(2);
    }
}
