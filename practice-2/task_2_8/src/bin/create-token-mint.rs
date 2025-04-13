use solana_utils::{load_env_keypair, get_connection, create_token_mint};

#[tokio::main]
async fn main() {
    let sender = load_env_keypair().expect("Can't load keypair");

    let connection= get_connection();

    create_token_mint(
        &connection,
        &sender
    ).await.expect("Can't create token_mint");
}