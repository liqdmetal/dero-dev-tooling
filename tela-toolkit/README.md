# tela-toolkit

A scaffolding generator for **TELA** dApps — "create-react-app for DERO's
decentralized web." TELA is DERO's decentralized-web standard: dApp files and
data live on-chain, encrypted, and execute locally. This toolkit starts you from
a working skeleton instead of a blank page.

Part of the [`dero-dev-tooling`](../) workspace.

## Usage

```bash
cargo run -p tela-toolkit -- --name demo                 # -> ./demo
cargo run -p tela-toolkit -- --name demo --out ./where   # explicit output dir
cargo run -p tela-toolkit -- --name demo --force         # overwrite generated files
```

Generated tree:

| Path | Contents |
|---|---|
| `tela.json` | app manifest — `name`, `version`, `entry`, `engine: "tela"`, `encrypted: ["data/"]` |
| `index.html` | entry point |
| `app.js` | dApp logic stub |
| `encrypted/data/` | placeholder for on-chain encrypted data |

## Library API

| Item | Purpose |
|---|---|
| `scaffold(dir, name)` | generate; refuses to overwrite existing generated files |
| `scaffold_with_options(dir, name, force)` | same, with explicit overwrite consent |
| `Scaffold` | `root` plus the `files` written (implements `Display`) |
| `ScaffoldError` | `InvalidName`, `AlreadyExists`, `Io`, `Manifest` |
| `cli::run()` | the `clap` CLI entrypoint (`--name`, `--out`, `--force`) |

## Safety properties

- Names are validated as ASCII alphanumeric plus `-` and `_` **before** any
  filesystem work, so a name cannot escape the output directory.
- Generated files are written to a temp path and then renamed in, and existing
  files are refused unless `force` is set — the generator is non-destructive by
  default.
- The manifest is serialized with `serde_json`, never hand-rolled JSON.

## Notes

- `#![forbid(unsafe_code)]`.
