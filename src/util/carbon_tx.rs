use anyhow::{Result};
use cosmrs::crypto::secp256k1;
use cosmrs::{Coin, tx};
use cosmrs::tx::{Fee, Msg, SignDoc, SignerInfo};
use crate::conf::Carbon;
use crate::util::carbon_msg::MsgStartRelay;
use crate::util::cosmos::{estimate_gas, get_account_info, get_latest_block_height, send_transaction};

pub async fn send_msg_start_relay(
    conf: Carbon,
    nonce: u64,
    pending_action_type: u64,
) -> Result<()> {

    // let sender_private_key = secp256k1::SigningKey::from_bytes(&hex::decode(&conf.relayer_private_key)?).map_err(|e| anyhow!(e))?;
    let sender_private_key = secp256k1::SigningKey::random();

    let sender_public_key = sender_private_key.public_key();
    let sender_account_id = sender_public_key.account_id(&conf.account_prefix).unwrap();

    let (account_number, sequence) = get_account_info(&sender_account_id.to_string(), &conf.rest_url).await?;

    let msg_start_relay = MsgStartRelay {
        relayer: sender_account_id.clone().to_string(),
        nonce,
        pending_action_type,
    }
        .to_any()
        .unwrap();

    let chain_id = conf.chain_id.parse().unwrap();

    let fee_coin = Coin::new(2000, "swth").expect("unable to parse coin");

    let tx_body = tx::BodyBuilder::new().msg(msg_start_relay.clone()).finish();
    let signer_info = SignerInfo::single_direct(Some(sender_public_key.into()), sequence);
    let auth_info = signer_info.clone().auth_info(Fee::from_amount_and_gas(fee_coin.clone(), "0".parse::<u64>()?));


    let gas_estimate = estimate_gas(&conf.rest_url, &tx_body, &auth_info).await?;
    let auth_info = signer_info.clone().auth_info(Fee::from_amount_and_gas(fee_coin.clone(), gas_estimate));


    // add timeout height
    let latest_block_height = get_latest_block_height(&conf.rpc_url).await?;
    let timeout_height = latest_block_height + 100; // Set timeout height to current height + 100


    let tx_body = tx::BodyBuilder::new().msg(msg_start_relay).timeout_height(timeout_height).finish();
    let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number).expect("signdoc failed");
    let tx_signed = sign_doc.sign(&sender_private_key).expect("signing failed");
    let tx_bytes = tx_signed.to_bytes().expect("to_bytes failed");

    send_transaction(&conf.rest_url, tx_bytes).await
}
