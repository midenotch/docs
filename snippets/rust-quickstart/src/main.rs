// src/main.rs
//
// Entry point for `cargo run`.  When the "network" feature is active the binary
// scans the Soroban testnet for stealth payments addressed to the keypair
// supplied via STELLAR_SECRET.  Without "network" it just derives keys and
// prints them — useful for local testing without an RPC call.

pub mod keypair;
pub mod network;
pub mod scan;
pub mod send;
pub mod stealth;

use network::STEALTH_SIGNING_MESSAGE;

#[cfg(not(feature = "network"))]
fn main() -> anyhow::Result<()> {
    use ed25519_dalek::Signer;
    use keypair::load_keypair;

    let secret = std::env::var("STELLAR_SECRET").unwrap_or_else(|_| {
        keypair::fixture_keypair()
            .signing_key
            .to_bytes()
            .iter()
            .fold(String::new(), |acc, _| acc)
    }); // unreachable branch; just satisfies the type
    let secret = if secret.is_empty() {
        // No env var — use the fixture keypair for a smoke-run.
        println!("STELLAR_SECRET not set; using built-in fixture keypair.");
        return run_with_fixture();
    } else {
        secret
    };

    let account = load_keypair(&secret)?;
    println!("Loaded account: {}", account.address);

    let sig: [u8; 64] = account
        .signing_key
        .sign(STEALTH_SIGNING_MESSAGE.as_bytes())
        .to_bytes();
    let keys = stealth::derive_stealth_keys(&sig);
    println!("spending pub : {}", hex::encode(keys.spending_pub));
    println!("viewing  pub : {}", hex::encode(keys.viewing_pub));
    Ok(())
}

#[cfg(not(feature = "network"))]
fn run_with_fixture() -> anyhow::Result<()> {
    use ed25519_dalek::Signer;

    let acct = keypair::fixture_keypair();
    println!("Fixture account: {}", acct.address);
    let sig: [u8; 64] = acct
        .signing_key
        .sign(STEALTH_SIGNING_MESSAGE.as_bytes())
        .to_bytes();
    let keys = stealth::derive_stealth_keys(&sig);
    println!("spending pub : {}", hex::encode(keys.spending_pub));
    println!("viewing  pub : {}", hex::encode(keys.viewing_pub));
    Ok(())
}

#[cfg(feature = "network")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secret = std::env::var("STELLAR_SECRET")?;
    scan::run_scan(&secret).await
}
