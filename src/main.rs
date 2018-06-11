extern crate app_dirs;
extern crate blake2;
extern crate clap;
extern crate time;
extern crate toml;
extern crate serde_json;
extern crate stderrlog;
extern crate regex;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

use std::env;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;

use app_dirs::{AppInfo, app_root, app_dir, AppDataType};
use clap::{Arg, App};

mod emote;
mod emote_map;
mod util;

use emote::Emote;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

const APP_INFO: AppInfo = AppInfo{name: NAME, author: AUTHOR};

fn main() {
    let cli_matches = App::new(NAME)
                          .version(VERSION)
                          .arg(Arg::with_name("text")
                               .index(1))
                          .arg(Arg::with_name("debug")
                               .short("d")
                               .long("debug"))
                          // TODO bash completion (gen_completions)
                          .get_matches();

    if cli_matches.is_present("debug") {
        stderrlog::new().module(module_path!())
                        .verbosity(3)
                        .init().unwrap();
    }

    let mut emote = Emote::new(get_config_path(),
                           get_cache_path("config.hash"),
                           get_cache_path("emote.map"));
    emote.setup().expect("failed setup!");

    if cli_matches.is_present("text") {
        let text = cli_matches.value_of("text").unwrap();
        match emote.get(text) {
            Some(emote) => {
                println!("{}", emote);
            }
            None => {
                println!("no match for {}", text);
                exit(1);
            }
        }
    } else {
        info!("reading from stdin");
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            println!("{}", emote.replace(&line.unwrap()));
        }
    }
}

fn get_config_path() -> PathBuf {
    let config_dir_env = env::var("EMOTE_CONFIG_DIR");
    let config_path = if config_dir_env.is_ok() {
        let path = PathBuf::from(config_dir_env.unwrap());
        std::fs::create_dir_all(&path).expect("failed to create config dir!");
        path
    } else {
        app_root(AppDataType::UserConfig, &APP_INFO).expect("failed to get config path!")
    };

    config_path.join(&format!("{}.toml", NAME))
}

fn get_cache_path(s: &'static str) -> PathBuf {
    let cache_dir_env = env::var("EMOTE_CACHE_DIR");
    let cache_path = if cache_dir_env.is_ok() {
        let path = PathBuf::from(cache_dir_env.unwrap());
        std::fs::create_dir_all(&path).expect("failed to create cache dir!");
        path
    } else {
        app_dir(AppDataType::UserCache, &APP_INFO, "cache").expect("failed to get cache path!")
    };
    cache_path.join(s)
}
