use std::env;
use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::{create_associated_token_account_idempotent};
use spl_token_2022::instruction::mint_to;

pub async fn create_user_ata(
    connection: &RpcClient,
    keypair: &Keypair,
    token_mint: &Pubkey
) -> anyhow::Result<Pubkey> {
    let token_account = get_associated_token_address_with_program_id(
        &keypair.pubkey(),
        &token_mint,
        &spl_token_2022::id(),
    );

    let token_account_ix = create_associated_token_account_idempotent(
        &keypair.pubkey(),
        &keypair.pubkey(),
        &token_mint,
        &spl_token_2022::id(),
    );

    let tx = Transaction::new_signed_with_payer(
        &[token_account_ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        connection.get_latest_blockhash().await?,
    );

    connection.send_and_confirm_transaction(&tx)
        .await
        .expect("Can't send tx for creating associated_token")
    ;

    Ok(token_account)
}

pub fn load_env_keypair(key: &str) -> Option<Keypair> {
    dotenv::dotenv().ok();

    let secret_key_str = env::var(key)
        .expect("Add {key} to .env!");

    let secret_key_str = secret_key_str.trim_matches(|c| c == '[' || c == ']');
    let secret_key_bytes: Vec<u8> = secret_key_str
        .split(',')
        .map(|s| s.trim().parse::<u8>().expect("Can't parse {key} string"))
        .collect();

    match Keypair::from_bytes(secret_key_bytes.as_ref()) {
        Ok(keypair) => {
            Some(keypair)
        }
        Err(error) => {
            eprintln!("{}", error);
            None
        }
    }
}

pub fn get_connection() -> RpcClient {
    RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    )
}

pub async fn mint_tokens(
    connection: &RpcClient,
    sender: &Keypair,
    token_mint_str: &str,
    recipient_associated_token_str: &str,
    amount: u64,
) -> anyhow::Result<()> {
    const MINOR_UNITS_PER_MAJOR_UNITS:u64 = 10u64.pow(2);

    let token_mint = Pubkey::from_str_const(token_mint_str);

    let recipient_associated_token = Pubkey::from_str_const(recipient_associated_token_str);

    let mint_to_ix = mint_to(
        &spl_token_2022::id(),
        &token_mint,
        &recipient_associated_token,
        &sender.pubkey(),
        &[&sender.pubkey()],
        amount * MINOR_UNITS_PER_MAJOR_UNITS,
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&sender.pubkey()),
        &[&sender],
        connection.get_latest_blockhash().await?,
    );

    let signature = connection.send_and_confirm_transaction(&transaction).await?;

    println!("Mint Token Transaction Signature: {}", signature);
    println!("✅ Success!");

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct JsonTx {
    json: String,
}

pub fn serialize_tx(tx: Transaction) -> anyhow::Result<String> {
    let tx_str = serde_json::to_string(&tx)?;

    let wrapped_tx = JsonTx { json: tx_str };

    let json_bytes = serde_json::to_vec(&wrapped_tx)?;

    Ok(general_purpose::STANDARD.encode(&json_bytes))
}

pub fn deserialize_tx(base64_encoded: String) -> anyhow::Result<Transaction> {
    let received_bytes = general_purpose::STANDARD.decode(&base64_encoded)?;
    let received_struct: JsonTx = serde_json::from_slice(&received_bytes)?;

    Ok(serde_json::from_str(&received_struct.json)?)
}