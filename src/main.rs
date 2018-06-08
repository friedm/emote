extern crate toml;
extern crate clap;

use clap::{Arg, App};
use toml::Value;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const AMOJI_TOML: &'static str = include_str!("amoji.toml");

fn main() {
    let value = AMOJI_TOML.parse::<Value>().expect("invalid toml in amoji.toml!");

    let cli_matches = App::new(NAME)
                          .version(VERSION)
                          .arg(Arg::with_name("clipboard")
                               .short("c")
                               .help("copy output to clipboard"))
                          .arg(Arg::with_name("text")
                               .required(true)
                               .index(1))
                          .get_matches();
    let use_clipboard = cli_matches.is_present("clipboard");
    let input = cli_matches.value_of("text").unwrap();

    println!("{:?} {:?}", use_clipboard, input);
}
