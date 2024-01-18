use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::data::multipliers::MultiplierData;
use crate::data::rods::Rod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FishRarity {
    Common, // 40%
    Uncommon, // 30%
    Rare, // 20%
    Elusive, // 8.9%
    Legendary, // 1%
    Mythical, // 0.01%
}

impl FishRarity {
    pub fn get_weight(&self) -> u16 {
        match self {
            FishRarity::Common => 400,
            FishRarity::Uncommon => 300,
            FishRarity::Rare => 200,
            FishRarity::Elusive => 89,
            FishRarity::Legendary => 10,
            FishRarity::Mythical => 1,
        }
    }

    pub fn ident(&self) -> u8 {
        match self {
            FishRarity::Common => 0,
            FishRarity::Uncommon => 1,
            FishRarity::Rare => 2,
            FishRarity::Elusive => 3,
            FishRarity::Legendary => 4,
            FishRarity::Mythical => 5,
        }
    }

    pub fn value_multiplier(&self) -> f32 {
        match self {
            Self::Common => 1.0,
            Self::Uncommon => 1.5,
            Self::Rare => 2.0,
            Self::Elusive => 3.0,
            Self::Legendary => 5.0,
            Self::Mythical => 10.0,
        }
    }

    pub fn weighted_random() -> Self {
        let mut rng = rand::thread_rng();

        let num = rng.gen_range(1..=1000);

        let mut current_weight = 0;

        // hey!("Mythical chance: {} generated: {}",
        //     1000 - (Self::Common.get_weight() + Self::Uncommon.get_weight() + Self::Rare.get_weight() + Self::Elusive.get_weight() + Self::Legendary.get_weight()),
        // num);

        if num <= current_weight + Self::Common.get_weight() {
            return Self::Common;
        }
        current_weight += Self::Common.get_weight();

        if num <= current_weight + Self::Uncommon.get_weight() {
            return Self::Uncommon;
        }
        current_weight += Self::Uncommon.get_weight();

        if num <= current_weight + Self::Rare.get_weight() {
            return Self::Rare;
        }
        current_weight += Self::Rare.get_weight();

        if num <= current_weight + Self::Elusive.get_weight() {
            return Self::Elusive;
        }
        current_weight += Self::Elusive.get_weight();

        if num <= current_weight + Self::Legendary.get_weight() {
            return Self::Legendary;
        }
        current_weight += Self::Legendary.get_weight();

        if num <= current_weight + Self::Mythical.get_weight() {
            return Self::Mythical;
        }
        Self::Common
    }

    pub fn from_string<S: Into<String>>(rarity: S) -> Option<Self> {
        let rarity = rarity.into();
        match rarity.as_str() {
            "Common" => Some(Self::Common),
            "Uncommon" => Some(Self::Uncommon),
            "Rare" => Some(Self::Rare),
            "Elusive" => Some(Self::Elusive),
            "Legendary" => Some(Self::Legendary),
            "Mythical" => Some(Self::Mythical),
            _ => None,
        }
    }
}

impl FromStr for FishRarity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Common" => Ok(Self::Common),
            "Uncommon" => Ok(Self::Uncommon),
            "Rare" => Ok(Self::Rare),
            "Elusive" => Ok(Self::Elusive),
            "Legendary" => Ok(Self::Legendary),
            "Mythical" => Ok(Self::Mythical),
            _ => Err(()),
        }
    }
}

impl Display for FishRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FishRarity::Common => write!(f, "Common"),
            FishRarity::Uncommon => write!(f, "Uncommon"),
            FishRarity::Rare => write!(f, "Rare"),
            FishRarity::Elusive => write!(f, "Elusive"),
            FishRarity::Legendary => write!(f, "Legendary"),
            FishRarity::Mythical => write!(f, "Mythical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishType {
    pub name: String,
    pub depth: u32,
    pub value: u32,
    pub min_weight: u32,
    pub avg_weight: u32,
    pub max_weight: u32,
    pub min_rarity: String,
}

impl FishType {
    pub fn random_weight(&self) -> f32 {
        rand::thread_rng().gen_range(self.min_weight as f32..self.max_weight as f32)
    }

    pub fn get_value(&self, weight: f32) -> f32 {
        let fish_data = FishData::load();

        let value_diff = (weight - self.avg_weight as f32) * fish_data.weight_factor;

        let min_value = (self.value / 4) as f32;

        let multipliers = MultiplierData::load();

        (self.value as f32 + value_diff).round().max(min_value) * multipliers.value
    }
}

impl Display for FishType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishData {
    pub weight_factor: f32,
    pub fish: Vec<FishType>,
}

impl FishData {
    pub fn load() -> Self {
        let raw_path = "./data/fish.json".to_string();
        let path = Path::new(raw_path.as_str());

        if !path.exists() {
            panic!("Failed to load rod data: file does not exist");
        }

        let contents = std::fs::read_to_string(path).unwrap();

        serde_json::from_str(contents.as_str()).unwrap()
    }

    pub fn get_fish_above_depth(&self, depth: u32) -> Vec<&FishType> {
        self.fish.iter().filter(|fish| fish.depth <= depth).collect()
    }

    pub fn fish_type_by_name<S: Into<String>>(&self, name: S) -> Option<&FishType> {
        let name = name.into();
        self.fish.iter().find(|fish| fish.name == name.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Fish {
    pub fish_type: FishType,
    pub rarity: FishRarity,
    pub weight: f32,
}

impl Fish {

    pub fn random_fish(fish_data: &FishData, caught_with: &Rod) -> Self {
        let mut rng = rand::thread_rng();

        // generate the fish's rarity
        let rarity = FishRarity::weighted_random();

        // generate the fish type based on depth and randomness
        let fishing_depth = caught_with.get_depth();
        let fish_above_depth = fish_data.get_fish_above_depth(fishing_depth);

        let mut fish_type = fish_above_depth[rng.gen_range(0..fish_above_depth.len())].clone();

        // ensure rare fish only pop up when the rarity is high enough
        while FishRarity::from_string(fish_type.min_rarity.clone()).unwrap().ident() > rarity.ident() {
            fish_type = fish_above_depth[rng.gen_range(0..fish_above_depth.len())].clone();
        }

        // generate the fish's weight
        let weight = (fish_type.random_weight() * 10.0).round() / 10.0;

        Self {
            fish_type,
            rarity: rarity.clone(),
            weight,
        }
    }

    pub fn get_value(&self) -> u32 {
        (self.fish_type.get_value(self.weight) * self.rarity.value_multiplier()) as u32
    }
}

impl Display for Fish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.rarity, self.fish_type)
    }
}