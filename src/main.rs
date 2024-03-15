use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use sqlx::PgPool;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use conf::AppConfig;

mod conf;
mod ws;
mod listener_carbon;
mod listener_evm;
mod db;
mod broadcaster_evm;
mod tx_sync;

#[derive(Parser)]
#[command(author = "Switcheo Labs Pte. Ltd.", name = "Carbon-Axelar Relayer", version, about = "Carbon-Axelar Relayer", long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    config: PathBuf,

    /// Sets the level of verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run
    Run,
    /// Sync a specific transaction
    Sync {
        /// Transaction hash to resync
        #[arg(value_name = "TX_HASH")]
        tx_hash: String,
    },
    /// Sync from a specific block range
    SyncFrom {
        /// Start block height for resync
        #[arg(value_name = "START_HEIGHT")]
        start_height: u64,
        /// End block height for resync
        #[arg(value_name = "END_HEIGHT")]
        end_height: u64,
    },
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
    // Initialize cli
    let cli = Cli::parse();

    // Initialize logger
    let log_level = match cli.verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Initialize config
    let conf = AppConfig::new(cli.config)?;
    let pg_pool = PgPool::connect(&conf.database.pg_url)
        .await
        .expect("Failed to create pg pool.");
    let pg_pool = Arc::new(pg_pool);

    // Run commands based on user input
    match &cli.command {
        Some(Commands::Run) => {
            // Spawn listener_carbon::init_ws as a concurrent task
            let carbon_pg_pool = pg_pool.clone();
            let carbon_listen_task = tokio::spawn(async move {
                listener_carbon::init_ws(&conf.carbon, carbon_pg_pool).await;
            });

            // Spawn listener_evm::init_all_ws as a concurrent task
            let evm_pg_pool = pg_pool.clone();
            let evm_chains = conf.evm_chains.clone();
            let evm_listen_all_task = tokio::spawn(async move {
                listener_evm::init_all_ws(evm_chains, evm_pg_pool).await;
            });

            // Spawn broadcaster_evm::init_all as a concurrent task
            let broadcaster_evm_pg_pool = pg_pool.clone();
            let evm_chains = conf.evm_chains.clone();
            let evm_execute_task = tokio::spawn(async move {
                broadcaster_evm::init_all(evm_chains, broadcaster_evm_pg_pool).await;
            });

            // Wait for all spawned tasks to complete
            let _ = tokio::join!(carbon_listen_task, evm_listen_all_task, evm_execute_task);
        },
        Some(Commands::Sync { tx_hash }) => {
            // Call a function to handle the sync logic for a specific transaction hash
            // TODO: implement
            info!("NYI, input: {}", tx_hash);
        },
        Some(Commands::SyncFrom { start_height, end_height }) => {
            // Call a function to handle the sync logic for a range of block heights
            tx_sync::sync_block_range(conf.clone(), pg_pool.clone(), *start_height, *end_height).await?;
        }
        None => {}
    }

    Ok(())
}
