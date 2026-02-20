mod api;
mod client;
mod config;
mod model;
mod queries;
mod state;

use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let client_id = std::env::var("WCL_CLIENT_ID")?;
    let client_secret = std::env::var("WCL_CLIENT_SECRET")?;

    let config = config::load_config("config.toml")?;
    let wcl = client::WclClient::new(&client_id, &client_secret).await?;

    let state = Arc::new(state::AppState { wcl, config });

    let app = Router::new()
        .route("/api/encounters", get(api::get_encounters))
        .route("/api/analyze", post(api::post_analyze))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
