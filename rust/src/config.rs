use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};

#[frb]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub is_bootstrap_node: bool,
    pub relay_address: String,
    pub bootstrap_nodes: Vec<String>,
    pub listen_port: u16,
}