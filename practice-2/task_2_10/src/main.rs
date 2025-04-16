mod utils;

use std::str::FromStr;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    message::Message,
    signature::{Signer},
    transaction::Transaction,
};
use spl_token_2022::instruction::transfer_checked;
use crate::utils::{create_user_ata, deserialize_tx, get_connection, load_env_keypair, mint_tokens, serialize_tx};

#[tokio::main]
async fn main() {
    let amount = 50_000_000;
    let token_decimals = 2;
    let token_mint_str = "5PcbXNtkdeVcRqZxizFxAhnrQi6j8SEx1uX8oMEimcZy";
    let token_mint = Pubkey::from_str(token_mint_str).expect("Can't get mint pubkey");

    let connection = get_connection();
    let mint_owner_keypair = load_env_keypair("MINT_OWNER_SECRET_KEY")
        .expect("Can't load MINT_OWNER_SECRET_KEY")
        ;

    let sender_keypair = load_env_keypair("SENDER_SECRET_KEY")
        .expect("Can't load SENDER_SECRET_KEY")
        ;
    println!("Відправник (власник токенів): {}", sender_keypair.pubkey());
 
    let sender_token_account = create_user_ata(
        &connection,
        &sender_keypair,
        &token_mint,
    ).await.expect("Can't create sender token account");
    println!("Токен-акаунт відправника: {}", sender_token_account);

    let recipient_keypair = load_env_keypair("RECIPIENT_SECRET_KEY")
        .expect("Can't load RECIPIENT_SECRET_KEY")
        ;
    println!("Отримувач (платник комісії): {}", recipient_keypair.pubkey());
 
    let recipient_token_account = create_user_ata(
        &connection,
        &recipient_keypair,
        &token_mint,
    ).await.expect("Can't create recipient token account");
    println!("Токен-акаунт отримувача: {}", recipient_token_account);

    mint_tokens(
        &connection,
        &mint_owner_keypair,
        token_mint_str,
        sender_token_account.to_string().as_str(),
        amount,
    ).await.expect("Can't mint tokens for sender");

    let transfer_ix = transfer_checked(
        &spl_token_2022::id(),
        &sender_token_account,
        &token_mint,
        &recipient_token_account,
        &sender_keypair.pubkey(),
        &[&sender_keypair.pubkey()],
        amount,
        token_decimals,
    ).expect("Can't create transfer instruction");

    let recent_blockhash = connection.get_latest_blockhash()
        .await
        .expect("Can't get latest blockhash")
        ;

    let message = Message::new(
        &[transfer_ix],
        Some(&recipient_keypair.pubkey())
    );

    let mut tx = Transaction::new_unsigned(message);
    tx.partial_sign(&[&sender_keypair], recent_blockhash);

    let base64_encoded = serialize_tx(tx).expect("Can't serialize tx");

    let mut recovered_tx: Transaction = deserialize_tx(base64_encoded).expect("Can't deserialize tx");

    recovered_tx.partial_sign(&[&recipient_keypair], recent_blockhash);

    match connection.send_and_confirm_transaction(&recovered_tx).await {
        Ok(signature) => {
            println!("Трансфер успішно виконано!");
            println!("Signature: {}", signature);
        }
        Err(err) => {
            println!("Помилка відправлення транзакції: {}", err);
        }
    }
}