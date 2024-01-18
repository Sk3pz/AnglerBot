use std::fmt::Display;
use chrono::{Duration, Local, NaiveDateTime};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serenity::all::UserId;
use crate::data::rods::{Rod, RodData};
use crate::say;

fn next_midnight() -> NaiveDateTime {
    let now = Local::now();

    (now + Duration::days(1)).date_naive().and_hms_opt(0, 0, 0).unwrap()
}

pub enum RodRarity {
    Common, // 40%
    Uncommon, // 30%
    Rare, // 20%
    Epic, // 10%
    Unobtainable, // 0%
}

impl RodRarity {
    pub fn get_weight(&self) -> u64 {
        match self {
            RodRarity::Common => 4500,
            RodRarity::Uncommon => 3000,
            RodRarity::Rare => 2000,
            RodRarity::Epic => 1000,
            RodRarity::Unobtainable => 0,
        }
    }

    pub fn get_ident(&self) -> u8 {
        match self {
            RodRarity::Common => 0,
            RodRarity::Uncommon => 1,
            RodRarity::Rare => 2,
            RodRarity::Epic => 3,
            RodRarity::Unobtainable => 4,
        }
    }

    pub fn is_rarer(&self, other: &Self) -> bool {
        self.get_ident() > other.get_ident()
    }

    pub fn from_string<S: Into<String>>(string: S) -> Self {
        let from = string.into();
        match from.as_str() {
            "Common" => RodRarity::Common,
            "Uncommon" => RodRarity::Uncommon,
            "Rare" => RodRarity::Rare,
            "Epic" => RodRarity::Epic,
            "Unobtainable" => RodRarity::Unobtainable,
            _ => RodRarity::Common,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let num = rng.gen_range(0..10000);

        let mut combined = 0;
        if num < (combined + RodRarity::Common.get_weight()) {
            return RodRarity::Common;
        }
        combined += RodRarity::Common.get_weight();
        if num < (combined + RodRarity::Uncommon.get_weight()) {
            return RodRarity::Uncommon;
        }
        combined += RodRarity::Uncommon.get_weight();
        if num < (combined + RodRarity::Rare.get_weight()) {
            return RodRarity::Rare;
        }
        combined += RodRarity::Rare.get_weight();
        if num < (combined + RodRarity::Epic.get_weight()) {
            return RodRarity::Epic;
        }
        combined += RodRarity::Epic.get_weight();
        if num < (combined + RodRarity::Unobtainable.get_weight()) {
            return RodRarity::Unobtainable;
        }
        RodRarity::Common
    }
}

impl Display for RodRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RodRarity::Common => write!(f, "Common"),
            RodRarity::Uncommon => write!(f, "Uncommon"),
            RodRarity::Rare => write!(f, "Rare"),
            RodRarity::Epic => write!(f, "Epic"),
            RodRarity::Unobtainable => write!(f, "Unobtainable"),
        }
    }
}

pub enum BuyError {
    NoMoney,
    InvalidRod,
}

impl Display for BuyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuyError::NoMoney => write!(f, "You don't have enough money to buy this rod!"),
            BuyError::InvalidRod => write!(f, "That rod is no longer available!"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shop {
    pub refresh: i64,
    pub rods: Vec<String>,
    // todo: bait goes here
}

impl Shop {

    pub fn create() -> Shop {
        let rod_data = RodData::load();

        // create a shop
        let mut shop = Self {
            refresh: next_midnight().timestamp(),
            rods: Vec::new(),
        };

        // generate daily rods rods and add them to the shop
        shop.rods.push(rod_data.generate_rod_base(RodRarity::Common).name);
        shop.rods.push(rod_data.generate_rod_base(RodRarity::Common).name);

        shop.rods.push(rod_data.generate_rod_base(RodRarity::Uncommon).name);
        shop.rods.push(rod_data.generate_rod_base(RodRarity::Uncommon).name);

        shop.rods.push(rod_data.generate_rod_base(RodRarity::Rare).name);

        shop.rods.push(rod_data.generate_rod_base(RodRarity::Epic).name);

        // write the shop to a file
        let serialized = serde_json::to_string(&shop).unwrap();

        let raw_path = "./data/shop.json".to_string();

        let path = std::path::Path::new(raw_path.as_str());

        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        }

        std::fs::write(path, serialized).unwrap();

        shop
    }

    pub fn get_time_until_restock(&self) -> String {
        // from now to next midnight
        let now = Local::now();
        let next_midnight = next_midnight();

        let duration = next_midnight - now.naive_local();

        let hours = duration.num_hours();
        let minutes = duration.num_minutes() - (hours * 60);
        let seconds = duration.num_seconds() - (minutes * 60) - (hours * 60 * 60);

        format!("{} hours, {} minutes, {} seconds", hours, minutes, seconds)
    }

    pub fn should_refresh(&self) -> bool {
        let now = Local::now();
        let refresh = NaiveDateTime::from_timestamp_opt(self.refresh, 0).unwrap().and_local_timezone(Local).unwrap();

        // if more than 24 hours have passed, refresh the shop
        if now >= refresh {
            return true;
        }
        false
    }

    pub fn load() -> Self {
        let raw_path = "./data/shop.json".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        if !path.exists() {
            return Self::create();
        }

        let contents = std::fs::read_to_string(path).unwrap();

        let mut shop: Shop = serde_json::from_str(contents.as_str()).unwrap();

        // check if refresh is needed
        if shop.should_refresh() {
            say!("Refreshing shop");
            let mut new_shop = Self::create();
            new_shop.refresh = next_midnight().timestamp();
            shop = new_shop;
        }

        shop
    }

    pub fn sell_rod(&mut self, spot: usize, guild_id: u64, user_id: &UserId) -> Result<String, BuyError> {
        if spot >= self.rods.len() {
            return Err(BuyError::InvalidRod);
        }

        let rod_data = RodData::load();

        let base_rod_name = self.rods.get(spot).unwrap().clone();
        let base_rod = rod_data.get_base_by_name(base_rod_name).unwrap();

        let mut user_file = crate::data::userfile::read_userfile(user_id, guild_id);

        let multiplier = crate::data::multipliers::MultiplierData::load();

        let mut cost = base_rod.cost;

        if multiplier.shop_discount != 0.0 {
            cost -= cost * multiplier.shop_discount;
        }

        let cost = cost.round() as u32;

        // ensure the user has enough money to buy the rod
        if user_file.money < cost {
            return Err(BuyError::NoMoney);
        }

        // remove the rod from the shop
        //let users_rod = self.rods.remove(spot);

        // remove the money from the user
        user_file.money -= cost;

        let modifier = rod_data.generate_modifier();

        // generate the rod
        let rod = Rod {
            base: base_rod,
            modifier: modifier.clone(),
        };

        // add the rod to the user's inventory
        user_file.rod_name = rod.base.name.clone();
        if let Some(m) = &rod.modifier {
            user_file.rod_modifier = Some(m.name.clone());
        } else {
            user_file.rod_modifier = None;
        }

        // update the user's file
        crate::data::userfile::update_userfile(user_id, user_file, guild_id);

        if let Some(m) = &modifier {
            Ok(format!("You now own a {}!\n  Your rod has a modifier: {}!", rod, m.name))
        } else {
            Ok(format!("You now own a {}!", rod))
        }
    }

}