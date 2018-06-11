use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use blake2::{Blake2b, Digest};

pub struct FileUtil {
    path: PathBuf,
    hash_path: Option<PathBuf>
}

impl FileUtil {
    pub fn new(path: PathBuf) -> FileUtil {
        FileUtil {
            path: path,
            hash_path: None
        }
    }

    pub fn new_hashed(path: PathBuf, hash_path: PathBuf) -> FileUtil {
        FileUtil {
            path: path,
            hash_path: Some(hash_path)
        }
    }

    pub fn write<'a>(&self, to_write: &'a str) -> io::Result<()> {
        write(&self.path, to_write)?;
        self.store_hash()
    }

    pub fn read<'a>(&self) -> io::Result<String> {
        read(&self.path)
    }

    pub fn store_hash(&self) -> io::Result<()> {
        if self.hash_path.is_none() {
            return Ok(())
        }
        let hash = self.get_hash()?;
        let hash_path = self.hash_path.clone().unwrap();
        write(&hash_path, &hash)
    }

    pub fn is_stale(&self) -> io::Result<bool> {
        if self.hash_path.is_none() ||
           !&self.hash_path.clone().unwrap().is_file() {
            return Ok(true);
        }
        let cached_hash = read(&self.hash_path.clone().unwrap())?;
        Ok(self.get_hash()? != cached_hash)
    }

    fn get_hash(&self) -> io::Result<String> {
        let mut f = File::open(&self.path)?;
        let hash = Blake2b::digest_reader(&mut f)?; 
        Ok(format!("{:x}", hash))
    }

    pub fn exists(&self) -> bool {
        self.path.is_file()
    }
}

fn write<'a>(path: &PathBuf, s: &'a str) -> io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(s.as_bytes())?;
    f.sync_all()?;
    Ok(())
}

fn read(path: &PathBuf) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut config = String::new();
    f.read_to_string(&mut config)?;
    Ok(config)
}
