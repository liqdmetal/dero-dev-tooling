//! # dvm-dev
//!
//! A local development environment for DERO DVM (v151) smart contracts —
//! a "hardhat for DERO." Parses, validates, and reports on DVM contracts so a
//! developer can iterate on contract logic without touching mainnet.
//!
//! Target: DVM v151 only (dero-crypto's DVM is the v151 AS-IS set; v9/verify_proof
//! /K0 are Go track-B and NOT in dero-rs).
#![forbid(unsafe_code)]

use dero_rs::dvm::{parse_smart_contract, tokenize, SmartContract};
use std::fmt;

/// A parsed DVM smart contract with diagnostics.
#[derive(Debug, Clone)]
pub struct Contract {
    /// The original source.
    pub source: String,
    /// The parsed contract.
    pub parsed: SmartContract,
}

/// Diagnostics about a parsed contract.
#[derive(Debug, Default)]
pub struct ContractReport {
    /// Number of functions.
    pub function_count: usize,
    /// Number of tokens.
    pub token_count: usize,
    /// Function names.
    pub functions: Vec<String>,
    /// Whether the contract parsed cleanly.
    pub ok: bool,
}

impl fmt::Display for ContractReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Contract report:")?;
        writeln!(f, "  parse ok:      {}", self.ok)?;
        writeln!(f, "  functions:     {}", self.function_count)?;
        writeln!(f, "  tokens:        {}", self.token_count)?;
        write!(f, "  names:         {:?}", self.functions)
    }
}

/// Parse and validate a DVM smart contract from source.
///
/// Returns an error if the contract does not parse under DVM v151.
pub fn parse_contract(source: &str) -> Result<Contract, String> {
    let parsed = parse_smart_contract(source).map_err(|e| format!("parse error: {e}"))?;
    Ok(Contract { source: source.to_string(), parsed })
}

/// Produce a diagnostic report for a contract source.
pub fn analyze(source: &str) -> Result<ContractReport, String> {
    let contract = parse_contract(source)?;
    let tokens = tokenize(source);
    let functions = contract
        .parsed
        .functions
        .values()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>();
    Ok(ContractReport {
        ok: true,
        function_count: contract.parsed.functions.len(),
        token_count: tokens.len(),
        functions,
    })
}

/// A minimal in-memory execution context for exercising a contract's pure
/// functions without a live chain. (DVM v151 execution semantics live in
/// dero-crypto; this is the harness boundary.)
#[derive(Debug, Default)]
pub struct SimContext {
    /// Storage keyed by name -> u64.
    pub store: std::collections::HashMap<String, u64>,
}

impl SimContext {
    /// Create an empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a storage value.
    pub fn set(&mut self, key: &str, value: u64) {
        self.store.insert(key.to_string(), value);
    }

    /// Get a storage value (0 if absent).
    pub fn get(&self, key: &str) -> u64 {
        self.store.get(key).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_CONTRACT: &str = r#"
    Function myFunc(amount Uint64) Uint64
    End Function
    "#;

    #[test]
    fn parses_simple_contract() {
        let report = analyze(SIMPLE_CONTRACT).expect("should parse");
        assert!(report.ok);
        assert!(!report.functions.is_empty());
    }

    #[test]
    fn tokenizes_source() {
        let tokens = tokenize(SIMPLE_CONTRACT);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn sim_context_roundtrip() {
        let mut ctx = SimContext::new();
        ctx.set("balance", 42);
        assert_eq!(ctx.get("balance"), 42);
        assert_eq!(ctx.get("missing"), 0);
    }
}
