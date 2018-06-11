extern crate app_dirs;
extern crate blake2;
extern crate clap;
extern crate time;
extern crate toml;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate stderrlog;
extern crate regex;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;

use app_dirs::{AppInfo, app_root, app_dir, AppDataType};
use blake2::{Blake2b, Digest};
use clap::{Arg, App};
use time::precise_time_ns;
use regex::Regex;

mod emote_map;
mod util;

use emote_map::EmoteMap;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

const APP_INFO: AppInfo = AppInfo{name: NAME, author: AUTHOR};

const DEFAULT_CONFIG: &'static str = include_str!("defaults.toml");

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


    let t0 = precise_time_ns();
    setup_config().expect("failed to write default config file!");
    let t1 = precise_time_ns();
    let emote_map = load_and_cache_emotes().expect("failed to load emote map!");
    let t2 = precise_time_ns();
    info!("setup:  {}us", (t1-t0)/1000);
    info!("load:   {}us", (t2-t1)/1000);
    info!("total:  {}us", (t2-t0)/1000);

    if cli_matches.is_present("text") {
        let text = cli_matches.value_of("text").unwrap();
        match emote_map.get(text) {
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
            println!("{}", replace_matches_in_text(line.unwrap(), &emote_map));
        }
    }
}

fn replace_matches_in_text<'a>(s: String, map: &'a EmoteMap) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r":[a-zA-Z_]+?:").unwrap();
    }
    let mut o = String::with_capacity(s.len());
    let mut last_match = 0usize;
    for m in RE.find_iter(&s) {
        let l: usize = m.end()-m.start();
        o.push_str(&s[last_match..m.start()]);
        let key = &m.as_str()[1..l-1];
        let r = map.get(key);
        if r.is_some() {
            o.push_str(r.unwrap());
        } else {
            o.push_str(m.as_str());
        }
        last_match = m.end();
    }
    o.push_str(&s[last_match..s.len()]);
    o
}

fn setup_config() -> std::io::Result<()> {
    if !get_config_path().is_file() {
        write_config(DEFAULT_CONFIG)?;
    }
    Ok(())
}

fn write_config(s: &'static str) -> std::io::Result<()> {
    write(get_config_path(), s)
}

fn write<'a>(path: PathBuf, s: &'a str) -> std::io::Result<()> { // TODO clean up
    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())?;
    f.sync_all()?;
    Ok(())
}

fn read(path: PathBuf) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut config = String::new();
    f.read_to_string(&mut config)?;
    Ok(config)
}

fn read_config() -> std::io::Result<String> {
    read(get_config_path())
}

fn is_cached_data_stale() -> std::io::Result<bool> {
    let hash_file = get_cache_path("config.hash");
    if !hash_file.is_file() {
        return Ok(true);
    }
    let mut f = File::open(hash_file)?;
    let mut cached_hash = String::new();
    f.read_to_string(&mut cached_hash)?;
    
    let mut f = File::open(get_config_path())?;
    let hash = Blake2b::digest_reader(&mut f)?; 
    let hash = format!("{:x}", hash);
    Ok(hash != cached_hash)
}

fn update_config_hash() -> std::io::Result<()> {
    let mut f = File::open(get_config_path())?;
    let hash = Blake2b::digest_reader(&mut f)?;
    let hash = format!("{:x}", hash);
    write(get_cache_path("config.hash"), &hash)
}

fn load_and_cache_emotes() -> std::io::Result<EmoteMap> {
    let cache_path = get_cache_path("emote.map");
    if !cache_path.is_file() || is_cached_data_stale()? {
        info!("building emote map");
        build_and_cache_map()
    } else {
        info!("loading emote map from disk");
        // read map from disk
        let map = EmoteMap::load(get_cache_path("emote.map"));
        match map {
            Ok(map) => Ok(map),
            Err(e) => {
                warn!("failed to load, building instead: {}", e);
                build_and_cache_map()
            }
        }
    }
}

fn build_and_cache_map() -> std::io::Result<EmoteMap> {
    // read config
    let config = read_config()?;
    // serialize and write to disk
    let map = EmoteMap::build(config);
    map.persist(get_cache_path("emote.map"))?;
    // write cached sha
    update_config_hash()?;
    Ok(map)
}

// TODO refactor
// TODO emoji support
