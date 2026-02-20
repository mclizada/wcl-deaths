use serde::Deserialize;

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

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}