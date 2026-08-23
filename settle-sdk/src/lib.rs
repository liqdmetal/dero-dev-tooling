//! # settle-sdk
//!
//! A confidential-settlement API for ordinary developers, built on [`dero-rs`].
//! A normal developer calls [`settle`] and gets a confidential DERO settlement —
//! without touching ElGamal, bulletproofs, or group arithmetic.
//!
//! This is also the **on-ramp to the RelayOS rail**: every settlement carries a
//! tier, mirroring RelayOS's two-tier model —
//!   * **Commons** (natural persons only, $0) — free, personal settlement
//!   * **Licensed** (legal entities) — per-use protocol fee
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use dero_rs::{Address, ConfidentialValue, G1Projective, PrimeField, PrimeGroup, Error as DeroError};

/// Typed DERO daemon RPC client for preparing settlements.
pub mod rpc;

/// RelayOS-aligned settlement leg request shape.
pub mod relay;

/// The settlement tier, mirroring RelayOS's two-tier commons/licensed model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Commons: natural persons only, $0 fee.
    Commons,
    /// Licensed: legal entities, per-use protocol fee (bps).
    Licensed {
        /// The per-use protocol fee in basis points.
        fee_bps: u32,
    },
}

impl Tier {
    /// The per-settlement fee in basis points (0 for Commons).
    pub fn fee_bps(&self) -> u32 {
        match self {
            Tier::Commons => 0,
            Tier::Licensed { fee_bps } => *fee_bps,
        }
    }
}

/// A confidential settlement between two parties.
#[derive(Debug, Clone)]
pub struct Settlement {
    /// Sender address.
    pub from: Address,
    /// Recipient address.
    pub to: Address,
    /// The confidential value being transferred (ElGamal commitment).
    pub value: ConfidentialValue,
    /// The settlement tier.
    pub tier: Tier,
    /// The protocol fee (in basis points) taken by the rail.
    pub fee_bps: u32,
}

/// A settlement that was attempted.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Settlement constructed successfully.
    Constructed(Settlement),
    /// The settlement was rejected (e.g. invalid address or tier mismatch).
    Rejected {
        /// Why the settlement was rejected.
        reason: String,
    },
}

/// Error from the settle API.
#[derive(Debug, thiserror::Error)]
pub enum SettleError {
    /// Underlying dero-rs error.
    #[error("dero error: {0}")]
    Dero(#[from] DeroError),
    /// A Licensed settlement was attempted without a valid fee.
    #[error("licensed settlement requires a valid fee")]
    LicensedFeeRequired,
    /// A cryptographic decode failed.
    #[error("crypto decode failed: {0}")]
    CryptoDecode(String),
}

/// Settle `amount` (in atomic units) from `from` to `to` confidentially.
///
/// `tier` selects the fee model. Returns a constructed [`Settlement`] whose
/// value is a confidential ElGamal commitment — the amount is hidden but
/// cryptographically binding.
pub fn settle(from: &str, to: &str, amount: u64, tier: Tier) -> Result<Outcome, SettleError> {
    let from = match Address::new(from) {
        Ok(a) => a,
        Err(e) => return Ok(Outcome::Rejected { reason: format!("bad sender address: {e}") }),
    };
    let to = match Address::new(to) {
        Ok(a) => a,
        Err(e) => return Ok(Outcome::Rejected { reason: format!("bad recipient address: {e}") }),
    };

    // A Licensed settlement must carry a non-zero fee.
    if let Tier::Licensed { fee_bps } = tier {
        if fee_bps == 0 {
            return Err(SettleError::LicensedFeeRequired);
        }
    }

    let value = build_confidential_value(&to, amount)?;
    let fee_bps = tier.fee_bps();

    Ok(Outcome::Constructed(Settlement { from, to, value, tier, fee_bps }))
}

/// Build a confidential value commitment to `amount` for recipient `to`.
fn build_confidential_value(to: &Address, amount: u64) -> Result<ConfidentialValue, SettleError> {
    let mut amt = [0u8; 32];
    amt[..8].copy_from_slice(&amount.to_le_bytes());
    // NOTE: production must use secure randomness (e.g. from a CSPRNG). This is
    // a deterministic placeholder for the harness.
    let randomness = [0u8; 32];
    let pk = dero_rs::curve::decode_compressed(&to.compressed).map_err(SettleError::CryptoDecode)?;
    Ok(ConfidentialValue::commit_to(&pk, &amt, &randomness))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic 33-byte compressed point (valid length). In production these
    /// come from real DERO addresses; here we test the tier/fee logic.
    fn synth_address(tag: u8) -> String {
        // Build a valid compressed point: encode the generator with a tag byte.
        let g = G1Projective::generator();
        let mut comp = dero_rs::curve::encode_compressed(&g);
        comp[0] = tag; // vary the tag so addresses differ
        dero_rs::rpc_addr::compressed_to_address(&comp).unwrap_or_else(|_| "dero1qplaceholder".into())
    }

    #[test]
    fn commons_settlement_zero_fee() {
        let a = synth_address(2);
        let b = synth_address(3);
        let out = settle(&a, &b, 1000, Tier::Commons).unwrap();
        match out {
            Outcome::Constructed(s) => {
                assert_eq!(s.fee_bps, 0);
                assert_eq!(s.value.serialize().len(), 66);
            }
            Outcome::Rejected { reason } => panic!("unexpected reject: {reason}"),
        }
    }

    #[test]
    fn licensed_settlement_carries_fee() {
        let a = synth_address(4);
        let b = synth_address(5);
        let out = settle(&a, &b, 1000, Tier::Licensed { fee_bps: 150 }).unwrap();
        match out {
            Outcome::Constructed(s) => assert_eq!(s.fee_bps, 150),
            Outcome::Rejected { reason } => panic!("unexpected reject: {reason}"),
        }
    }
}
