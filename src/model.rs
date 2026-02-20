use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Actor {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub actor_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Fight {
    pub id: u32,
    #[serde(rename = "encounterID")]
    pub encounter_id: u32,
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
    pub kill: Option<bool>,
    #[serde(rename = "friendlyPlayers")]
    pub friendly_players: Vec<i32>,
}

impl Fight {
    pub fn is_real_encounter(&self) -> bool {
        self.encounter_id != 0 && self.kill.is_some()
    }
}

#[derive(Debug, Deserialize)]
pub struct DeathEvent {
    pub timestamp: f64,
    #[serde(rename = "targetID")]
    pub target_id: i32,
    #[serde(rename = "killingAbilityGameID")]
    pub killing_ability_game_id: Option<u32>,
}