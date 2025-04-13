use std::env;
use std::str::FromStr;

use mpl_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;
use mpl_token_metadata::instructions::CreateV1Builder;
use mpl_token_metadata::types::{DataV2, TokenStandard};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::{create_account, transfer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_memo::build_memo;
use spl_token_2022::{
    id as token_2022_program_id,
    instruction::{initialize_mint, mint_to},
    state::Mint
};

use crate::explorer_link::get_explorer_link;

mod explorer_link;

pub fn load_env_keypair() -> Option<Keypair> {
    dotenv::dotenv().ok();

    let secret_key_str = env::var("SECRET_KEY")
        .expect("Add SECRET_KEY to .env!");

    let secret_key_str = secret_key_str.trim_matches(|c| c == '[' || c == ']');
    let secret_key_bytes: Vec<u8> = secret_key_str
        .split(',')
        .map(|s| s.trim().parse::<u8>().expect("Can't parse secret_key string"))
        .collect();

    match Keypair::from_bytes(secret_key_bytes.as_ref()) {
        Ok(keypair) => {
            Some(keypair)
        }
        Err(_) => {
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

pub fn show_public_key(pubkey: Pubkey) {
    println!("🔑 Our public key is: {}", pubkey);
}

pub async fn send_sol_with_memo(
    connection: &RpcClient,
    sender: &Keypair,
    recipient_str: &str,
    amount: f64,
    memo_text: &str,
) -> anyhow::Result<()> {
    show_public_key(sender.pubkey());

    let recipient = Pubkey::from_str_const(recipient_str);
    println!("💸 Attempting to send {} SOL to {}...", amount, recipient);

    let send_sol_ix = transfer(
        &sender.pubkey(),
        &recipient,
        (amount * LAMPORTS_PER_SOL as f64) as u64,
    );

    println!("📝 memo is: {}", memo_text);

    let add_memo_ix = build_memo(
        memo_text.as_ref(),
        &[&sender.pubkey()],
    );

    let mut transaction = Transaction::new_with_payer(
        &[send_sol_ix, add_memo_ix],
        Some(&sender.pubkey()),
    );
    transaction.sign(&[sender], connection.get_latest_blockhash().await?);

    match connection.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => println!("✅ Transaction confirmed, signature: {} !", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    };

    Ok(())
}

pub async fn create_token_mint(
    connection: &RpcClient,
    sender: &Keypair,
) -> anyhow::Result<()> {
    show_public_key(sender.pubkey());

    let token_mint = Keypair::new();

    let mint_space = Mint::LEN;
    let rent = connection.get_minimum_balance_for_rent_exemption(mint_space).await?;

    let create_account_ix = create_account(
        &sender.pubkey(),
        &token_mint.pubkey(),
        rent,
        mint_space as u64,
        &token_2022_program_id(),
    );

    let initialize_mint_ix = initialize_mint(
        &token_2022_program_id(),
        &token_mint.pubkey(),
        &sender.pubkey(),
        Some(&sender.pubkey()),
        2,
    )?;

    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, initialize_mint_ix],
        Some(&sender.pubkey()),
        &[&sender, &token_mint],
        connection.get_latest_blockhash().await?,
    );

    let transaction_signature = connection.send_and_confirm_transaction(&transaction).await?;

    println!("Mint Address: {}", token_mint.pubkey());
    println!("Transaction Signature: {}", transaction_signature);

    let link = get_explorer_link("address", token_mint.pubkey().to_string(), "devnet");
    println!("✅ Token Mint: {}", link);

    Ok(())
}

pub async fn create_token_account(
    connection: &RpcClient,
    sender: &Keypair,
    token_mint_str: &str,
    recipient_str: &str,
) -> anyhow::Result<()> {
    show_public_key(sender.pubkey());

    let token_mint = Pubkey::from_str_const(token_mint_str);

    let recipient = Pubkey::from_str_const(recipient_str);

    let token_account = get_associated_token_address_with_program_id(
        &recipient,
        &token_mint,
        &spl_token_2022::id(),
    );

    let ixs = [
        create_associated_token_account(
            &sender.pubkey(),
            &recipient,
            &token_mint,
            &spl_token_2022::id(),
        ),
    ];

    let transaction = Transaction::new_signed_with_payer(
        &ixs,
        Some(&sender.pubkey()),
        &[&sender],
        connection.get_latest_blockhash().await?,
    );

    let result = connection.send_and_confirm_transaction(&transaction).await;

    match result {
        Ok(transaction_signature) => {
            println!("Transaction Signature: {}", transaction_signature);
            println!("Token Account: {}", token_account.to_string());

            let link = get_explorer_link("address", token_account.to_string(), "devnet");
            println!("✅ Created token account: {}", link);
        }
        Err(e) => {
            if e.to_string().contains("already in use") {
                println!("Token Account: {}", token_account.to_string());

                let link = get_explorer_link("address", token_account.to_string(), "devnet");
                println!("✅ Token Account already exists: {}", link);
            } else {
                eprintln!("Error creating Token Account: {}", e);
            }
        }
    }

    Ok(())
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

    println!("Transaction Signature: {}", signature);
    println!("✅ Success!");

    let link = get_explorer_link("transaction", signature.to_string(), "devnet");
    println!("Mint Token Transaction: {}", link);

    Ok(())
}

pub async fn create_token_metadata(
    connection: &RpcClient,
    sender: &Keypair,
    token_mint_str: &str,
    metadata_data: DataV2
) -> anyhow::Result<()> {
    let token_mint = Pubkey::from_str(token_mint_str)?;

    let seeds = &[
        b"metadata",
        TOKEN_METADATA_PROGRAM_ID.as_ref(),
        token_mint.as_ref(),
    ];
    let (metadata_pda, _) = Pubkey::find_program_address(seeds, &TOKEN_METADATA_PROGRAM_ID);

    let create_metadata_ix = CreateV1Builder::new()
        .metadata(metadata_pda)
        .mint(token_mint, false)
        .authority(sender.pubkey())
        .payer(sender.pubkey())
        .update_authority(sender.pubkey(), true)
        .token_standard(TokenStandard::Fungible)
        .name(metadata_data.name)
        .symbol(metadata_data.symbol)
        .uri(metadata_data.uri)
        .seller_fee_basis_points(metadata_data.seller_fee_basis_points)
        .instruction();

    let transaction = Transaction::new_signed_with_payer(
        &[create_metadata_ix],
        Some(&sender.pubkey()),
        &[&sender],
        connection.get_latest_blockhash().await?,
    );

    let signature = connection.send_and_confirm_transaction(&transaction).await?;
    println!("Transaction signature: {}", signature);

    let token_mint_link = get_explorer_link("address", token_mint.to_string(), "devnet");
    println!("✅ Look at the token mint again: {}", token_mint_link);

    Ok(())
}