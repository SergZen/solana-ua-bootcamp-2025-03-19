use solana_utils::{load_env_keypair, get_connection, send_sol_with_memo};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let sender = load_env_keypair().expect("Can't load keypair");

    let connection= get_connection();

    let recipient_str = "serg2Wr1AVcjA81qDRkFojdDofqsggsiRS8wKGprsvJ";
    let amount = 0.01;
    let memo_text = "Hello from Solana Training!";

    send_sol_with_memo(
        &connection,
        &sender,
        recipient_str,
        amount,
        memo_text
    ).await.expect("Can't send SOL");
}