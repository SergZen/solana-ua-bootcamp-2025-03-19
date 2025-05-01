use std::time::Duration;
use log::info;
use tokio::time::sleep;
use crate::utils::{create_tx, deserialize_tx, serialize_tx, sign_and_send_tx};

mod utils;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp_millis()
        .init();

    let (tx, nonce) = create_tx().await;
    let serialized_tx = serialize_tx(tx).unwrap();
    info!("Mint tx was created");

    sleep(Duration::from_secs(150)).await;

    let tx = deserialize_tx(serialized_tx).unwrap();

    let signature = sign_and_send_tx(tx, nonce).await.unwrap();
    info!("Mint tx Signature: {}", signature);
}