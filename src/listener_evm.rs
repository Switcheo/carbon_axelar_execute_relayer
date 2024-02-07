use std::sync::Arc;

use ethers::{
    contract::EthEvent,
    core::types::{Address, Filter, H160, H256, U256},
    prelude::*,
    providers::{Provider, Ws},
};
use sqlx::PgPool;
use std::str::FromStr;
use sqlx::types::BigDecimal;

use crate::conf::ChainConfig;

#[derive(Debug, Clone, PartialEq, Eq, Default, EthEvent)]
#[ethevent(name = "ContractCallApproved", abi = "ContractCallApproved(bytes32,string,string,address,bytes32,bytes32,uint256)")]
pub struct ContractCallApprovedEvent {
    #[ethevent(indexed)]
    pub command_id: H256,
    pub source_chain: String,
    pub source_address: String,
    #[ethevent(indexed)]
    pub contract_address: Address,
    #[ethevent(indexed)]
    pub payload_hash: H256,
    pub source_tx_hash: H256,
    pub source_event_index: U256,
}

pub async fn init_all_ws(evm_chains: Vec<ChainConfig>, pg_pool: Arc<PgPool>) {
    for chain in evm_chains {
        let pg_pool_clone = pg_pool.clone();
        println!("Subscribing to {} on {}", &chain.name, &chain.ws_url);
        tokio::spawn(async move {
            if let Err(e) = init_ws(&chain.ws_url.clone(), &chain.axelar_gateway_proxy.clone(), &chain.carbon_axelar_gateway.clone(), pg_pool_clone).await {
                eprintln!("Error initializing WebSocket for {}: {}", &chain.ws_url, e);
            }
        });
    }
}

// init_ws connect to the evm network via WebSocket and watch for relevant events
async fn init_ws(ws_url: &String, axelar_gateway_proxy: &str, carbon_axelar_gateway: &str, pg_pool: Arc<PgPool>) -> Result<(), Box<dyn std::error::Error>> {

    // Connect to the Ethereum node
    let provider = Provider::<Ws>::connect_with_reconnects(ws_url, 100).await?;
    let provider = Arc::new(provider);

    let address = axelar_gateway_proxy.parse::<Address>()?;
    let address = ValueOrArray::Value(address);
    // filter for contract_address (2nd indexed topic)
    let topic2 = H256::from(carbon_axelar_gateway.parse::<H160>()?);
    let event = ContractCallApprovedEvent::new::<_, Provider<Ws>>(Filter::new().topic2(topic2), Arc::clone(&provider)).address(address);
    let mut events = event.subscribe().await?.take(5);

    while let Some(log) = events.next().await {
        match log {
            Ok(event) => {
                println!("ContractCallApprovedEvent: {:?}", event);
                // Process the event data as needed
                // save to db
                sqlx::query!(
                    "INSERT INTO contract_call_approved_events (command_id, source_chain, source_address, contract_address, payload_hash, source_tx_hash, source_event_index) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    format!("{:?}", event.command_id),
                    event.source_chain,
                    event.source_address,
                    format!("{:?}", event.contract_address),
                    format!("{:?}", event.payload_hash),
                    format!("{:?}", event.source_tx_hash),
                    BigDecimal::from_str(&event.source_event_index.to_string()).unwrap(),
                )
                    .execute(&*pg_pool)
                    .await?;
            },
            Err(e) => println!("Error listening for ContractCallApprovedEvent logs: {:?}", e),
        }
    }

    Ok(())
}
