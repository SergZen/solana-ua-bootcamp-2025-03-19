use std::env;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

fn generate_keypair() -> Keypair {
    Keypair::new()
}

fn generate_keypair_with_prefix(prefix: &str) -> Keypair {
    loop  {
        let keypair = Keypair::new();
        if keypair.pubkey().to_string().starts_with(prefix) {
            show_keypair(&keypair);
            return keypair;
        } else {
            println!("The public key is: {}", keypair.pubkey());
        }
    }
}

fn generate_keypair_with_postfix(postfix: &str) -> Keypair {
    loop  {
        let keypair = Keypair::new();
        if keypair.pubkey().to_string().ends_with(postfix) {
            return keypair;
        } else {
            println!("The public key is: {}", keypair.pubkey());
        }
    }
}

fn load_keypair () -> Option<Keypair> {
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

fn check_balance(public_key: Pubkey) -> Result<(), Box<dyn std::error::Error>> {
    let connection = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    println!("⚡️ Connected to devnet");

    let balance_in_lamports = connection.get_balance(&public_key)?;
    let balance_in_sol = balance_in_lamports as f64 / LAMPORTS_PER_SOL as f64;

    println!(
        "💰 The balance for the wallet at address {} is: {} SOL",
        public_key, balance_in_sol
    );

    Ok(())
}

fn show_keypair(keypair: &Keypair) {
    println!("The public key is: {}", keypair.pubkey());
    println!("The secret key is: {:?}", keypair.secret());
}

fn main() {
    dotenv::dotenv().ok();

    println!("-- generate_keypair --");
    let keypair = generate_keypair();
    show_keypair(&keypair);

    println!("-- load_keypair --");
    let keypair = load_keypair().expect("Can't load keypair");
    show_keypair(&keypair);

    println!("-- check_balance --");
    check_balance(keypair.pubkey()).expect("Can't check balance");

    // println!("-- generate_keypair_with_prefix --");
    // let keypair = generate_keypair_with_prefix("serg");
    // show_keypair(&keypair);
}
