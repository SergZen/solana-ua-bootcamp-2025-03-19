use std::env;
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose;
use log::{error, info};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};
use solana_program::hash::Hash;
use solana_program::nonce::State;
use solana_program::nonce::state::Versions;
use solana_program::system_instruction::advance_nonce_account;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_sdk::signature::Signature;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::{create_associated_token_account_idempotent}
};
use spl_token_2022::{
    id,
    instruction::{initialize_mint2, initialize_multisig, mint_to},
    state::Multisig
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
            error!("{}", error);
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

async fn get_nonce(connection: &RpcClient, nonce_account_pubkey: Pubkey) -> anyhow::Result<Hash>{
    let nonce_account_data = connection.get_account(&nonce_account_pubkey)
        .await?;

    match bincode::deserialize::<Versions>(&nonce_account_data.data) {
        Ok(Versions::Current(state)) => match *state {
            State::Initialized(ref data) => Ok(data.blockhash()),
            _ => Err(anyhow!("Nonce can't be initialized")),
        },
        _ => Err(anyhow!("Can't deserialize")),
    }
}

async fn setup_nonce_account(connection: &RpcClient, payer: &Keypair) -> anyhow::Result<Keypair> {
    let nonce_account = Keypair::new();
    let nonce_authority = payer.pubkey();

    let min_balance = connection.get_minimum_balance_for_rent_exemption(80)
        .await?;

    let create_nonce_account_ix = system_instruction::create_nonce_account(
        &payer.pubkey(),
        &nonce_account.pubkey(),
        &nonce_authority,                 // Nonce authority
        min_balance,
    );

    let tx = Transaction::new_signed_with_payer(
        &create_nonce_account_ix,
        Some(&payer.pubkey()),
        &[payer, &nonce_account],
        connection.get_latest_blockhash().await.unwrap(),
    );

    connection.send_and_confirm_transaction(&tx).await
        .expect("Can't create nonce account");

    Ok(nonce_account)
}

pub async fn create_tx() -> (Transaction, Hash) {
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

    info!("Tx signature creating Multisig + Mint + ATA : {}", signature);
    info!("Multisig : {}", multisig_pubkey);
    info!("Mint : {}", mint.pubkey());
    info!("ATA: {}", ata);

    let mint_to_ix = mint_to(
        &id(),
        &mint.pubkey(),
        &ata,
        &multisig_pubkey,
        &[&signer1.pubkey(), &signer2.pubkey()],
        100 * 10u64.pow(mint_decimals as u32),
    ).unwrap();

    let nonce_account = setup_nonce_account(&connection, &signer1).await.expect("Can't create nonce account");
    let nonce = get_nonce(&connection, nonce_account.pubkey())
        .await
        .expect("Can't get nonce");
    let advance_nonce_ix = advance_nonce_account(
        &nonce_account.pubkey(),
        &signer1.pubkey(),
    );

    let mut tx2 = Transaction::new_with_payer(
        &[advance_nonce_ix, mint_to_ix],
        Some(&signer1.pubkey()),
    );
    tx2.partial_sign(&[&signer1], nonce);

    (tx2, nonce)
}

pub fn serialize_tx(tx: Transaction) -> anyhow::Result<String> {
    let serialized = bincode::serialize(&tx)?;
    Ok(general_purpose::STANDARD.encode(&serialized))
}

pub fn deserialize_tx(base64_encoded: String) -> anyhow::Result<Transaction> {
    let received_bytes = general_purpose::STANDARD.decode(&base64_encoded)?;
    Ok(bincode::deserialize(received_bytes.as_ref())?)
}

pub async fn sign_and_send_tx(mut tx: Transaction, nonce:Hash) -> anyhow::Result<Signature> {
    let connection = get_connection();

    let signer2 = load_env_keypair("SIGNER2_SECRET_KEY").expect("Can't load SIGNER2_SECRET_KEY");

    tx.partial_sign(&[&signer2], nonce);

    let signature = connection.send_and_confirm_transaction(&tx).await?;

    Ok(signature)
}