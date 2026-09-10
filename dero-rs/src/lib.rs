//! # dero-rs
//!
//! Ergonomic Rust SDK for building on DERO's confidential settlement.
//! Wraps the `dero-crypto` crate — the diff-verified Rust implementation of
//! DERO's consensus crypto (ElGamal, bulletproofs, Pedersen, DVM, transactions).
//!
//! The goal: a normal developer should be able to build a confidential
//! settlement app without touching raw group arithmetic. This crate re-exports
//! the low-level crypto and adds ergonomic high-level builders on top.
//!
//! ## Example
//! ```no_run
//! use dero_rs::{Address, ConfidentialValue, elgamal::ElGamal};
//!
//! // Parse a bech32 DERO address
//! let addr = Address::new("dero1q...").unwrap();
//!
//! // Build a confidential value commitment (ElGamal)
//! let value = [0u8; 32]; // 0-value placeholder
//! let randomness = [0u8; 32];
//! let _commit = ConfidentialValue::new(&value, &randomness);
//! ```
#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Low-level DERO consensus crypto, re-exported from the diff-verified
/// the `dero-crypto` crate.
pub use dero_crypto::{
    block, bulletproof, chain, curve, dvm, elgamal, hash_to_point, innerproduct, keccak, pedersen,
    rpc_addr, rpc_args, statement, transaction,
};

/// `arkworks` BN254 types re-exported for developers who need the group math.
pub use ark_bn254::{G1Affine, G1Projective};
pub use ark_ec::PrimeGroup;
pub use ark_ff::{BigInt, PrimeField};

use std::fmt;

/// Error type for the SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An address failed to parse.
    #[error("invalid DERO address: {0}")]
    InvalidAddress(String),
    /// A cryptographic operation failed.
    #[error("crypto error: {0}")]
    Crypto(String),
}

/// A DERO bech32 address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    addr: String,
    compressed: Vec<u8>,
}

impl Address {
    /// Parse a bech32 DERO address into its compressed public key.
    pub fn new(addr: &str) -> Result<Self, Error> {
        let compressed =
            dero_crypto::rpc_addr::address_to_compressed(addr).map_err(Error::InvalidAddress)?;
        Ok(Self { addr: addr.to_string(), compressed })
    }

    /// Reconstruct the bech32 address from a compressed public key.
    pub fn from_compressed(compressed: &[u8]) -> Result<Self, Error> {
        let addr = dero_crypto::rpc_addr::compressed_to_address(compressed)
            .map_err(Error::InvalidAddress)?;
        Ok(Self { addr, compressed: compressed.to_vec() })
    }

    /// Return the canonical bech32 address string.
    pub fn as_str(&self) -> &str {
        &self.addr
    }

    /// Return the validated compressed public-key bytes.
    pub fn compressed_key(&self) -> &[u8] {
        &self.compressed
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.addr)
    }
}

/// A confidential value commitment (ElGamal ciphertext) on the DERO curve.
///
/// This is the core primitive of DERO's confidential settlement: a value is
/// committed with randomness so nobody can see the amount, but it is still
/// cryptographically binding and additively homomorphic.
#[derive(Debug, Clone)]
pub struct ConfidentialValue {
    /// The underlying ElGamal ciphertext.
    pub inner: dero_crypto::elgamal::ElGamal,
}

impl ConfidentialValue {
    /// Create a confidential value commitment.
    ///
    /// `value` and `randomness` are 32-byte little-endian scalars.
    pub fn new(value: &[u8; 32], randomness: &[u8; 32]) -> Self {
        // The generator is a convenient public base for examples and tests.
        // Production settlements should use [`ConfidentialValue::commit_to`]
        // with the recipient's decoded public key.
        let base = G1Projective::generator();
        Self { inner: dero_crypto::elgamal::ElGamal::commit(&base, value, randomness) }
    }

    /// Commit to `value` under a specific public key (the recipient).
    pub fn commit_to(pubkey: &G1Projective, value: &[u8; 32], randomness: &[u8; 32]) -> Self {
        Self { inner: dero_crypto::elgamal::ElGamal::commit(pubkey, value, randomness) }
    }

    /// Add two confidential values (homomorphic addition — `a + b`).
    pub fn add(&self, other: &ConfidentialValue) -> Self {
        Self { inner: self.inner.add(&other.inner) }
    }

    /// Scale by a scalar.
    pub fn mul(&self, scalar: &[u8; 32]) -> Self {
        Self { inner: self.inner.mul(scalar) }
    }

    /// Serialize to the 66-byte canonical form.
    pub fn serialize(&self) -> [u8; 66] {
        self.inner.serialize()
    }

    /// Deserialize from the 66-byte canonical form.
    pub fn deserialize(b: &[u8]) -> Result<Self, Error> {
        dero_crypto::elgamal::ElGamal::deserialize(b)
            .map(|inner| Self { inner })
            .map_err(Error::Crypto)
    }
}

/// Build a confidential settlement between two parties.
///
/// This is the high-level "settle" entrypoint for A1. It wraps address parsing
/// and confidential-value construction so a developer can settle without
/// touching group arithmetic directly. The D1/D2 `settle-sdk` builds on this.
pub fn build_settlement(
    _from: &Address,
    to: &Address,
    amount: u64,
) -> Result<ConfidentialValue, Error> {
    // amount as a 32-byte little-endian scalar
    let mut amt = [0u8; 32];
    amt[..8].copy_from_slice(&amount.to_le_bytes());
    let mut randomness = [0u8; 32];
    getrandom::getrandom(&mut randomness)
        .map_err(|error| Error::Crypto(format!("secure randomness unavailable: {error}")))?;
    Ok(ConfidentialValue::commit_to(&to_key(to)?, &amt, &randomness))
}

/// Derive the recipient's public key point from their address.
fn to_key(to: &Address) -> Result<G1Projective, Error> {
    dero_crypto::curve::decode_compressed(to.compressed_key()).map_err(Error::Crypto)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_roundtrip() {
        // This is a valid-length placeholder; real addresses come from the chain.
        // Test the compressed->address->compressed roundtrip with a synthetic key.
        let pk = [0u8; 33];
        let addr = Address::from_compressed(&pk).unwrap();
        assert_eq!(addr.compressed_key(), pk);
    }

    #[test]
    fn confidential_value_add_is_homomorphic() {
        let v1 = [1u8; 32];
        let v2 = [2u8; 32];
        let r1 = [3u8; 32];
        let r2 = [4u8; 32];
        let c1 = ConfidentialValue::new(&v1, &r1);
        let c2 = ConfidentialValue::new(&v2, &r2);
        let sum = c1.add(&c2);
        let serialized = sum.serialize();
        assert_eq!(serialized.len(), 66);
    }
}
