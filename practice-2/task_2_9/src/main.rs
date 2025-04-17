use std::env;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use solana_sdk::program_pack::Pack;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent
};
use spl_token_2022::{
    id,
    instruction::{initialize_mint2, initialize_multisig, mint_to},
    state::Multisig,
};

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

pub fn get_connection() -> solana_client::nonblocking::rpc_client::RpcClient {
    solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    )
}

#[tokio::main]
async fn main() {
    let connection = get_connection();

    let signer1 = load_env_keypair("SIGNER1_SECRET_KEY").expect("Can't load SIGNER1_SECRET_KEY");
    let signer2 = load_env_keypair("SIGNER2_SECRET_KEY").expect("Can't load SIGNER2_SECRET_KEY");
    let signer3 = load_env_keypair("SIGNER3_SECRET_KEY").expect("Can't load SIGNER3_SECRET_KEY");

    let multisig = Keypair::new();
    let multisig_pubkey = multisig.pubkey();
    let multisig_space = Multisig::get_packed_len();
    let multisig_rent = connection
        .get_minimum_balance_for_rent_exemption(multisig_space)
        .await
        .unwrap();

    let create_multisig_ix = system_instruction::create_account(
        &signer1.pubkey(),
        &multisig_pubkey,
        multisig_rent,
        multisig_space as u64,
        &id(),
    );

    let multisig_signers = [
        &signer1.pubkey(),
        &signer2.pubkey(),
        &signer3.pubkey(),
    ];

    let init_multisig_ix = initialize_multisig(
        &id(),
        &multisig_pubkey,
        &multisig_signers,
        2, // m of n (2 of 3)
    ).unwrap();

    let mint = Keypair::new();
    let mint_rent = connection
        .get_minimum_balance_for_rent_exemption(spl_token_2022::state::Mint::get_packed_len())
        .await
        .unwrap();

    let create_mint_ix = system_instruction::create_account(
        &signer1.pubkey(),
        &mint.pubkey(),
        mint_rent,
        spl_token_2022::state::Mint::get_packed_len() as u64,
        &id(),
    );

    let mint_decimals = 6;
    let init_mint_ix = initialize_mint2(
        &id(),
        &mint.pubkey(),
        &multisig_pubkey,
        None,
        mint_decimals,
    ).unwrap();

    let recipient = Keypair::new();
    let ata = get_associated_token_address_with_program_id(
        &recipient.pubkey(),
        &mint.pubkey(),
        &id(),
    );
    let create_ata_ix = create_associated_token_account_idempotent(
        &signer1.pubkey(),
        &recipient.pubkey(),
        &mint.pubkey(),
        &id(),
    );

    let mut tx1 = Transaction::new_with_payer(
        &[
            create_multisig_ix,
            init_multisig_ix,
            create_mint_ix,
            init_mint_ix,
            create_ata_ix,
        ],
        Some(&signer1.pubkey()),
    );
    let recent_blockhash = connection.get_latest_blockhash().await.unwrap();
    tx1.sign(&[&signer1, &multisig, &mint], recent_blockhash);
    let signature = connection.send_and_confirm_transaction(&tx1).await.unwrap();

    println!("Tx signature creating Multisig + Mint + ATA : {}", signature);
    println!("Multisig : {}", multisig_pubkey);
    println!("Mint : {}", mint.pubkey());
    println!("ATA: {}", ata);

    let mint_to_ix = mint_to(
        &id(),
        &mint.pubkey(),
        &ata,
        &multisig_pubkey,
        &[&signer1.pubkey(), &signer2.pubkey()],
        100 * 10u64.pow(mint_decimals as u32),
    ).unwrap();

    let mut tx2 = Transaction::new_with_payer(
        &[mint_to_ix],
        Some(&signer1.pubkey()),
    );
    let recent_blockhash = connection.get_latest_blockhash().await.unwrap();
    tx2.sign(&[&signer1, &signer2], recent_blockhash);

    match connection.send_and_confirm_transaction(&tx2).await {
        Ok(signature) => println!("Tx signature mint tokens : {}", signature),
        Err(e) => {
            eprintln!("Tx failed: {:#?}", e);
            return;
        }
    }
}