# FAQ — dero-dev-tooling

**What is this?**
A Rust workspace that makes DERO's confidential crypto consumable by
ordinary developers: `dero-rs` (addresses, ElGamal confidential values,
settlement construction), `dvm-dev` (DVM v151 contract dev environment),
`tela-toolkit` (TELA dApp generator), and `settle-sdk` (typed RPC client +
confidential-settlement API).

**Do I need DERO experience to use it?**
No group arithmetic, Pedersen commitments, or bulletproof internals by
hand — that is the point. Read an address, commit an amount, parse and run
a contract, scaffold a dApp.

**Is the crypto safe to use in production?**
The underlying crypto is the diff-verified Rust implementation of DERO's
confidential cryptography (`derohe-rs`), differential-tested against the Go
reference. The SDK itself constructs confidential values only — it does not
sign, broadcast, or validate mainnet transactions. Broadcasting is an
explicit, caller-gated operation outside this workspace.

**Why does the build need derohe-rs checked out beside the workspace?**
`dero-rs` consumes `../derohe-rs/crypto` as a path dependency, so trust
stays pinned to a specific reviewed revision (the CI pins `ef200ca39ca`,
a full SHA inside derohe-rs's public history — no tags or moving
branches).

**Which DVM version is targeted?**
DVM v151 only — the AS-IS mainnet set (36 intrinsics, 10M gas / 11k evals,
no group arithmetic). The v9/`verify_proof`/K0 track is Go track-B and is
NOT mirrored here.

**What does the two-tier model in settle-sdk mean?**
Commons (natural persons) settle at zero bps; licensed tiers pay a fee-bps
on settlement. The SDK carries the model; enforcement is out-of-band.

**What are the license terms?**
The workspace wraps `derohe-rs`, whose Rust core (including the `crypto`
crate used here) is BSD-3-Clause; only the Go `oracle/` harness keeps the
DERO Research License. Derivative work must respect those terms (see
`derohe-rs/LICENSE` and `derohe-rs/NOTICE`).
