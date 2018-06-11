use std::collections::HashMap;

use toml::Value;

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

    pub fn get<'a>(&self, key: &'a str) -> Option<&String> {
        self.map.get(key)
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
