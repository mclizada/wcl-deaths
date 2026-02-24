use crate::client::WclClient;
use crate::model::{Actor, DeathEvent, Fight};
use serde_json::json;
use std::collections::HashMap;

pub struct ReportData {
    pub fights: Vec<Fight>,
    pub actors: HashMap<i32, Actor>,
}

pub async fn get_report_data(client: &WclClient, code: &str) -> anyhow::Result<ReportData> {
    let query = r#"
        query GetReport($code: String!) {
          reportData {
            report(code: $code) {
              masterData {
                actors {
                  id
                  name
                  type
                  subType
                }
              }
              fights {
                id
                name
                encounterID
                startTime
                endTime
                kill
                difficulty
                friendlyPlayers
              }
            }
          }
        }
    "#;

    let data = client.query(query, json!({ "code": code })).await?;
    let report = &data["reportData"]["report"];

    let actors: Vec<Actor> = serde_json::from_value(
        report["masterData"]["actors"].clone()
    )?;

    let actor_map = actors.into_iter().map(|a| (a.id, a)).collect();

    let fights: Vec<Fight> = serde_json::from_value(
        report["fights"].clone()
    )?;

    Ok(ReportData { fights, actors: actor_map })
}

pub async fn get_deaths(
    client: &WclClient,
    code: &str,
    fight_id: u32,
    start_time: f64,
    end_time: f64,
) -> anyhow::Result<Vec<DeathEvent>> {
    let query = r#"
        query GetDeaths($code: String!, $fightId: Int!, $startTime: Float!, $endTime: Float!) {
          reportData {
            report(code: $code) {
              events(
                fightIDs: [$fightId]
                startTime: $startTime
                endTime: $endTime
                dataType: Deaths
              ) {
                data
                nextPageTimestamp
              }
            }
          }
        }
    "#;

    let mut all_events: Vec<DeathEvent> = Vec::new();
    let mut current_start = start_time;

    loop {
        let data = client.query(query, json!({
            "code": code,
            "fightId": fight_id,
            "startTime": current_start,
            "endTime": end_time,
        })).await?;

        let events_node = &data["reportData"]["report"]["events"];

        if let Some(events) = events_node["data"].as_array() {
            let page: Vec<DeathEvent> = serde_json::from_value(
                serde_json::Value::Array(events.clone())
            )?;
            all_events.extend(page);
        }

        match events_node["nextPageTimestamp"].as_f64() {
            Some(next) => current_start = next,
            None => break,
        }
    }

    Ok(all_events)
}

pub async fn get_report_codes_for_guild(
    client: &WclClient,
    guild_name: &str,
    guild_server_slug: &str,
    guild_server_region: &str,
    start_time: f64,
    end_time: f64,
) -> anyhow::Result<Vec<(String, f64)>> {
    let query = r#"
        query GetGuildReports(
            $guildName: String!,
            $guildServerSlug: String!,
            $guildServerRegion: String!,
            $startTime: Float!,
            $endTime: Float!
        ) {
          reportData {
            reports(
              guildName: $guildName
              guildServerSlug: $guildServerSlug
              guildServerRegion: $guildServerRegion
              startTime: $startTime
              endTime: $endTime
            ) {
              data {
                code
                startTime
              }
            }
          }
        }
    "#;

    let data = client.query(query, json!({
        "guildName": guild_name,
        "guildServerSlug": guild_server_slug,
        "guildServerRegion": guild_server_region,
        "startTime": start_time,
        "endTime": end_time,
    })).await?;

    let mut reports = Vec::new();
    if let Some(arr) = data["reportData"]["reports"]["data"].as_array() {
        for report in arr {
            if let (Some(code), Some(start)) = (report["code"].as_str(), report["startTime"].as_f64()) {
                reports.push((code.to_string(), start));
            }
        }
    }

    Ok(reports)
}

pub async fn get_ability_names(
    client: &WclClient,
    ability_ids: &[u32],
) -> anyhow::Result<HashMap<u32, String>> {
    if ability_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let fields: String = ability_ids
        .iter()
        .map(|id| format!("a{}: ability(id: {}) {{ name }}", id, id))
        .collect::<Vec<_>>()
        .join("\n");

    let query = format!("{{ gameData {{ {} }} }}", fields);

    let data = client.query(&query, json!({})).await?;
    let game_data = &data["gameData"];

    let mut map = HashMap::new();
    for id in ability_ids {
        let key = format!("a{}", id);
        if let Some(name) = game_data[&key]["name"].as_str() {
            map.insert(*id, name.to_string());
        }
    }

    Ok(map)
}