use crate::client::WclClient;
use crate::config::Config;

pub struct AppState {
    pub wcl: WclClient,
    pub config: Config,
}
