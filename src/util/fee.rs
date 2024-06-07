use tracing::info;
use crate::conf::Carbon;
use crate::db::RelayDetails;

// carbon
#[derive(Debug, Clone, PartialEq)]
pub struct FeeResponse {
    pub withdraw: String,
    pub register_token: String,
    pub deregister_token: String,
    pub deploy_native_token: String,
    pub quoted_at: String,
}

pub fn should_relay(_carbon_config: &Carbon, relay_details: RelayDetails) -> bool {
    info!("relay_details from Carbon {:?}", relay_details);
    info!("fee: {:?}", relay_details.fee);
    // TODO: process relay fee and see if fee makes sense
    // let fee = get_hydrogen_fee(carbon_config.clone(), relay_details);
    // info!("hydrogen fee: {}", fee);

    return true
}

pub fn has_expired(_carbon_config: &Carbon, relay_details: RelayDetails) -> bool {
    info!("relay_details from Carbon {:?}", relay_details);
    info!("fee: {:?}", relay_details.fee);
    // TODO: process relay fee and see if fee makes sense
    // let fee = get_hydrogen_fee(carbon_config.clone(), relay_details);
    // info!("hydrogen fee: {}", fee);

    return relay_details.has_expired()
}

// pub fn get_hydrogen_fee(carbon_config: Carbon, relay_details: RelayDetails) -> FeeResponse {
//
// }

//
// async fn validate_withdraw(pg_pool: &Arc<PgPool>, nonce: &BigDecimal) -> anyhow::Result<()> {
//     // Check if we should broadcast this event by checking the withdraw_token_confirmed_events
//     let result = sqlx::query_as::<_, DbWithdrawTokenConfirmedEvent>(
//         r#"
//                         SELECT * FROM withdraw_token_confirmed_events
//                         WHERE nonce = $1
//                         AND (coin->>'amount')::numeric > 0
//                         AND (relay_fee->>'amount')::numeric > 0
//                         "#,
//     )
//         .bind(nonce)
//         .fetch_optional(pg_pool.as_ref()).await?;
//
//     let withdraw_event = match result {
//         Some(event) => {
//             info!("Found matching withdraw_token_confirmed_events in DB with nonce: {:?}", &nonce);
//             event
//         }
//         None => {
//             anyhow::bail!("Skipping as DbWithdrawTokenConfirmedEvent nonce {:?} does not exist in DB or has 0 amounts", &nonce);
//         }
//     };
//
//     // TODO: translate to handle different relay fee denom and amounts
//     if withdraw_event.relay_fee.amount < 10 {
//         // 10 is just an arbitrary number, we should do custom logic to convert price
//         warn!("withdraw_event.relay_fee.amount < 10");
//         anyhow::bail!("withdraw_event.relay_fee.amount < 10");
//     }
//     Ok(())
// }
