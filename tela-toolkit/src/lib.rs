//! # tela-toolkit
//!
//! A scaffolding generator for TELA dApps — "create-react-app for DERO's
//! decentralized web." Generates a runnable TELA app skeleton with the
//! encrypted-storage + local-execution model TELA provides.
//!
//! TELA = DERO's Decentralized Web standard: dApp files and data live
//! on-chain, encrypted, and execute locally. This toolkit scaffolds that
//! structure so a developer starts from a working skeleton instead of a
//! blank page.
#![forbid(unsafe_code)]

use std::fmt;
use std::path::{Path, PathBuf};

/// A generated TELA app scaffold on disk.
#[derive(Debug, Clone)]
pub struct Scaffold {
    /// Root directory of the scaffold.
    pub root: PathBuf,
    /// Files written.
    pub files: Vec<PathBuf>,
}

/// Error writing a scaffold.
#[derive(Debug, thiserror::Error)]
pub enum ScaffoldError {
    /// I/O failure.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl fmt::Display for Scaffold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Scaffolded TELA dApp at {}", self.root.display())?;
        for file in &self.files {
            writeln!(f, "  - {}", file.strip_prefix(&self.root).unwrap_or(file).display())?;
        }
        Ok(())
    }
}

/// Generate a TELA dApp scaffold under `dir`.
///
/// Creates:
///   - `tela.json`   — the TELA app manifest (name, version, entry)
///   - `index.html`  — entry point
///   - `app.js`      — the dApp logic stub
///   - `encrypted/`  — placeholder for on-chain encrypted data
pub fn scaffold(dir: &Path, name: &str) -> Result<Scaffold, ScaffoldError> {
    std::fs::create_dir_all(dir)?;
    let mut files = Vec::new();

    let manifest = format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "entry": "index.html",
  "engine": "tela",
  "encrypted": ["data/"]
}}
"#
    );
    let manifest_path = dir.join("tela.json");
    std::fs::write(&manifest_path, manifest)?;
    files.push(manifest_path);

    let index = r#"<!doctype html>
<html>
<head><meta charset="utf-8"><title>TELA dApp</title></head>
<body>
  <h1>TELA dApp</h1>
  <div id="app"></div>
  <script src="app.js"></script>
</body>
</html>
"#;
    let index_path = dir.join("index.html");
    std::fs::write(&index_path, index)?;
    files.push(index_path);

    let app = r#"// TELA dApp logic stub — encrypted storage + local execution.
// Data written here is encrypted and stored on-chain (TELA model).
const app = document.getElementById("app");
app.textContent = "Hello from a TELA dApp.";
"#;
    let app_path = dir.join("app.js");
    std::fs::write(&app_path, app)?;
    files.push(app_path);

    let data_dir = dir.join("encrypted").join("data");
    std::fs::create_dir_all(&data_dir)?;

    Ok(Scaffold { root: dir.to_path_buf(), files })
}

/// CLI entrypoint.
pub mod cli {
    use crate::scaffold;
    use clap::Parser;

    /// Scaffold a new TELA dApp.
    #[derive(Parser)]
    #[command(name = "tela", version, about = "Scaffold a TELA dApp")]
    pub struct Args {
        /// Name of the app.
        #[arg(long, short)]
        pub name: String,
        /// Output directory (defaults to ./name).
        #[arg(long, short)]
        pub out: Option<String>,
    }

    /// Run the CLI.
    pub fn run() -> std::process::ExitCode {
        let args = Args::parse();
        let dir = args.out.clone().unwrap_or_else(|| format!("./{}", args.name));
        match scaffold(std::path::Path::new(&dir), &args.name) {
            Ok(s) => {
                println!("{s}");
                std::process::ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::ExitCode::FAILURE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffolds_app() {
        let dir = std::env::temp_dir().join("tela_test_app");
        let _ = std::fs::remove_dir_all(&dir);
        let s = scaffold(&dir, "demo").expect("scaffold ok");
        assert!(dir.join("tela.json").exists());
        assert!(dir.join("index.html").exists());
        assert!(dir.join("app.js").exists());
        assert!(!s.files.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
