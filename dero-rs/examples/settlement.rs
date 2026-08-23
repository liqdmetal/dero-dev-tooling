//! Example: build a confidential settlement and inspect the commitment.
//!
//! Run with: `cargo run -p dero-rs --example settlement`
use dero_rs::{Address, ConfidentialValue, PrimeGroup};

fn main() {
    // NOTE: these are synthetic 33-byte compressed keys for the example.
    // In production these come from real DERO addresses (rpc_addr).
    let pk = [2u8; 33]; // placeholder recipient public key
    let to = Address::from_compressed(&pk).expect("address from pk");

    // A confidential value of 1000 atomic units.
    let mut amt = [0u8; 32];
    amt[..8].copy_from_slice(&1000u64.to_le_bytes());
    let randomness = [7u8; 32]; // NOTE: production uses secure randomness

    let commit = ConfidentialValue::commit_to(
        &dero_rs::G1Projective::generator(),
        &amt,
        &randomness,
    );

    println!("recipient: {}", to);
    println!("confidential value serialized ({} bytes):", commit.serialize().len());
    println!("  {:?}", &commit.serialize()[..]);
    println!("(the amount 1000 is hidden in this commitment — ElGamal)");
}
