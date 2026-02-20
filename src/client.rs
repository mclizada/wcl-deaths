use anyhow::bail;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct WclClient {
    pub http: Client,
    pub token: String,
}

#[derive(Serialize)]
struct GraphQLRequest<'a> {
    query: &'a str,
    variables: Value,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

impl WclClient {
    pub async fn new(client_id: &str, client_secret: &str) -> anyhow::Result<Self> {
        let http = Client::new();

        let res: TokenResponse = http
            .post("https://www.warcraftlogs.com/oauth/token")
            .basic_auth(client_id, Some(client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?
            .json()
            .await?;

        Ok(Self {
            http,
            token: res.access_token,
        })
    }

    pub async fn query(&self, query: &str, variables: Value) -> anyhow::Result<Value> {
        let res = self
            .http
            .post("https://www.warcraftlogs.com/api/v2/client")
            .bearer_auth(&self.token)
            .json(&GraphQLRequest { query, variables })
            .send()
            .await?
            .json::<Value>()
            .await?;

        if let Some(errors) = res.get("errors") {
            bail!("GraphQL errors: {}", errors);
        }

        Ok(res["data"].clone())
    }
}