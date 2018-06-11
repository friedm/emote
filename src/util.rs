use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub struct FileUtil {
    path: PathBuf
}

impl FileUtil {
    pub fn new(path: PathBuf) -> FileUtil {
        FileUtil {
            path: path
        }
    }

    pub fn write<'a>(&self, to_write: &'a str) -> io::Result<()> {
        write(&self.path, to_write)
    }

    pub fn read<'a>(&self) -> io::Result<String> {
        read(&self.path)
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


