use std::path::PathBuf;

use config::{Config, ConfigError, File};
use dotenvy::dotenv;
use serde::Deserialize;
use tracing::info;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    #[serde(default)]
    pub debug: bool, // defaults to false with serde(default)
    pub database: Database,
    pub carbon: Carbon,
    #[serde(rename = "evm_chain")]
    pub evm_chains: Vec<Chain>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Carbon {
    pub relay_admin_payloads: bool,
    pub relay_user_payloads: bool,
    pub rpc_url: String,
    pub ws_url: String,
    pub relayer_deposit_address: String,
    pub axelar_bridge_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Chain {
    pub chain_id: String,
    pub rpc_url: String,
    pub ws_url: String,
    pub axelar_gateway_proxy: String,
    pub carbon_axelar_gateway: String,
    pub max_query_blocks: u64,
    pub relayer_private_key: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub pg_url: String,
}

impl AppConfig {
    pub fn new(config_path: PathBuf) -> Result<Self, ConfigError> {
        info!("Initializing AppConfig");
        // Load environment variables from .env file
        dotenv().ok();

        // Use the config file
        let c = Config::builder()
            .add_source(File::from(config_path))
            .build()?;

        // Deserialize (and thus freeze) the entire configuration
        c.try_deserialize()
    }
}