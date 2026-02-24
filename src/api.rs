use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use chrono::DateTime;

use crate::queries;
use crate::state::AppState;

fn ms_to_pacific_date(ms: f64) -> String {
    // PST = UTC-8
    let secs = (ms / 1000.0) as i64 - 8 * 3600;
    let dt = DateTime::from_timestamp(secs, 0).expect("invalid timestamp");
    dt.format("%Y-%m-%d").to_string()
}

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub guild_name: String,
    pub guild_server_slug: String,
    pub guild_server_region: String,
    pub start_time: f64,  // UNIX timestamp in milliseconds
    pub end_time: f64,    // UNIX timestamp in milliseconds
    pub encounter_id: u32,
}

#[derive(Serialize)]
pub struct DeathDetail {
    pub date: String,
    pub fight_id: u32,
    pub death_order: usize,
    pub out_of: usize,
    pub ability_name: String,
}

#[derive(Serialize)]
pub struct PlayerResult {
    pub name: String,
    pub bad_deaths: usize,
    pub avg_death_order: f64,
    pub early_deaths: usize,
    pub details: Vec<DeathDetail>,
}

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub players: Vec<PlayerResult>,
}

#[derive(Serialize)]
pub struct EncounterInfo {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize)]
pub struct EncountersResponse {
    pub encounters: Vec<EncounterInfo>,
}

pub async fn get_encounters(
    State(state): State<Arc<AppState>>,
) -> Json<EncountersResponse> {
    let encounters = state
        .config
        .encounter
        .iter()
        .map(|e| EncounterInfo { id: e.id, name: e.name.clone() })
        .collect();
    Json(EncountersResponse { encounters })
}

pub async fn post_analyze(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, String> {
    run_analyze(&state, req).await.map(Json).map_err(|e| e.to_string())
}

async fn run_analyze(state: &AppState, req: AnalyzeRequest) -> anyhow::Result<AnalyzeResponse> {
    let encounter_config = state
        .config
        .encounter
        .iter()
        .find(|e| e.id == req.encounter_id)
        .ok_or_else(|| anyhow::anyhow!("Encounter ID {} not found in config", req.encounter_id))?;

    let bad_ability_set: HashSet<u32> = encounter_config.bad_abilities.iter().cloned().collect();
    let ability_names = queries::get_ability_names(&state.wcl, &encounter_config.bad_abilities).await?;

    let reports = queries::get_report_codes_for_guild(
        &state.wcl,
        &req.guild_name,
        &req.guild_server_slug,
        &req.guild_server_region,
        req.start_time,
        req.end_time,
    ).await?;

    println!("Found {} report(s): {:?}", reports.len(), reports.iter().map(|(c, _)| c).collect::<Vec<_>>());

    // player_name -> Vec<(date, fight_id, death_order, out_of, ability_id)>
    let mut bad_deaths: HashMap<String, Vec<(String, u32, usize, usize, u32)>> = HashMap::new();
    // player_name -> Vec<death_order> across all deaths for avg
    let mut all_death_orders: HashMap<String, Vec<usize>> = HashMap::new();
    // all player names seen in relevant fights
    let mut all_players: HashSet<String> = HashSet::new();

    for (code, report_start_ms) in &reports {
        let date = ms_to_pacific_date(*report_start_ms);
        let report = queries::get_report_data(&state.wcl, code).await?;

        let actor_names: HashMap<i32, String> = report
            .actors
            .iter()
            .filter(|(_, a)| a.actor_type == "Player")
            .map(|(id, a)| (*id, a.name.clone()))
            .collect();

        let relevant_fights: Vec<&crate::model::Fight> = report
            .fights
            .iter()
            .filter(|f| f.is_real_encounter() && f.encounter_id == req.encounter_id)
            .collect();

        for fight in &relevant_fights {
            for id in &fight.friendly_players {
                if let Some(name) = actor_names.get(id) {
                    all_players.insert(name.clone());
                }
            }
        }

        for fight in relevant_fights {
            let deaths = queries::get_deaths(&state.wcl, code, fight.id, fight.start_time, fight.end_time).await?;

            let mut sorted_deaths: Vec<&crate::model::DeathEvent> = deaths
                .iter()
                .filter(|d| fight.friendly_players.contains(&d.target_id))
                .collect();

            sorted_deaths.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

            let out_of = fight.friendly_players.len();

            for (index, death) in sorted_deaths.iter().enumerate() {
                if let Some(name) = actor_names.get(&death.target_id) {
                    all_death_orders.entry(name.clone()).or_default().push(index + 1);

                    if let Some(ability_id) = death.killing_ability_game_id {
                        if bad_ability_set.contains(&ability_id) {
                            bad_deaths
                                .entry(name.clone())
                                .or_default()
                                .push((date.clone(), fight.id, index + 1, out_of, ability_id));
                        }
                    }
                }
            }
        }
    }

    let empty = Vec::new();
    let mut all_names: Vec<&String> = all_players.iter().collect();
    all_names.sort();
    // Sort: bad death players first (desc), then clean players alphabetically
    let mut summary: Vec<(&String, &Vec<(String, u32, usize, usize, u32)>)> = all_names
        .into_iter()
        .map(|name| (name, bad_deaths.get(name).unwrap_or(&empty)))
        .collect();
    summary.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));

    let players = summary
        .into_iter()
        .map(|(player, deaths)| {
            let overall_orders = all_death_orders.get(player).map(|v| v.as_slice()).unwrap_or(&[]);
            let avg_death_order = if overall_orders.is_empty() {
                0.0
            } else {
                overall_orders.iter().sum::<usize>() as f64 / overall_orders.len() as f64
            };
            let early_deaths = overall_orders.iter().filter(|&&o| o <= 3).count();

            let details = deaths
                .iter()
                .map(|(date, fight_id, order, out_of, ability_id)| DeathDetail {
                    date: date.clone(),
                    fight_id: *fight_id,
                    death_order: *order,
                    out_of: *out_of,
                    ability_name: ability_names
                        .get(ability_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Unknown({})", ability_id)),
                })
                .collect();

            PlayerResult {
                name: player.clone(),
                bad_deaths: deaths.len(),
                avg_death_order,
                early_deaths,
                details,
            }
        })
        .collect();

    Ok(AnalyzeResponse { players })
}
