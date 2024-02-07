use std::sync::Arc;

use sqlx::PgPool;

use crate::conf::ChainConfig;

pub async fn init_all(evm_chains: Vec<ChainConfig>, pg_pool: Arc<PgPool>) {
    for chain in evm_chains {
        let pg_pool_clone = pg_pool.clone();
        println!("Watching for events to broadcast for {} to {}", &chain.name, &chain.rpc_url);
        tokio::spawn(async move {
            // TODO: implement
        });
    }
}

