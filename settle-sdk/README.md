# settle-sdk

A confidential-settlement API for ordinary developers, built on
[`dero-rs`](../dero-rs). Call `settle` and get a confidential DERO settlement —
no ElGamal, bulletproofs, or group arithmetic in your code.

This is also the **on-ramp to the RelayOS rail**: every settlement carries a
tier, mirroring RelayOS's two-tier model.

Part of the [`dero-dev-tooling`](../) workspace.

## Two-tier model

| Tier | Who | Fee |
|---|---|---|
| `Tier::Commons` | natural persons only | `$0` |
| `Tier::Licensed { fee_bps }` | legal entities | per-use protocol fee, `1..=10_000` bps |

Build a licensed tier with `Tier::licensed(fee_bps)`, which validates the range
up front — `0` and anything above `Tier::MAX_FEE_BPS` (`10_000`) are rejected.
`settle` re-checks a directly-constructed `Licensed` variant, so bypassing the
constructor does not bypass the rule.

## API

| Item | Purpose |
|---|---|
| `settle(from, to, amount, tier)` | parse both addresses, build the confidential commitment, attach `tier` + `fee_bps` |
| `Settlement` | `from`, `to`, `value` (ElGamal commitment), `tier`, `fee_bps` |
| `Outcome` | `Constructed(Box<Settlement>)` or `Rejected { reason }` — bad addresses are *reported*, not thrown |
| `SettleError` | `Dero`, `LicensedFeeRequired`, `InvalidFeeBps`, `CryptoDecode`, `Randomness` |

### Modules

- **`rpc`** — typed DERO daemon RPC client for **preparing** settlements:
  `Client::new(host, port)`, `with_timeout`, `with_max_response_bytes`,
  `get_balance(address)`, `call(method, params)`. Parameters are marshaled with
  dero-crypto's `Arguments` so they encode exactly as the daemon expects.
  Read/prepare only — **broadcast is intentionally left to the caller.**
- **`relay`** — the RelayOS-aligned request shape. `SettlementLegRequest` is
  emitted so the Rail's existing verifier (`SettlementLeg.verify_leg`) can consume
  it directly. An `amount > 0` leg **requires** `authorized_evidence_keys`;
  `EntityClass` separates commons (natural persons) from licensed (legal
  entities). The fee attaches to the settlement, not to the node.

## Example

```rust
use settle_sdk::{settle, SettleError, Outcome, Tier};

let out = settle(sender, recipient, 1_000, Tier::Commons)?;
match out {
    Outcome::Constructed(s) => assert_eq!(s.fee_bps, 0),
    Outcome::Rejected { reason } => eprintln!("rejected: {reason}"),
}
```

## Security boundary

Commitments are blinded with OS-provided CSPRNG randomness (`getrandom`). This
crate constructs settlements; it does not sign, broadcast, or assert mainnet
validity. Signing and broadcast are the caller's gated step.

## Notes

- `dero-crypto` returns `Result<_, String>`; those are mapped onto `SettleError`
  variants rather than propagated as strings.
- `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`.
