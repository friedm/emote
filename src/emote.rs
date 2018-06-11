use std::io;
use std::path::PathBuf;

use time::precise_time_ns;
use regex::Regex;

use emote_map::EmoteMap;
use util::FileUtil;

const DEFAULT_CONFIG: &'static str = include_str!("defaults.toml");

pub struct Emote {
    config_file: FileUtil,
    map: EmoteMap // TODO emoji support (separate map)
}

impl Emote {
    pub fn new(config_path: PathBuf,
               config_hash_path: PathBuf,
               map_path: PathBuf) -> Emote {
        Emote {
            config_file: FileUtil::new_hashed(config_path,
                                              config_hash_path),
            map: EmoteMap::new(map_path)
        }
   }

    pub fn setup(&mut self) -> io::Result<()> {
        let t0 = precise_time_ns();
        self.setup_config()?;
        let t1 = precise_time_ns();
        self.load_and_cache_emotes()?;
        let t2 = precise_time_ns();
        info!("setup:  {}us", (t1-t0)/1000);
        info!("load:   {}us", (t2-t1)/1000);
        info!("total:  {}us", (t2-t0)/1000);
        Ok(())
    }

    fn setup_config(&mut self) -> io::Result<()> {
        if !self.config_file.exists() {
            self.config_file.write(DEFAULT_CONFIG)?;
        }
        Ok(())
    }

    fn load_and_cache_emotes(&mut self) -> io::Result<()> {
        if !self.map.has_been_persisted() || self.config_file.is_stale()? {
            info!("building emote map");
            self.build_and_cache_map()
        } else {
            info!("loading emote map from disk");
            // read map from disk
            match self.map.load() {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("failed to load, building instead: {}", e);
                    self.build_and_cache_map()
                }
            }
        }
    }
    
    fn build_and_cache_map(&mut self) -> io::Result<()> {
        self.map.build(self.config_file.read()?);
        self.map.persist()?;
        self.config_file.store_hash()?;
        Ok(())
    }

    pub fn get<'a>(&self, key: &'a str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn replace<'a>(&self, s: &'a str) -> String {
        lazy_static! {
            static ref RE: Regex = Regex::new(r":[a-zA-Z_]+?:").unwrap();
        }
        let mut o = String::with_capacity(s.len());
        let mut last_match = 0usize;
        for m in RE.find_iter(&s) {
            let l: usize = m.end()-m.start();
            o.push_str(&s[last_match..m.start()]);
            let key = &m.as_str()[1..l-1];
            let r = self.map.get(key);
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
}
