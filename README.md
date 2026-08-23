# dero-dev-tooling

DERO developer-tooling platform — built on [`derohe-rs`](https://github.com/liqdmetal/derohe-rs) (the diff-verified Rust implementation of DERO's confidential crypto).

The thesis: DERO's dev-tooling is thin, and `derohe-rs` is the rarest asset in the ecosystem (full ElGamal + bulletproofs + Pedersen + DVM + transactions, differential-tested against the Go reference). This workspace makes that crypto **consumable by ordinary developers** — and feeds the RelayOS rail's two-tier model.

## Crates

| Crate | What it does |
|---|---|
| [`dero-rs`](dero-rs) | The SDK — ergonomic `Address`, `ConfidentialValue` (ElGamal), `build_settlement()`. Re-exports the full diff-verified crypto. |
| [`dvm-dev`](dvm-dev) | DVM v151 smart-contract dev environment — parse, analyze, execute against a simulated chain (hardhat-for-DERO). |
| [`tela-toolkit`](tela-toolkit) | TELA dApp generator CLI (create-react-app for DERO's decentralized web). |
| [`settle-sdk`](settle-sdk) | Confidential-settlement API + typed DERO RPC client. Carries RelayOS's two-tier model (Commons $0 / Licensed fee bps). |

## Building & testing

```bash
cargo build          # whole workspace
cargo test           # all crates
cargo run -p tela-toolkit -- --name demo   # scaffold a TELA dApp
```

## DVM scope

`dero-rs`/`dvm-dev` target **DVM v151** only — the AS-IS mainnet set (36 intrinsics, 10M gas / 11k evals, no group arithmetic). The v9/`verify_proof`/K0 track is Go track-B, NOT mirrored here.

## Licensing

The workspace wraps `derohe-rs`, which is under the DERO Research License. Derivative work must respect that license (see `derohe-rs/LICENSE`).
