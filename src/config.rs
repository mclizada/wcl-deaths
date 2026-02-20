use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub encounter: Vec<EncounterConfig>,
}

#[derive(Debug, Deserialize)]
pub struct EncounterConfig {
    pub id: u32,
    pub name: String,
    pub bad_abilities: Vec<u32>,
}

impl EncounterConfig {
    pub fn is_bad_ability(&self, ability_id: u32) -> bool {
        self.bad_abilities.contains(&ability_id)
    }
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}