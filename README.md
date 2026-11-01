# dero-dev-tooling

DERO developer-tooling platform — built on [`derohe-rs`](https://github.com/liqdmetal/derohe-rs) (the diff-verified Rust implementation of DERO's confidential crypto).

The thesis: DERO's dev-tooling is thin, and `derohe-rs` is the rarest asset in the ecosystem (full ElGamal + bulletproofs + Pedersen + DVM + transactions, differential-tested against the Go reference). This workspace makes that crypto **consumable by ordinary developers** — parse an address, commit an amount confidentially, parse and run a contract, scaffold a dApp — without writing group arithmetic, Pedersen commitments, or bulletproof internals by hand.

## Crates

| Crate | What it does |
|---|---|
| [`dero-rs`](dero-rs) | The SDK — ergonomic `Address`, `ConfidentialValue` (ElGamal), `build_settlement()`. Re-exports the full diff-verified crypto. |
| [`dvm-dev`](dvm-dev) | DVM v151 smart-contract dev environment — parse, analyze, execute against a simulated chain (hardhat-for-DERO). |
| [`tela-toolkit`](tela-toolkit) | TELA dApp generator CLI (create-react-app for DERO's decentralized web). |
| [`settle-sdk`](settle-sdk) | Confidential-settlement API + typed DERO RPC client. Carries RelayOS's two-tier model (Commons $0 / Licensed fee bps). |

## Building & testing

```bash
cargo build --workspace --all-targets
cargo test --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --all-features --no-deps
cargo run -p tela-toolkit -- --name demo   # scaffold a TELA dApp
cargo run -p tela-toolkit -- --name demo --force   # explicitly overwrite generated files
```

The same gates are available as `make ci` when GNU Make is installed. CI runs
the format, check, test, clippy, and rustdoc gates on every push and pull request.
On Windows, run the `cargo` commands directly from the table above.

CI (`.github/workflows/ci.yml`) checks out the pinned public `derohe-rs`
revision into `./derohe-rs` before Cargo runs — no credentials needed.

## Security boundary

`dero-rs::build_settlement` and `settle-sdk::settle` use OS-provided CSPRNG
randomness for commitment blinding. They construct confidential values; they do
not sign, broadcast, or claim that a transaction is valid on mainnet. The RPC
client is read/prepare-only. Broadcasting remains an explicit, caller-gated
operation outside this workspace.

The SDK currently consumes `../derohe-rs/crypto` as a path dependency. Clone
`liqdmetal/derohe-rs` beside this repository before building.

## DVM scope

`dero-rs`/`dvm-dev` target **DVM v151** only — the AS-IS mainnet set (36 intrinsics, 10M gas / 11k evals, no group arithmetic). The v9/`verify_proof`/K0 track is Go track-B, NOT mirrored here.

## Licensing

The workspace wraps `derohe-rs`, whose Rust core (including the `crypto`
crate used here) is BSD-3-Clause; only the Go `oracle/` harness keeps the
DERO Research License. Derivative work must respect those terms (see
`derohe-rs/LICENSE` and `derohe-rs/NOTICE`).

Questions: `FAQ.md`.
