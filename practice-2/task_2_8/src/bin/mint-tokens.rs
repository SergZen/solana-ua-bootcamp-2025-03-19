use solana_utils::{load_env_keypair, get_connection, mint_tokens};

#[tokio::main]
async fn main() {
    let sender = load_env_keypair().expect("Can't load keypair");

    let connection= get_connection();

    let token_mint_str = "5PcbXNtkdeVcRqZxizFxAhnrQi6j8SEx1uX8oMEimcZy";
    let recipient_associated_token_str = "5hmQc3KvCjX5ypDFpTfs6qJ4hSaAb7dNiPk4rEJGqhd4";
    let amount = 10;

    mint_tokens(
        &connection,
        &sender,
        token_mint_str,
        recipient_associated_token_str,
        amount,
    ).await.expect("Can't mint tokens");
}