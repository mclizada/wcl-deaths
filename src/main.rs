mod client;
mod config;
mod model;
mod queries;

use std::collections::{HashMap, HashSet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let client_id = std::env::var("WCL_CLIENT_ID")?;
    let client_secret = std::env::var("WCL_CLIENT_SECRET")?;

    let config = config::load_config("config.toml")?;
    let wcl = client::WclClient::new(&client_id, &client_secret).await?;

    // Hardcoded for now, will make configurable later
    let report_codes = vec![""];
    let target_player = "";
    let encounter_id = ;

    let encounter_config = config
        .encounter
        .iter()
        .find(|e| e.id == encounter_id)
        .expect("Encounter not found in config");

    let bad_ability_set: HashSet<u32> = encounter_config.bad_abilities.iter().cloned().collect();

    // Collect all bad ability IDs upfront so we can resolve names in one batch
    let ability_names = queries::get_ability_names(&wcl, &encounter_config.bad_abilities).await?;

    let mut bad_deaths: Vec<(String, usize, usize, u32, f64)> = Vec::new(); // (report, order, out_of, ability_id, timestamp)

    for code in &report_codes {
        println!("Processing report: {}", code);

        let report = queries::get_report_data(&wcl, code).await?;

        // Find this player's actor ID in this report
        let player_actor = report
            .actors
            .values()
            .find(|a| a.name == target_player && a.actor_type == "Player");

        let player_id = match player_actor {
            Some(a) => a.id,
            None => {
                println!("  Player {} not found in report {}", target_player, code);
                continue;
            }
        };

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
            let mut player_deaths: Vec<&model::DeathEvent> = deaths
                .iter()
                .filter(|d| fight.friendly_players.contains(&d.target_id))
                .collect();

            player_deaths.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

            // Find this player's death
            let player_death = player_deaths
                .iter()
                .enumerate()
                .find(|(_, d)| d.target_id == player_id);

            if let Some((index, death)) = player_death {
                let order = index + 1;
                let out_of = fight.friendly_players.len();
                let ability_id = death.killing_ability_game_id;

                if bad_ability_set.contains(&ability_id) {
                    let ability_name = ability_names
                        .get(&ability_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Unknown({})", ability_id));

                    println!(
                        "  Fight {:>2} — died {}/{} to {}",
                        fight.id, order, out_of, ability_name
                    );

                    bad_deaths.push((code.to_string(), order, out_of, ability_id, death.timestamp));
                }
            }
        }
    }

    // Summary
    if bad_deaths.is_empty() {
        println!("\nNo bad deaths found for {} on this encounter.", target_player);
    } else {
        let avg = bad_deaths.iter().map(|(_, o, _, _, _)| *o as f64).sum::<f64>()
            / bad_deaths.len() as f64;

        println!("\n--- Summary for {} ---", target_player);
        println!("Bad deaths: {}", bad_deaths.len());
        println!("Avg death order: {:.1}", avg);
    }

    Ok(())
}