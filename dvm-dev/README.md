# dvm-dev

A local development environment for DERO **DVM v151** smart contracts — a
"hardhat for DERO." Parse, validate, and execute contract logic against a
simulated chain so you can iterate without touching mainnet.

Part of the [`dero-dev-tooling`](../) workspace.

## What it gives you

| Item | Purpose |
|---|---|
| `parse_contract(source)` | tokenize + parse DVM source into a `SmartContract` |
| `analyze(source)` | `ContractReport` — parse status, function count, token count, sorted function names |
| `SimContext` | in-memory `name -> u64` store for exercising pure functions |
| `runner::ContractRunner` | execute a parsed function against the DVM; returns `ExecResult` |

`ContractReport` implements `Display` for a quick human-readable dump.

### `SimContext`

`get(key)` returns `Option<u64>` and preserves the distinction between an absent
key and a stored zero. `get_or_zero(key)` collapses that difference when that is
what you want.

## Example

```rust
use dvm_dev::analyze;

let report = analyze(r#"
Function myFunc(amount Uint64) Uint64
End Function
"#)?;

println!("{report}");
```

## DVM source syntax

The parameter form is **type-then-name with no comma**:

```
Function myFunc(amount Uint64) Uint64
End Function
```

Identifier case matters: entry points must start with a capital letter. The DVM
rejects `getAnswer` where `GetAnswer` is accepted — the runner test suite asserts
this, which is how the execution path is proven connected.

## Scope

DVM **v151 only** — the AS-IS mainnet intrinsic set. The v9 / `verify_proof` / K0
track is Go track-B and is **not** in `dero-crypto`, so it is not here either.
Do not assume newer DVM features exist.

## Notes

- `SmartContract.functions` is a `HashMap<String, Function>`, **not** a `Vec` —
  iterate `.values()` to enumerate functions.
- Parse errors surface as `String`, not a typed error.
- `#![forbid(unsafe_code)]`.
