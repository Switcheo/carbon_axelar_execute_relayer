use std::sync::Arc;
use crate::conf::{AppConfig, Chain};
use anyhow::{Context, Result};
use ethers::addressbook::Address;
use ethers::prelude::{EthEvent, Filter, H256, Http, Middleware, Provider, ValueOrArray};
use ethers::utils::hex::{decode, encode_prefixed};
use ethers::utils::keccak256;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, instrument};
use crate::constants::events::{CARBON_AXELAR_CALL_CONTRACT_EVENT, CARBON_BRIDGE_PENDING_ACTION_EVENT};
use crate::db::carbon_events::{save_axelar_call_contract_event, save_bridge_pending_action_event};
use crate::listener_evm::{ContractCallApprovedEvent, save_call_contract_approved_event};
use crate::util::carbon::{parse_axelar_call_contract_event, parse_bridge_pending_action_event};
use crate::util::cosmos::{Event, TxResultInner};
use crate::util::fee::should_relay;
use crate::util::strip_quotes;

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResult {
    id: i32,
    jsonrpc: String,
    result: QueryResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct QueryResult {
    txs: Vec<TxResult>,
    total_count: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxResult {
    pub hash: String,
    pub height: String,
    pub index: u64,
    pub tx_result: TxResultInner,
    pub tx: String,
}

// 1) sync from carbon's start block height to end block height to find relevant txs
// 2) loop through all event's payload_hash and sync evm txs based on the payload_hash found
// 3) save to db, running relayer will continue and broadcast if needed
pub async fn sync_block_range(conf: AppConfig, pg_pool: Arc<PgPool>, start_height: u64, end_height: u64) -> Result<()> {
    info!("Syncing {:?} from blocks {} to {}", &conf.carbon.rpc_url, start_height, end_height);

    // Find and save CARBON_BRIDGE_PENDING_ACTION_EVENT event
    let query = format!("{}.bridge_id CONTAINS '{}' AND tx.height>={} AND tx.height<={}", CARBON_BRIDGE_PENDING_ACTION_EVENT, &conf.carbon.axelar_bridge_id, start_height, end_height);
    let response = abci_query(&conf.carbon.rpc_url, &query).await?;
    info!("Found {} transactions with {}", response.result.total_count, CARBON_BRIDGE_PENDING_ACTION_EVENT);
    // extract all events and save events
    for event in extract_events(response, CARBON_BRIDGE_PENDING_ACTION_EVENT) {
        let bridge_pending_action_event = parse_bridge_pending_action_event(*event.clone());

        // check if relayer should relay (enough fees, etc.)
        if !should_relay(bridge_pending_action_event.get_relay_details()) {
            continue
        }

        save_bridge_pending_action_event(pg_pool.clone(), &bridge_pending_action_event).await;
    }

    let saved_call_contract_events: Vec<>
    // Find and save CARBON_AXELAR_CALL_CONTRACT_EVENT event
    let query = format!("{}.nonce EXISTS AND tx.height>={} AND tx.height<={}", CARBON_AXELAR_CALL_CONTRACT_EVENT, start_height, end_height);
    let response = abci_query(&conf.carbon.rpc_url, &query).await?;
    info!("Found {} transactions with {}", response.result.total_count, CARBON_AXELAR_CALL_CONTRACT_EVENT);
    // extract all events and save events
    for event in extract_events(response, CARBON_AXELAR_CALL_CONTRACT_EVENT) {
        let axelar_call_contract_event = parse_axelar_call_contract_event(event);
        save_axelar_call_contract_event(pg_pool.clone(), *axelar_call_contract_event.clone()).await;
    }

    // Find and save EVM event for each new payload_hash found
    // TODO: can be refactored and optimized to pass in multiple payload_hashes
    for event in payload_events {
        let chain_id = event.attributes.iter().find(|a| a.key == "chain_id").map(|a| a.value.clone()).unwrap_or_default();
        let chain_id = strip_quotes(&chain_id);
        let chain_config = conf.evm_chains.iter().find(|a| a.chain_id == chain_id).unwrap();
        let chain_config = chain_config.clone();
        let payload = event.attributes.iter().find(|a| a.key == "payload").map(|a| a.value.clone()).unwrap_or_default();
        let payload_bytes = decode(strip_quotes(&payload.clone()))
            .expect("Decoding failed");
        let payload_hash = keccak256(&payload_bytes);
        let payload_hash = encode_prefixed(payload_hash);
        // save corresponding evm event
        save_contract_call_approved_events(chain_config, pg_pool.clone(), &payload_hash).await.context("save contract call approved event failed")?;
    }

    Ok(())
}

fn extract_events(response: JsonRpcResult, event_type: &str) -> Vec<Event> {
    response.result.txs.iter()
        .flat_map(|tx| {
            let events: Vec<Event> = tx.tx_result.events.iter().filter(|e| e.event_type == event_type).cloned().collect();
            events
        }
        ).collect()
}

async fn abci_query(carbon_rpc_url: &str, query: &str) -> Result<JsonRpcResult> {

    // URL encode the query
    let encoded_query = urlencoding::encode(query);

    // Construct the URL for the tx_search endpoint with the query
    let query_url = format!(r#"{}/tx_search?query="{}""#, carbon_rpc_url, encoded_query);

    // Perform the GET request
    let client = reqwest::Client::new();
    client.get(&query_url)
        .send()
        .await.context("abci request send failed")?
        .json::<JsonRpcResult>() // Deserialize the JSON response into a serde_json::Value
        .await.context("json deserializing failed")
}

#[instrument(name = "tx_sync::save_contract_call_approved_events", skip_all, fields(chain = chain_config.chain_id))]
async fn save_contract_call_approved_events(chain_config: Chain, pg_pool: Arc<PgPool>, payload_hash: &str) -> Result<()> {
    let provider = Provider::<Http>::try_from(&chain_config.rpc_url)?;
    let provider = Arc::new(provider);

    info!("Looking for payload hash: {}", payload_hash);

    let address = chain_config.axelar_gateway_proxy.parse::<Address>().context("axelar_gateway_proxy parse failed")?;
    let address = ValueOrArray::Value(address);
    // filter for contract_address (2nd indexed topic)
    let topic2 = H256::from(chain_config.carbon_axelar_gateway.clone().parse::<Address>().context("axelar_gateway_proxy parse failed")?);
    // filter for contract_address (3rd indexed topic)
    let topic3 = payload_hash.parse::<H256>().context("payload_hash parse failed")?;

    // specify range of blocks to search
    // TODO: create a config to allow specifying evm block range outside of the limit for older txs because the current algorithm only looks for the most recent `max_query_blocks` blocks
    let current_block = provider.get_block_number().await?.as_u64();

    // Calculate the starting block to only search the latest x blocks
    let from_block = if current_block > chain_config.max_query_blocks - 1 { current_block - chain_config.max_query_blocks } else { 0 };

    let event = ContractCallApprovedEvent::new::<_, Provider<Http>>(
        Filter::new().address(address)
            .topic2(topic2)
            .topic3(topic3)
            .from_block(from_block),
        Arc::clone(&provider)
    );
    let events = event.query().await?;
    info!("{} events found!", events.iter().len());

    // loop all events found
    for event in events {
        let chain_config = chain_config.clone();
        let pg_pool = pg_pool.clone();
        save_call_contract_approved_event(chain_config, pg_pool, event).await;
    }

    Ok(())
}