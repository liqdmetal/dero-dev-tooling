# dero-rs

Ergonomic Rust SDK for building on DERO's confidential settlement. Wraps
`dero-crypto` — the diff-verified Rust implementation of DERO's consensus crypto
(ElGamal, bulletproofs, Pedersen, DVM, transactions).

The point: build a confidential settlement app **without touching raw group
arithmetic**.

Part of the [`dero-dev-tooling`](../) workspace.

## What it gives you

| Item | Purpose |
|---|---|
| `Address` | bech32 DERO address ⇄ 33-byte compressed public key |
| `ConfidentialValue` | ElGamal value commitment — additively homomorphic, 66-byte wire form |
| `build_settlement(from, to, amount)` | high-level builder: address parse → commit under the recipient key with CSPRNG blinding |
| `Error` | typed SDK error (`InvalidAddress`, `Crypto`) |

Plus re-exports of the whole `dero-crypto` surface — `block`, `bulletproof`,
`chain`, `curve`, `dvm`, `elgamal`, `hash_to_point`, `innerproduct`, `keccak`,
`pedersen`, `rpc_addr`, `rpc_args`, `statement`, `transaction` — and the
`arkworks` BN254 types (`G1Projective`, `G1Affine`, `PrimeGroup`, `BigInt`,
`PrimeField`) for code that does want the group math.

## Example

```rust
use dero_rs::{Address, ConfidentialValue};

// Real addresses come from the chain; this is a synthetic compressed key.
let to = Address::from_compressed(&[2u8; 33]).expect("valid compressed key");
let pk = dero_rs::curve::decode_compressed(to.compressed_key()).expect("decode");

let mut amt = [0u8; 32];
amt[..8].copy_from_slice(&1000u64.to_le_bytes());
let randomness = [7u8; 32]; // production: use secure randomness

let value = ConfidentialValue::commit_to(&pk, &amt, &randomness);
assert_eq!(value.serialize().len(), 66);
```

Runnable version: `cargo run -p dero-rs --example settlement`

## Security boundary

`build_settlement` uses OS-provided CSPRNG randomness (`getrandom`) for the
blinding scalar. It **constructs** a confidential value; it does not sign,
broadcast, or assert that anything is valid on mainnet. Broadcasting is an
explicit, caller-gated step outside this crate.

## Notes

- DERO is **account-model ElGamal**, not RingCT/UTXO. A confidential value is an
  ElGamal commitment with homomorphic add/multiply.
- `dero-crypto` returns `Result<_, String>`. This crate maps those into typed
  `Error` variants — don't `?` a `String` into a `thiserror` error.
- `G1Projective::generator()` needs `ark_ec::PrimeGroup` in scope (not
  `ark_ec::Group`). Re-exporting ark types requires them as direct dependencies.
- `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`.
- Path dependency: `dero-crypto = { path = "../../derohe-rs/crypto" }`.
