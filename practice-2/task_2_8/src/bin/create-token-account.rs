use solana_utils::{load_env_keypair, get_connection, create_token_account};

#[tokio::main]
async fn main() {
    let sender = load_env_keypair().expect("Can't load keypair");

    let connection= get_connection();

    let token_mint_str = "5PcbXNtkdeVcRqZxizFxAhnrQi6j8SEx1uX8oMEimcZy";
    let recipient_str = "serg2Wr1AVcjA81qDRkFojdDofqsggsiRS8wKGprsvJ";

    create_token_account(
        &connection,
        &sender,
        token_mint_str,
        recipient_str
    ).await.expect("Can't create token account");
}