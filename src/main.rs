use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use sqlx::PgPool;

use conf::AppConfig;

mod conf;
mod ws;
mod listener_carbon;
mod listener_evm;
mod db;
mod broadcaster_evm;

#[derive(Parser)]
#[command(author = "Switcheo Labs Pte. Ltd.", name = "Carbon Axelar Relayer", version, about = "Carbon Axelar Relayer", long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run
    Run,
    // Add { name: Option<String>, lol: Option<String> },
    // Run
    // #[command(subcommand)]
    // query_command: Option<QueryCommands>,
}


#[derive(Subcommand)]
enum QueryCommands {
    /// does testing things
    Pending {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let conf = AppConfig::new(cli.config)?;
    let pg_pool = PgPool::connect(&conf.database.pg_url)
        .await
        .expect("Failed to create pg pool.");
    let pg_pool = Arc::new(pg_pool);

    match &cli.command {
        Some(Commands::Run) => {
            // Spawn carbon_listener init_ws as a concurrent task
            let carbon_pg_pool = pg_pool.clone();
            let carbon_task = tokio::spawn(async move {
                listener_carbon::init_ws(&conf.carbon_ws_url, &conf.relayer_deposit_address, carbon_pg_pool).await;
            });

            // Spawn evm_listener init_ws as a concurrent task
            let evm_pg_pool = pg_pool.clone();
            let evm_chains = conf.evm_chains.clone();
            let evm_task = tokio::spawn(async move {
                listener_evm::init_all_ws(evm_chains, evm_pg_pool).await;
            });

            // Spawn broadcaster_evm init as a concurrent task
            let broadcaster_evm_pg_pool = pg_pool.clone();
            let evm_chains = conf.evm_chains.clone();
            let evm_task = tokio::spawn(async move {
                broadcaster_evm::init_all(evm_chains, broadcaster_evm_pg_pool).await;
            });

            // Wait for all spawned tasks to complete
            let _ = tokio::join!(carbon_task, evm_task);
        }

        None => {}
    }

    Ok(())
}
