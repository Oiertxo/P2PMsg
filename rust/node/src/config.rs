use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub is_bootstrap_node: bool,
    pub relay_address: String,
    pub bootstrap_nodes: Vec<String>,
    pub listen_port: u16,
}
