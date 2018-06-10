extern crate app_dirs;
extern crate blake2;
extern crate clap;
extern crate time;
extern crate toml;
extern crate serde_json;

use std::process::exit;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;

use app_dirs::{AppInfo, app_root, app_dir, AppDataType};
use blake2::{Blake2b, Digest};
use clap::{Arg, App};
use time::precise_time_ns;
use toml::Value;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

const APP_INFO: AppInfo = AppInfo{name: NAME, author: AUTHOR};

const DEFAULT_CONFIG: &'static str = include_str!("defaults.toml");

fn main() {
    let cli_matches = App::new(NAME)
                          .version(VERSION)
                          .arg(Arg::with_name("text")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("debug")
                               .short("d")
                               .long("debug"))
                          // TODO bash completion (gen_completions)
                          .get_matches();

    let debug = cli_matches.is_present("debug");
    let text = cli_matches.value_of("text").unwrap();

    let t1 = precise_time_ns();
    setup_config().expect("failed to write default config file!");
    let t2 = precise_time_ns();
    let amoji_map = load_and_cache_amojis().expect("failed to load amoji map!");
    let t3 = precise_time_ns();
    match amoji_map.get(text) {
        Some(amoji) => {
            let t4 = precise_time_ns();
            if debug {
                println!("\
setup:  {}us
load:   {}us
lookup: {}us
total:  {}us",
                         (t2-t1)/1000,
                         (t3-t2)/1000,
                         (t4-t3)/1000,
                         (t4-t1)/1000);
            }
            println!("{}", amoji);
        }
        None => {
            println!("no match for {}", text);
            exit(1);
        }
    }
}

fn setup_config() -> std::io::Result<()> {
    if !get_config_path().is_file() {
        write_config(DEFAULT_CONFIG)?;
    }
    Ok(())
}

fn get_config_path() -> PathBuf {
    let config_path = app_root(AppDataType::UserConfig, &APP_INFO).expect("failed to get config path!");
    config_path.join(&format!("{}.toml", NAME))
}

fn write_config(s: &'static str) -> std::io::Result<()> {
    write(get_config_path(), s)
}

fn write_cached_map<'a>(s: &'a str) -> std::io::Result<()> {
    write(get_cache_path("amoji.map"), s)
}

fn write<'a>(path: PathBuf, s: &'a str) -> std::io::Result<()> { // TODO clean up
    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())?;
    f.sync_all()?;
    Ok(())
}

fn read_config() -> std::io::Result<String> {
    read(get_config_path())
}

fn read_cached_map() -> std::io::Result<String> {
    read(get_cache_path("amoji.map"))
}

fn read(path: PathBuf) -> std::io::Result<String> {
    let mut f = File::open(path)?;
    let mut config = String::new();
    f.read_to_string(&mut config)?;
    Ok(config)
}

fn get_cache_path(s: &'static str) -> PathBuf {
    let cache_path = app_dir(AppDataType::UserCache, &APP_INFO, "cache").expect("failed to get cache path!");
    cache_path.join(s)
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

fn load_and_cache_amojis() -> std::io::Result<HashMap<String, String>> {
    let cache_path = get_cache_path("amoji.map");
    if !cache_path.is_file() || is_cached_data_stale()? {
        println!("building amoji map"); // TODO use debug flag here
        build_and_cache_map()
    } else {
        println!("loading amoji map from disk");
        // read map from disk
        let map = serde_json::from_str(&read_cached_map()?);
        match map {
            Ok(map) => Ok(map),
            Err(_) => {
                println!("failed to load, building instead");
                build_and_cache_map()
            }
        }
    }
}

fn build_and_cache_map() -> std::io::Result<HashMap<String, String>> {
    // read config
    let config = read_config()?;
    // build_amoji_map
    let value = config.parse::<Value>().expect("invalid toml in amoji.toml!");
    // serialize and write to disk
    let map = build_amoji_map(&value);
    let json = serde_json::to_string(&map).expect("error serializing amoji map!");
    write_cached_map(&json);
    // write cached sha
    update_config_hash()?;
    Ok(map)
}

fn build_amoji_map(toml_value: &Value) -> HashMap<String, String> {
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
            map.insert(word.to_string(), amoji.to_string());
        }
    }

    map
}
