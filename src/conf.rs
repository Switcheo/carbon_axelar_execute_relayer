use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    #[serde(default)]
    pub debug: bool, // defaults to false with serde(default)
    pub database: Database,
    pub carbon_ws_url: String,
    pub relayer_deposit_address: String,
    #[serde(rename = "evm_chain")]
    pub evm_chains: Vec<ChainConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChainConfig {
    pub name: String,
    pub rpc_url: String,
    pub ws_url: String,
    pub axelar_gateway_proxy: String,
    pub carbon_axelar_gateway: String,
    pub relayer_private_key: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub pg_url: String,
}

impl AppConfig {
    pub fn new(config_path: PathBuf) -> Result<Self, ConfigError> {
        // Load environment variables from .env file
        dotenv().ok();

        // Use your config
        let c = Config::builder()
            // Add in `./Settings.toml`
            .add_source(File::from(config_path))
            // Add in settings from the environment (with a prefix of CAE)
            // Eg.. `CAE_DATABASE_URL=postgres://...` would set the `database_url` key
            .add_source(Environment::with_prefix("CAE"))
            .build()?;

        // Now that we're done, let's access our configuration
        // println!("debug: {:?}", c.get_bool("debug"));
        println!("database: {:?}", c.get::<String>("database.pg_url"));

        // You can deserialize (and thus freeze) the entire configuration as
        c.try_deserialize()
    }
}