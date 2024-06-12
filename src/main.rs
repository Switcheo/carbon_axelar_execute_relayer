use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use sqlx::PgPool;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use conf::AppConfig;
use crate::util::carbon_tx;

mod conf;
mod ws;
mod db;
mod constants;
mod util;
mod operational;
mod carbon;
mod evm;
mod fee;

mod switcheo {
    pub mod carbon {
        pub mod bridge {
            include!("../proto/gen/Switcheo.carbon.bridge.rs");
        }
    }
}

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
    /// Start relay on Carbon for a nonce
    StartRelay {
        /// nonce to start relay
        #[arg(value_name = "NONCE")]
        nonce: u64,
    },
    /// Expire pending actions for multiple nonces
    ExpirePendingActions {
        /// nonce to start relay
        #[arg(value_name = "NONCES", num_args = 1.., value_delimiter=',')]
        nonces: Vec<u64>,
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
            // initialize broadcaster_carbon
            let carbon_broadcaster = carbon::broadcaster::init_all(&conf.carbon).await;

            // Spawn listener_carbon::init_ws as a concurrent task
            let carbon_pg_pool = pg_pool.clone();
            let carbon_config = conf.carbon.clone();
            let fee_config = conf.fee.clone();
            let carbon_broadcaster_clone = carbon_broadcaster.clone();
            let carbon_listen_task = tokio::spawn(async move {
                carbon::listener::init_ws(&carbon_config, &fee_config, carbon_pg_pool, carbon_broadcaster_clone).await;
            });

            // Spawn retry_carbon::init_all as a concurrent task
            let carbon_pg_pool = pg_pool.clone();
            let carbon_config = conf.carbon.clone();
            let fee_config = conf.fee.clone();
            let carbon_broadcaster_clone = carbon_broadcaster.clone();
            let carbon_retry_task = tokio::spawn(async move {
                carbon::retry::init_all(&carbon_config, &fee_config, carbon_pg_pool, carbon_broadcaster_clone).await;
            });

            // Spawn listener_evm::init_all_ws as a concurrent task
            let evm_pg_pool = pg_pool.clone();
            let evm_chains = conf.evm_chains.clone();
            let evm_listen_all_task = tokio::spawn(async move {
                evm::listener::init_all_ws(evm_chains, evm_pg_pool).await;
            });

            // Spawn broadcaster_evm::init_all as a concurrent task
            let broadcaster_evm_pg_pool = pg_pool.clone();
            let evm_chains = conf.evm_chains.clone();
            let evm_execute_task = tokio::spawn(async move {
                evm::broadcaster::init_all(evm_chains, broadcaster_evm_pg_pool).await;
            });

            // Wait for all spawned tasks to complete
            let _ = tokio::join!(carbon_listen_task, carbon_retry_task, evm_listen_all_task, evm_execute_task);
        },
        Some(Commands::Sync { tx_hash }) => {
            // Call a function to handle the sync logic for a specific transaction hash
            // TODO: implement
            info!("NYI, input: {}", tx_hash);
        },
        Some(Commands::SyncFrom { start_height, end_height }) => {
            // Call a function to handle the sync logic for a range of block heights
            operational::tx_sync::sync_block_range(conf.clone(), pg_pool.clone(), *start_height, *end_height).await?;
        }
        Some(Commands::StartRelay { nonce }) => {
            // Call a function to handle the starting the relay
            let _ = carbon_tx::send_msg_start_relay(&conf.carbon.clone(), *nonce).await;
        }
        Some(Commands::ExpirePendingActions { nonces }) => {
            // Call a function to handle the starting the relay
            let _ = operational::expire::expire_pending_actions(&conf.carbon.clone(), nonces.clone()).await;
        }
        None => {}
    }

    Ok(())
}
