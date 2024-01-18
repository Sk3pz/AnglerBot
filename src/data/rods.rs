use std::fmt::{Display, Formatter};
use std::path::Path;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::data::multipliers::MultiplierData;
use crate::data::shop::RodRarity;

// https://docs.google.com/spreadsheets/d/1k_U3l-JPknjTrtXBf2-Y2J1kPrGCJLQs5KjHkO0jQ_E/edit?usp=sharing

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRod {
    pub name: String,
    pub description: String,
    pub catch_chance: f32,
    pub catch_rate: f32,
    pub depth: u32,
    pub weight_limit: u32,
    pub cost: f32,
    pub rarity: String,
}

impl BaseRod {
    pub fn get_rarity(&self) -> RodRarity {
        RodRarity::from_string(self.rarity.clone())
    }
}

// todo this should change price as well?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RodModifier {
    pub name: String,
    pub catch_chance: f32,
    pub catch_rate: f32,
    pub depth: i32,
    pub weight_limit: i32,
}

impl Display for RodModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RodData {
    pub rods: Vec<BaseRod>,
    pub modifiers: Vec<RodModifier>
}

impl RodData {
    pub fn load() -> Self {
        let raw_path = "./data/rods.json".to_string();
        let path = Path::new(raw_path.as_str());

        if !path.exists() {
            panic!("Failed to load rod data: file does not exist");
        }

        let contents = std::fs::read_to_string(path).unwrap();

        serde_json::from_str(contents.as_str()).unwrap()
    }

    pub fn get_base_by_name<S: Into<String>>(&self, name: S) -> Option<BaseRod> {
        let name = name.into();
        self.rods.clone().into_iter().find(|rod| rod.name == name)
    }

    pub fn get_modifier_by_name<S: Into<String>>(&self, name: S) -> Option<RodModifier> {
        let name = name.into();
        self.modifiers.clone().into_iter().find(|modifier| modifier.name == name)
    }

    pub fn generate_modifier(&self) -> Option<RodModifier> {
        let mut rng = rand::thread_rng();
        let chance = rng.gen_range(0..100);

        if chance < 10 {
            let index = rng.gen_range(0..self.modifiers.len());
            Some(self.modifiers[index].clone())
        } else {
            None
        }
    }

    pub fn generate_rod_base(&self, rarity: RodRarity) -> BaseRod {
        let mut rng = rand::thread_rng();

        let mut rods: Vec<&BaseRod> = self.rods.iter().filter(|r| r.get_rarity().get_ident() == rarity.get_ident()).collect();
        if rods.is_empty() {
            rods.push(self.rods.get(rng.gen_range(0..self.rods.len())).unwrap());
        }

        rods[rng.gen_range(0..rods.len())].clone()
    }

    pub fn generate_rod(&self, rarity: RodRarity) -> Rod {
        let base = self.generate_rod_base(rarity);
        let modifier = self.generate_modifier();

        Rod {
            base,
            modifier
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rod {
    pub base: BaseRod,
    pub modifier: Option<RodModifier>,
}

impl Rod {
    pub fn get_catch_rate(&self) -> u32 {
        let mut catch_rate = self.base.catch_rate;
        if let Some(m) = self.modifier.clone() {
            catch_rate += m.catch_rate;
        }

        catch_rate as u32
    }

    pub fn get_catch_chance(&self) -> u32 {
        let mut catch_chance = self.base.catch_chance;
        if let Some(m) = self.modifier.clone() {
            catch_chance += m.catch_chance;
        }

        (catch_chance * 1000.0).round() as u32
    }

    pub fn get_depth(&self) -> u32 {
        let mut depth = self.base.depth as i32;
        if let Some(m) = self.modifier.clone() {
            depth += m.depth;
        }

        depth as u32
    }

    pub fn get_weight_limit(&self) -> u32 {
        let mut weight_limit = self.base.weight_limit as i32;
        if let Some(m) = self.modifier.clone() {
            weight_limit += m.weight_limit;
        }

        weight_limit as u32
    }

    pub fn random_catch_time(&self) -> f32 {
        let catch_rate = self.get_catch_rate() as f32;
        let mut rng = rand::thread_rng();

        let random_multiplier = rng.gen_range(0.8..1.2);

        let multipliers = MultiplierData::load();

        let time = catch_rate * random_multiplier;

        time / multipliers.cast_time_divisor
    }
}

impl Display for Rod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}",
               if let Some(m) = self.modifier.clone() { format!("{} ", m) }
        else { "".to_string() }, self.base.name)
    }
}