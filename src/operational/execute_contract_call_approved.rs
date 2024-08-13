use anyhow::{Context,Result};
use base64::Engine;
use base64::engine::general_purpose;
use ethers::abi::RawLog;
use ethers::prelude::{EthEvent, H256, Middleware};
use ethers::utils::hex::encode_prefixed;
use sqlx::types::BigDecimal;
use tracing::{error, info};

use crate::conf::{Chain};
use crate::db::DbContractCallApprovedEvent;
use crate::evm::broadcaster::{broadcast_tx, init_provider};
use crate::util::evm::ContractCallApprovedEvent;

// Utility function to check if a string is hex
fn is_hex(s: &str) -> bool {
    let s = if s.starts_with("0x") { &s[2..] } else { s };
    s.chars().all(|c| c.is_digit(16))
}

// Utility function to convert base64 to hex
fn base64_to_hex(base64_str: &str) -> String {
    let payload_bytes = general_purpose::STANDARD.decode(base64_str).unwrap();
    encode_prefixed(&payload_bytes)
}

pub async fn execute_contract_call_approved(evm_chains: &Vec<Chain>, chain_id: String, tx_hash: String, payload: String) -> Result<()> {
    let chain_config = evm_chains.iter().find(|a| a.chain_id == chain_id).unwrap();
    let chain_config = chain_config.clone();

    info!("Finding ContractCallApproved event on {:?} for tx_hash: {:?} for execution", &chain_config.rpc_url, tx_hash);

    // find event first
    let provider = init_provider(chain_config.clone()).await?;
    let tx_hash = tx_hash.parse::<H256>().context("tx_hash parse failed")?;

    // Fetch the transaction receipt
    let receipt = provider.get_transaction_receipt(tx_hash).await?.unwrap();

    // Convert payload to hex if necessary
    let payload_hex = if is_hex(&payload) {
        // Ensure it has 0x prefix
        if payload.starts_with("0x") {
            payload.clone()
        } else {
            format!("0x{}", payload)
        }
    } else {
        // Convert base64 to hex
        base64_to_hex(&payload)
    };

    // Iterate through the logs to find your specific event
    for log in receipt.logs {
        // Decode the log
        if let Ok(decoded_log) = ContractCallApprovedEvent::decode_log(&RawLog {
            topics: log.topics,
            data: log.data.to_vec(),
        }) {
            // Convert to DbContractCallApprovedEvent
            let db_event = DbContractCallApprovedEvent {
                id: 0, // Just a  random id
                blockchain: chain_config.chain_id.clone(),
                broadcast_status: "pending".to_string(),
                command_id: hex::encode(decoded_log.command_id.as_bytes()),
                source_chain: decoded_log.source_chain,
                source_address: decoded_log.source_address,
                contract_address: hex::encode(decoded_log.contract_address.as_bytes()),
                payload_hash: hex::encode(decoded_log.payload_hash.as_bytes()),
                source_tx_hash: hex::encode(decoded_log.source_tx_hash.as_bytes()),
                source_event_index: BigDecimal::from(decoded_log.source_event_index.as_u64()),
                payload: payload_hex.clone(), // Set payload appropriately
            };

            // Call broadcast_tx function
            // broadcast_tx(chain_config.clone(), db_event, provider.clone()).await.context("Failed broadcast");
            match broadcast_tx(chain_config.clone(), db_event, provider.clone()).await {
                Ok(_) => {
                    info!("broadcast successful");
                },
                Err(e) => {
                    // Handle the error, log it, and add context
                    error!("Error broadcasting transaction: {:?}", e);
                }
            }
        }
    }
    Ok(())
}