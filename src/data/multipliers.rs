use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultiplierData {
    pub value: f32,
    pub cast_time_divisor: f32,
    pub catch_chance: u32,
    pub rarity_boost: bool,
    pub shop_discount: f32, // todo
}

impl MultiplierData {
    pub fn load() -> Self {
        let raw_path = "./data/multipliers.json".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        if !path.exists() {
            panic!("Failed to load config: file does not exist");
        }

        let contents = std::fs::read_to_string(path).unwrap();

        serde_json::from_str(contents.as_str()).unwrap()
    }
}