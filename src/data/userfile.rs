use std::path::Path;
use serde::{Deserialize, Serialize};
use serenity::all::UserId;
use crate::data::rods::{Rod, RodData};
use crate::nay;

const USERFILES_DIR: &str = "./data/guilds/";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserValues {
    pub fish_caught: u32,
    pub money: u32,
    pub rod_name: String,
    pub rod_modifier: Option<String>,
    // todo: bait
    pub cast: bool,
    pub has_seen: Vec<String>
}

impl UserValues {
    pub fn get_rod(&self) -> Rod {
        let rod_data = RodData::load();

        let rod_base = rod_data.get_base_by_name(self.rod_name.as_str()).unwrap();

        let modifier = self.rod_modifier.as_ref().map(|modifier|
            rod_data.get_modifier_by_name(modifier.as_str()).unwrap());

        Rod {
            base: rod_base,
            modifier,
        }
    }
}

impl Default for UserValues {
    fn default() -> Self {
        Self {
            fish_caught: 0,
            money: 0,
            rod_name: "Stick with String".to_string(),
            rod_modifier: None,
            cast: false,
            has_seen: vec![]
        }
    }
}

pub fn get_userfile_path(id: &UserId, guild_id: u64) -> String {
    format!("{}{}/users/{}.json", USERFILES_DIR, guild_id, id)
}

pub fn set_userfile_casting_false(path: String) {
    let raw_path = path.to_string();
    let path = Path::new(raw_path.as_str());

    if !path.exists() {
        return;
    }

    let contents = std::fs::read_to_string(path).unwrap();

    let mut user_values: UserValues = serde_json::from_str(contents.as_str()).unwrap();
    user_values.cast = false;

    // write the file
    let serialized = serde_json::to_string(&user_values).unwrap();

    std::fs::write(path, serialized).unwrap();
}

pub fn userfile_exists(id: &UserId, guild_id: u64) -> bool {
    let raw_path = get_userfile_path(id, guild_id);
    let path = Path::new(raw_path.as_str());

    path.exists()
}

pub fn create_userfile(id: &UserId, guild_id: u64) {
    let raw_path = get_userfile_path(id, guild_id);
    let path = Path::new(raw_path.as_str());

    if !path.exists() {
        // make the directories
        if let Err(e) = std::fs::create_dir_all(path.parent().unwrap()) {
            nay!("Failed to create userfile directories: {}", e);
            return;
        }
        // create the file
        if let Err(e) = std::fs::File::create(path) {
            nay!("Failed to create userfile: {}", e);
            return;
        }
    }

    let user_values = UserValues::default();

    let serialized = serde_json::to_string(&user_values).unwrap();

    std::fs::write(path, serialized).unwrap();
}

pub fn read_userfile(id: &UserId, guild_id: u64) -> UserValues {
    let raw_path = get_userfile_path(id, guild_id);
    let path = Path::new(raw_path.as_str());

    if !path.exists() {
        create_userfile(id, guild_id);
    }

    let contents = std::fs::read_to_string(path).unwrap();

    serde_json::from_str(contents.as_str()).unwrap()
}

pub fn update_userfile(id: &UserId, user_values: UserValues, guild_id: u64) {
    let raw_path = get_userfile_path(id, guild_id);
    let path = Path::new(raw_path.as_str());

    if !path.exists() {
        create_userfile(id, guild_id);
    }

    let serialized = serde_json::to_string(&user_values).unwrap();

    std::fs::write(path, serialized).unwrap();
}
