mod client;
mod config;
mod model;
mod queries;

use clap::Parser;
use std::collections::{HashMap, HashSet};

#[derive(Parser)]
#[command(
    about = "Analyze bad deaths on a WarcraftLogs encounter",
    long_about = None,
)]
struct Args {
    /// WarcraftLogs report code(s). Can be specified multiple times.
    /// Example: -r AbVphwHqgLJ7ZQ3Y -r GbkAZP4Hwvn68yfL
    #[arg(short, long, required = true, value_name = "CODE")]
    reports: Vec<String>,

    /// Encounter ID to analyze. Must be present in config.toml.
    /// Example: --encounter 3134 (Nexus-King Salhadaar)
    #[arg(short, long, default_value_t = 3134, value_name = "ID")]
    encounter: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let client_id = std::env::var("WCL_CLIENT_ID")?;
    let client_secret = std::env::var("WCL_CLIENT_SECRET")?;

    let args = Args::parse();

    let config = config::load_config("config.toml")?;
    let wcl = client::WclClient::new(&client_id, &client_secret).await?;

    let report_codes = args.reports;
    let encounter_id = args.encounter;

    let encounter_config = config
        .encounter
        .iter()
        .find(|e| e.id == encounter_id)
        .ok_or_else(|| anyhow::anyhow!(
            "Encounter ID {} not found in config.toml. Add a [[encounter]] block with id = {}.",
            encounter_id, encounter_id
        ))?;

    let bad_ability_set: HashSet<u32> = encounter_config.bad_abilities.iter().cloned().collect();

    // Collect all bad ability IDs upfront so we can resolve names in one batch
    let ability_names = queries::get_ability_names(&wcl, &encounter_config.bad_abilities).await?;

    // player_name -> Vec<(fight_id, death_order, out_of, ability_id)>
    let mut bad_deaths: HashMap<String, Vec<(u32, usize, usize, u32)>> = HashMap::new();
    // player_name -> Vec<death_order> across all deaths (bad or not), for overall avg
    let mut all_death_orders: HashMap<String, Vec<usize>> = HashMap::new();

    for code in &report_codes {
        println!("Processing report: {}", code);

        let report = queries::get_report_data(&wcl, code).await?;

        // Build actor_id -> name map for players in this report
        let actor_names: HashMap<i32, String> = report
            .actors
            .iter()
            .filter(|(_, a)| a.actor_type == "Player")
            .map(|(id, a)| (*id, a.name.clone()))
            .collect();

        // Filter to real pulls of the target encounter
        let relevant_fights: Vec<&model::Fight> = report
            .fights
            .iter()
            .filter(|f| f.is_real_encounter() && f.encounter_id == encounter_id)
            .collect();

        println!("  Found {} pulls of encounter {}", relevant_fights.len(), encounter_id);

        for fight in relevant_fights {
            let deaths = queries::get_deaths(&wcl, code, fight.id, fight.start_time, fight.end_time).await?;

            // Only player deaths, sorted by timestamp
            let mut sorted_deaths: Vec<&model::DeathEvent> = deaths
                .iter()
                .filter(|d| fight.friendly_players.contains(&d.target_id))
                .collect();

            sorted_deaths.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

            let out_of = fight.friendly_players.len();

            for (index, death) in sorted_deaths.iter().enumerate() {
                if let Some(name) = actor_names.get(&death.target_id) {
                    // Track all deaths for overall avg order
                    all_death_orders.entry(name.clone()).or_default().push(index + 1);

                    // Track bad deaths separately
                    if let Some(ability_id) = death.killing_ability_game_id {
                        if bad_ability_set.contains(&ability_id) {
                            bad_deaths
                                .entry(name.clone())
                                .or_default()
                                .push((fight.id, index + 1, out_of, ability_id));
                        }
                    }
                }
            }
        }
    }

    // Summary: sort by bad death count descending
    let mut summary: Vec<(&String, &Vec<(u32, usize, usize, u32)>)> = bad_deaths.iter().collect();
    summary.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    if summary.is_empty() {
        println!("\nNo bad deaths found.");
    } else {
        println!("\n--- Bad Deaths Summary ---");
        for (player, deaths) in &summary {
            let overall_orders = all_death_orders.get(*player).map(|v| v.as_slice()).unwrap_or(&[]);
            let avg_order = if overall_orders.is_empty() {
                0.0
            } else {
                overall_orders.iter().sum::<usize>() as f64 / overall_orders.len() as f64
            };
            let early_deaths = overall_orders.iter().filter(|&&o| o <= 3).count();
            println!("{}: {} bad death(s), avg death order {:.1}, top-3 deaths: {}", player, deaths.len(), avg_order, early_deaths);
            for (fight_id, order, out_of, ability_id) in *deaths {
                let ability_name = ability_names
                    .get(ability_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Unknown({})", ability_id));
                println!("  Fight {:>2} — died {}/{} to {}", fight_id, order, out_of, ability_name);
            }
        }
    }

    Ok(())
}
