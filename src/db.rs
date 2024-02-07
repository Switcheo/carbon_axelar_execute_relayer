use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WithdrawTokenAcknowledgedEvent {
    coin: String,
    connection_id: String,
    receiver: String,
    relay_fee: String,
    relayer_deposit_address: String,
    sender: String,
}
