use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use toml::Value;
use serde_json;

use util::FileUtil;

pub struct EmoteMap {
    map: HashMap<String, String>,
    persist_file: FileUtil
}

impl EmoteMap {
    pub fn new(path: PathBuf) -> EmoteMap {
        EmoteMap {
            map: HashMap::new(),
            persist_file: FileUtil::new(path)
        }
    }

    pub fn build(&mut self, toml_string: String) {
        let value = toml_string.parse::<Value>().expect("invalid toml in emote.toml!");
        self.map = build_emote_map(&value)
    }
    
    pub fn load(&mut self) -> io::Result<()> {
        let json = self.persist_file.read()?;
        self.map = serde_json::from_str(&json)?;
        Ok(())
    }

    pub fn get<'a>(&self, key: &'a str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn persist(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.map)?;
        self.persist_file.write(&json)?;
        Ok(())
    }

    pub fn has_been_persisted(&self) -> bool {
        self.persist_file.exists()
    }
}

fn build_emote_map(toml_value: &Value) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for (_, item) in toml_value["multi"].as_table()
                                        .expect("unexpected multi toml!")
                                        .into_iter() {
        let item = item.as_table()
                       .expect(&format!("unexpected multi toml: {:?}", item));
        let multi = item["multi"].as_str()
                                 .expect(&format!("missing multi in toml: {:?}", item))
                                 .clone();
        let words = item["words"].as_array()
                                 .expect(&format!("missing words in toml: {:?}", item));
        for word in words {
            let word = word.as_str()
                           .expect(&format!("missing word in toml: {:?}", item));
            map.insert(word.to_string(), multi.to_string());
        }
    }

    map
}
