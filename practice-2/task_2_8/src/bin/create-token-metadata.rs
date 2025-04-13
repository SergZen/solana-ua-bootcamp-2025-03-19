use mpl_token_metadata::types::DataV2;

use solana_utils::{create_token_metadata, get_connection, load_env_keypair};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let sender = load_env_keypair().expect("Can't load keypair");

    let connection= get_connection();

    let token_mint_str = "5PcbXNtkdeVcRqZxizFxAhnrQi6j8SEx1uX8oMEimcZy";

    let metadata_data = DataV2 {
        name: "Solana UA Bootcamp SZ3".to_string(),
        symbol: "SZ3-UAB-3".to_string(),
        uri: "https://arweave.net".to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    create_token_metadata(
        &connection,
        &sender,
        token_mint_str,
        metadata_data
    ).await.expect("Can't create token metadata");
}