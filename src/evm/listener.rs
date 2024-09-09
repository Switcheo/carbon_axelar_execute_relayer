use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use ethers::{
    contract::EthEvent,
    core::types::{Address, Filter, H256},
    prelude::*,
    providers::{Provider},
};
use ethers::abi::RawLog;
use ethers::utils::keccak256;
use sqlx::PgPool;
use tracing::{debug, error, info, instrument};

use crate::conf::Chain;
use crate::constants::events::EVM_CONTRACT_CALL_APPROVED_EVENT;
use crate::db::evm_events::save_call_contract_approved_event;
use crate::util::evm::ContractCallApprovedEvent;

#[instrument(name = "listener_evm", skip_all)]
pub async fn init_all(evm_chains: Vec<Chain>, pg_pool: Arc<PgPool>) {
    for chain in evm_chains {
        let pg_pool_clone = pg_pool.clone();
        let chain_clone = chain.clone();
        info!("Subscribing to {}, hasWS: {}", &chain.chain_id, &chain.has_ws);
        tokio::spawn(async move {
            if chain_clone.has_ws {
                if let Err(e) = init_ws(chain_clone.clone(), pg_pool_clone).await {
                    error!("Error initializing WebSocket for {}: {}", &chain_clone.ws_url, e);
                }
            } else {
                if let Err(e) = init_http(chain_clone.clone(), pg_pool_clone).await {
                    error!("Error initializing Http client for {}: {}", &chain_clone.rpc_url, e);
                }
            }
        });
        let pg_pool_clone = pg_pool.clone();
        let chain_clone = chain.clone();
        info!("Initializing http backfiller for {}", &chain_clone.chain_id);
        tokio::spawn(async move {
            if let Err(e) = init_backfiller(chain_clone.clone(), pg_pool_clone).await {
                error!("Error initializing Http client for {}: {}", &chain_clone.rpc_url, e);
            }
        });
    }
}

// init_ws connect to the evm network via WebSocket and watch for relevant events
#[instrument(name = "listener_evm_ws", skip_all, fields(chain = chain_config.chain_id))]
async fn init_ws(chain_config: Chain, pg_pool: Arc<PgPool>) -> Result<()> {
    // Connect to the evm node
    let provider = Provider::<Ws>::connect_with_reconnects(&chain_config.ws_url, 1000).await
        .context("Failed to connect to WS")?;


    let provider = Arc::new(provider);

    info!("Connected to {:?}", &chain_config.ws_url);

    let address = chain_config.axelar_gateway_proxy.parse::<Address>()?;
    let address = ValueOrArray::Value(address);
    // filter for contract_address (2nd indexed topic)
    let topic2 = H256::from(chain_config.carbon_axelar_gateway.clone().parse::<Address>()?);

    let event = ContractCallApprovedEvent::new::<_, Provider<Ws>>(
        Filter::new().address(address).topic2(topic2),
        Arc::clone(&provider),
    );
    let mut events = event.subscribe().await?;

    info!("Starting to watch {:?} {:?} for {:?} filtered by carbon contract: {:?}", &chain_config.chain_id, &chain_config.axelar_gateway_proxy, EVM_CONTRACT_CALL_APPROVED_EVENT, &chain_config.carbon_axelar_gateway);
    while let Some(log) = events.next().await {
        info!("found an event on {:?} {:?} for {:?} filtered by carbon contract: {:?}", &chain_config.chain_id, &chain_config.axelar_gateway_proxy, EVM_CONTRACT_CALL_APPROVED_EVENT, &chain_config.carbon_axelar_gateway);
        match log {
            Ok(event) => {
                let chain_config = chain_config.clone();
                let pg_pool = pg_pool.clone();
                let _ = tokio::spawn(async move {
                    info!("Received ContractCallApprovedEvent for carbon_axelar_gateway ({:?}): {:?}", &chain_config.carbon_axelar_gateway, event);
                    save_call_contract_approved_event(chain_config.clone(), pg_pool.clone(), event).await;
                });
            }
            Err(e) => error!("Error listening for ContractCallApprovedEvent logs: {:?}", e),
        }
    }

    Ok(())
}


// init_ws connect to the evm network via WebSocket and watch for relevant events
#[instrument(name = "listener_evm_http", skip_all, fields(chain = chain_config.chain_id))]
async fn init_http(chain_config: Chain, pg_pool: Arc<PgPool>) -> Result<()> {
    // Connect to the evm node
    let provider = Provider::<Http>::connect(&chain_config.rpc_url).await;

    let provider = Arc::new(provider);

    info!("Connected to {:?}", &chain_config.rpc_url);

    let address = chain_config.axelar_gateway_proxy.parse::<Address>()?;
    let address = ValueOrArray::Value(address);
    // filter for contract_address (2nd indexed topic)
    let topic2 = H256::from(chain_config.carbon_axelar_gateway.clone().parse::<Address>()?);

    let event = ContractCallApprovedEvent::new::<_, Provider<Http>>(
        Filter::new().address(address).topic2(topic2),
        Arc::clone(&provider),
    );
    let mut events = event.stream().await?;

    info!("Starting to watch {:?} {:?} for {:?} filtered by carbon contract: {:?}", &chain_config.chain_id, &chain_config.axelar_gateway_proxy, EVM_CONTRACT_CALL_APPROVED_EVENT, &chain_config.carbon_axelar_gateway);
    while let Some(log) = events.next().await {
        info!("found an event on {:?} {:?} for {:?} filtered by carbon contract: {:?}", &chain_config.chain_id, &chain_config.axelar_gateway_proxy, EVM_CONTRACT_CALL_APPROVED_EVENT, &chain_config.carbon_axelar_gateway);
        match log {
            Ok(event) => {
                let chain_config = chain_config.clone();
                let pg_pool = pg_pool.clone();
                let _ = tokio::spawn(async move {
                    info!("Received ContractCallApprovedEvent for carbon_axelar_gateway ({:?}): {:?}", &chain_config.carbon_axelar_gateway, event);
                    save_call_contract_approved_event(chain_config.clone(), pg_pool.clone(), event).await;
                });
            }
            Err(e) => error!("Error listening for ContractCallApprovedEvent logs: {:?}", e),
        }
    }

    Ok(())
}


// init_backfiller connect to the evm network via http and backfill events that were missed
#[instrument(name = "listener_evm_backfiller", skip_all, fields(chain = chain_config.chain_id))]
async fn init_backfiller(chain_config: Chain, pg_pool: Arc<PgPool>) -> Result<()> {
    // Connect to the evm node
    let provider = Provider::<Http>::connect(&chain_config.rpc_url).await;

    let provider = Arc::new(provider);

    info!("Connected to {:?} for backfilling", &chain_config.rpc_url);

    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    let chain_config_clone = chain_config.clone();
    let pool = pg_pool.clone();
    loop {
        debug!("Backfilling from {}", &chain_config_clone.rpc_url);
        if let Err(e) = backfill(chain_config_clone.clone(), provider.clone(), pool.clone()).await {
            error!("Failed to backfill from {}: {}", chain_config_clone.rpc_url, e);
        }
        interval.tick().await;
    }
}

// backfill will attempt to search the last `max_query_blocks` blocks for any ContractCallApprovedEvent that emitted by `carbon_axelar_gateway`
// This is so that any missed events will be recorded
async fn backfill(chain_config: Chain, provider: Arc<Provider<Http>>, pg_pool: Arc<PgPool>) -> Result<()> {
    // filter for axelar's gateway
    let address = chain_config.axelar_gateway_proxy.parse::<Address>()?;
    let address = ValueOrArray::Value(address);

    // filter for event signature: ContractCallApprovedEvent
    let event_signature = "ContractCallApproved(bytes32,string,string,address,bytes32,bytes32,uint256)";
    let topic0 = H256::from(keccak256(event_signature.as_bytes()));  // Hash the event signature to get `topic0`

    // filter for contract_address (2nd indexed topic)
    let topic2 = H256::from(chain_config.carbon_axelar_gateway.clone().parse::<Address>()?);

    // Get the latest block number
    let latest_block = provider.get_block_number().await?;

    // Set the block range to query the last `max_query_blocks` blocks
    let from_block = latest_block.saturating_sub(chain_config.max_query_blocks.into());
    let to_block = latest_block;

    // Build the filter to query logs in the block range
    let filter = Filter::new()
        .address(address)
        .topic0(topic0)
        .topic2(topic2)
        .from_block(from_block)
        .to_block(to_block);

    // Query logs from the provider using the filter
    let logs = provider.get_logs(&filter).await?;

    // Process each log
    for log in logs {
        let chain_clone = chain_config.clone();
        let pool_clone = pg_pool.clone();
        if let Ok(event) = <ContractCallApprovedEvent as EthEvent>::decode_log(&RawLog::from(log)) {
            save_call_contract_approved_event(chain_clone, pool_clone, event).await;
        } else {
            error!("Failed to decode log");
        }
    }
    Ok(())
}

