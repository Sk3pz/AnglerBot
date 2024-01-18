use std::fmt::{Display, Formatter};
use std::path::Path;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::data::multipliers::MultiplierData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RodType {
    name: String,
    base_cost: u32,
    base_catch_rate: f32,
    base_casting_depth: u32,
    base_weight_limit: u32,
    description: String,
    pub shop_rarity: String,
}

impl Display for RodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RodMaterial {
    name: String,
    base_cost: u32,
    base_catch_rate: f32,
    base_casting_depth: u32,
    base_weight_limit: u32,
    description: String,
    pub shop_rarity: String,
}

impl Display for RodMaterial {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RodData {
    rod_types: Vec<RodType>,
    rod_materials: Vec<RodMaterial>,
}

impl RodData {
    // load from `data/old_rods.json` using serde
    pub fn load() -> Self {
        let raw_path = "./data/old_rods.json".to_string();
        let path = Path::new(raw_path.as_str());

        if !path.exists() {
            panic!("Failed to load rod data: file does not exist");
        }

        let contents = std::fs::read_to_string(path).unwrap();

        serde_json::from_str(contents.as_str()).unwrap()
    }

    pub fn mat_from_string<S: Into<String>>(&self, material: S) -> Option<RodMaterial> {
        let search: String = material.into();
        for mat in self.rod_materials.iter() {
            if mat.name == search {
                return Some(mat.clone());
            }
        }
        None
    }

    pub fn type_from_string<S: Into<String>>(&self, rod_type: S) -> Option<RodType> {
        let search: String = rod_type.into();
        for rod in self.rod_types.iter() {
            if rod.name == search {
                return Some(rod.clone());
            }
        }
        None
    }

    pub fn random_mat(&self) -> RodMaterial {
        let rarity = crate::data::shop::RodRarity::random();
        let mut rng = rand::thread_rng();
        // generate a random rod until its rarity matches the rarity we want
        for _ in 0..100 {
            let random_mat = rng.gen_range(0..self.rod_materials.len());
            let mat = self.rod_materials.get(random_mat).unwrap();
            if mat.shop_rarity == rarity.to_string() {
                return mat.clone();
            }
        }
        // default in case we can't find a rod with the correct rarity
        return self.rod_materials.first().unwrap().clone();
    }

    pub fn random_type(&self) -> RodType {
        let rarity = crate::data::shop::RodRarity::random();
        let mut rng = rand::thread_rng();
        // generate a random rod until its rarity matches the rarity we want
        for _ in 0..100 {
            let random_type = rng.gen_range(0..self.rod_types.len());
            let rtype = self.rod_types.get(random_type).unwrap();
            if rtype.shop_rarity == rarity.to_string() {
                return rtype.clone();
            }
        }
        // default in case we can't find a rod with the correct rarity
        return self.rod_types.first().unwrap().clone();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rod {
    rod_type: String,
    material: String,
}

impl Rod {
    pub fn new(rod_type: String, material: String) -> Self {
        Self {
            rod_type,
            material,
        }
    }

    // todo: add weights to each rod type and material to make them more or less rare
    pub fn generate(rod_data: &RodData) -> Self {

        let rod_type = rod_data.random_type();
        let mut rod_material = rod_data.random_mat();

        if rod_type.name == "Jigstick" {
            rod_material = rod_data.mat_from_string("Chinesium").unwrap();
        } else {
            while rod_material.name == "Chinesium" {
                rod_material = rod_data.random_mat();
            }
        }

        Self {
            rod_type: rod_type.name.clone(),
            material: rod_material.name.clone(),
        }
    }

    pub fn get_name(&self) -> String {
        format!("{} {}", self.material, self.rod_type)
    }

    pub fn get_rod_type(&self, data: &RodData) -> RodType {
        data.type_from_string(&self.rod_type).unwrap()
    }

    pub fn get_rod_material(&self, data: &RodData) -> RodMaterial {
        data.mat_from_string(&self.material).unwrap()
    }

    pub fn get_cost(&self, data: &RodData) -> u32 {
        self.get_rod_type(data).base_cost + self.get_rod_material(data).base_cost
    }

    pub fn get_catch_rate(&self, data: &RodData) -> f32 {
        self.get_rod_type(data).base_catch_rate + self.get_rod_material(data).base_catch_rate
    }

    pub fn get_casting_depth(&self, data: &RodData) -> u32 {
        self.get_rod_type(data).base_casting_depth + self.get_rod_material(data).base_casting_depth
    }

    pub fn get_rod_weight_limit(&self, data: &RodData) -> u32 {
        self.get_rod_type(data).base_weight_limit + self.get_rod_material(data).base_weight_limit
    }

    pub fn random_catch_time(&self, data: &RodData) -> f32 {
        let catch_rate = self.get_catch_rate(data);

        let mut rng = rand::thread_rng();

        let random_multiplier = rng.gen_range(0.8..1.2);

        let multipliers = MultiplierData::load();

        let time = catch_rate * random_multiplier;

        time / multipliers.cast_time_divisor
    }
}

impl Display for Rod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}