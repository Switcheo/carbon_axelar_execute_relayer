use std::sync::Arc;

use anyhow::{Context, Result};
use ethers::{
    contract::EthEvent,
    core::types::{Address, Filter, H256},
    prelude::*,
    providers::{Provider},
};
use sqlx::PgPool;
use tracing::{error, info, instrument};

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
            if chain.has_ws {
                if let Err(e) = init_ws(chain_clone, pg_pool_clone).await {
                    error!("Error initializing WebSocket for {}: {}", &chain.ws_url, e);
                }
            } else {
                if let Err(e) = init_http(chain_clone, pg_pool_clone).await {
                    error!("Error initializing Http client for {}: {}", &chain.rpc_url, e);
                }
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
                    save_call_contract_approved_event(chain_config.clone(), pg_pool.clone(), event).await;
                });
            }
            Err(e) => error!("Error listening for ContractCallApprovedEvent logs: {:?}", e),
        }
    }

    Ok(())
}

