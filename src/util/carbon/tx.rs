use std::str::FromStr;

use anyhow::Result;
use bip32::DerivationPath;
use bip39::Mnemonic;
use cosmrs::{Coin, tx};
use cosmrs::crypto::secp256k1;
use cosmrs::tendermint::block::Height;
use cosmrs::tx::{Fee, SignDoc, SignerInfo};
use prost_types::Any;
use serde_json::Value;

use crate::conf::Carbon;
use crate::util::cosmos::{get_account_info, get_latest_block_height, send_transaction};

const COSMOS_HD_PATH: &str = "m/44'/118'/0'/0/0";

pub async fn send_msg_via_tx(
    conf: &Carbon,
    msg: impl Into<Any>
) -> Result<Value> {
    let tx_bytes = create_signed_tx(&conf, msg).await?;

    // send tx
    send_transaction(&conf.rest_url, tx_bytes).await
}

async fn create_signed_tx(conf: &Carbon, msg: impl Into<Any>) -> Result<Vec<u8>> {
// Generate private key from mnemonic
    let mnemonic = Mnemonic::parse(&conf.relayer_mnemonic)?;

    let seed = mnemonic.to_seed("");
    let derivation_path = DerivationPath::from_str(COSMOS_HD_PATH)?;
    let sender_private_key = secp256k1::SigningKey::derive_from_path(&seed, &derivation_path).expect("private key could not be derived");

    let sender_public_key = sender_private_key.public_key();
    let sender_account_id = sender_public_key.account_id(&conf.account_prefix).unwrap();

    let (account_number, sequence) = get_account_info(&conf.rest_url, &sender_account_id.to_string()).await?;

    let chain_id = conf.chain_id.parse().unwrap();

    let signer_info = SignerInfo::single_direct(Some(sender_public_key.into()), sequence);

    // set hard-coded gas values into auth_info
    let fee_coin = Coin::new(100000000, "swth").expect("unable to parse coin");
    let default_gas: u64 = 1000000000;
    let gas_multiplier: f64 = 1.2;
    let adjusted_gas = (default_gas as f64 * gas_multiplier) as u64;
    let auth_info = signer_info.clone().auth_info(Fee::from_amount_and_gas(fee_coin.clone(), adjusted_gas));

    // add timeout height
    let latest_block_height = get_latest_block_height(&conf.rpc_url).await?;
    let timeout_height = latest_block_height + 100; // Set timeout height to current height + 100
    let timeout_height = Height::try_from(timeout_height)?;

    // create tx
    let tx_body = tx::BodyBuilder::new().msg(msg).timeout_height(timeout_height).finish();
    let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number).expect("signdoc failed");
    let tx_signed = sign_doc.sign(&sender_private_key).expect("signing failed");
    let tx_bytes = tx_signed.to_bytes().expect("to_bytes failed");
    Ok(tx_bytes)
}
