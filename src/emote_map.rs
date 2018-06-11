use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use toml::Value;
use serde_json;

use util::FileUtil;

#[derive(Serialize, Deserialize)]
pub struct EmoteMap {
    map: HashMap<String, String>
}

impl EmoteMap {
    pub fn build(toml_string: String) -> EmoteMap {
        let value = toml_string.parse::<Value>().expect("invalid toml in emote.toml!");
        EmoteMap {
            map: build_emote_map(&value)
        }
    }
    
    pub fn load(path: PathBuf) -> io::Result<EmoteMap> {
        let json = FileUtil::new(path).read()?;
        let map = serde_json::from_str(&json)?;
        Ok(EmoteMap {
            map: map
        })
    }

    pub fn get<'a>(&self, key: &'a str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn persist(&self, path: PathBuf) -> io::Result<()> {
        let json = serde_json::to_string(&self.map)?;
        FileUtil::new(path).write(&json)?;
        Ok(())
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
