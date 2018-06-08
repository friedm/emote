extern crate toml;
extern crate clap;

use std::process::exit;
use std::collections::HashMap;

use clap::{Arg, App};
use toml::Value;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const AMOJI_TOML: &'static str = include_str!("amoji.toml");

fn main() {
    let value = AMOJI_TOML.parse::<Value>().expect("invalid toml in amoji.toml!");
    let amoji_map = build_amoji_map(&value);
    // TODO avoid loading the entire map on every invocation -- cache serialized HashMap?

    let cli_matches = App::new(NAME)
                          .version(VERSION)
                          .arg(Arg::with_name("clipboard")
                               .short("c")
                               .help("copy output to clipboard"))
                          .arg(Arg::with_name("text")
                               .required(true)
                               .index(1))
                          // TODO list subcommand
                          // TODO bash completion (gen_completions)
                          .get_matches();
    let use_clipboard = cli_matches.is_present("clipboard");
    let text = cli_matches.value_of("text").unwrap();
    match amoji_map.get(text) {
        Some(amoji) => println!("{}", amoji),
        None => {
            println!("no match for {}", text);
            exit(1);
        }
    }
}

fn build_amoji_map<'a>(toml_value: &'a Value) -> HashMap<&'a str, &'a str> {
    let mut map = HashMap::new();

    for (_, item) in toml_value["amoji"].as_table()
                                        .expect("unexpected amoji toml!")
                                        .into_iter() {
        let item = item.as_table()
                       .expect(&format!("unexpected amoji toml: {:?}", item));
        let amoji = item["amoji"].as_str()
                                 .expect(&format!("missing amoji in toml: {:?}", item))
                                 .clone();
        let words = item["words"].as_array()
                                 .expect(&format!("missing words in toml: {:?}", item));
        for word in words {
            let word = word.as_str()
                           .expect(&format!("missing word in toml: {:?}", item));
            map.insert(word, amoji);
        }
    }

    map
}
