#![allow(dead_code, unused_imports)]
extern crate tempfile;

use std::env;
use std::fs;
use std::fs::{File};
use std::path::{PathBuf, Path};
use std::ffi::OsString;
use std::io::prelude::*;
use std::process::Command;

use tempfile::tempdir;

fn get_paths() -> (PathBuf, PathBuf, PathBuf) {
    let mut root = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    if !root.join("emote").is_file() {
        root = root.parent().unwrap().to_path_buf();
    }
    let path = root.clone().join("emote");

    let data = tempdir().unwrap().into_path();
    let config = data.join("config");
    let cache = data.join("cache");

    assert!(!&config.exists(), "data directory should not exist yet!");
    assert!(!&cache.exists(), "data directory should not exist yet!");
    assert!(&path.is_file(), "failed to find binary to test!");
    (path, config, cache)
}

fn file_contains(path: &PathBuf, s: &'static str) -> std::io::Result<bool> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents.contains(s))
}

#[test]
fn can_execute_command() {
    let (path, config, cache) = get_paths();
    assert!(Command::new(path).env("EMOTE_CONFIG_DIR", &config.as_os_str())
                              .env("EMOTE_CACHE_DIR", &cache.as_os_str())
                              .arg("--help").output().is_ok());
}

#[test]
fn creates_config() {
    let (path, config, cache) = get_paths();
    assert!(!config.is_dir());
    assert!(Command::new(path).env("EMOTE_CONFIG_DIR", &config.as_os_str())
                              .env("EMOTE_CACHE_DIR", &cache.as_os_str())
                              .arg("shrug").output().is_ok());
    assert!(config.is_dir());
    let config_path = config.join("emote.toml");
    assert!(config_path.is_file());
    assert!(file_contains(&config_path, "multi").unwrap());
    assert!(file_contains(&config_path, "ƸӜƷ").unwrap());
}

#[test]
fn creates_cache() {
    let (path, config, cache) = get_paths();
    assert!(!cache.is_dir());
    assert!(Command::new(path).env("EMOTE_CONFIG_DIR", &config.as_os_str())
                              .env("EMOTE_CACHE_DIR", &cache.as_os_str())
                              .arg("shrug").output().is_ok());
    assert!(cache.is_dir());
    let config_hash_path = cache.join("config.hash");
    let map_path = cache.join("emote.map");
    assert!(config_hash_path.is_file());
    assert!(map_path.is_file());
}

#[test]
fn outputs_emoticon() {
    let (path, config, cache) = get_paths();
    assert!(!cache.is_dir());
    let output = Command::new(path).env("EMOTE_CONFIG_DIR", &config.as_os_str())
                                   .env("EMOTE_CACHE_DIR", &cache.as_os_str())
                                   .arg("shrug").output().unwrap().stdout;
    assert_eq!("¯\\_(ツ)_/¯\n",
               String::from_utf8(output).unwrap());
}

#[test]
fn loads_cache() {
    let (path, config, cache) = get_paths();
    assert!(!cache.is_dir());
    let output = Command::new(&path).env("EMOTE_CONFIG_DIR", &config.as_os_str())
                                   .env("EMOTE_CACHE_DIR", &cache.as_os_str())
                                   .arg("shrug")
                                   .arg("-d").output().unwrap().stdout;
    let s = String::from_utf8(output).unwrap();
    assert!(s.contains("building emote map"), s);
    assert!(!s.contains("loading"), s);
    let output = Command::new(&path).env("EMOTE_CONFIG_DIR", &config.as_os_str())
                                   .env("EMOTE_CACHE_DIR", &cache.as_os_str())
                                   .arg("shrug")
                                   .arg("-d").output().unwrap().stdout;
    let s = String::from_utf8(output).unwrap();
    assert!(s.contains("loading emote map"), s);
    assert!(!s.contains("building"), s);
}
