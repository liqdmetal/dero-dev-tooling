//! DVM contract execution harness — run a parsed contract's functions against a
//! simulated chain context (gas limits, storage) without touching mainnet.
//!
//! This is the "run" half of dvm-dev: after `parse_contract`, callers can
//! execute a function and inspect the return value, gas used, and storage —
//! iterate on contract logic locally.

use dero_rs::dvm::{run_smart_contract, BlockchainInput, SharedState, TXStorage, Variable};
use std::collections::HashMap;

/// A configurable execution context for a DVM contract.
#[derive(Debug, Clone)]
pub struct ContractRunner {
    /// Compute gas limit (0 = unlimited).
    pub gas_compute_limit: i64,
    /// Storage gas limit (0 = unlimited).
    pub gas_store_limit: i64,
    /// Simulated chain inputs (scid/blid/txid).
    pub chain: BlockchainInput,
    /// Storage backend.
    store: TXStorage,
}

impl Default for ContractRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractRunner {
    /// Create a runner with DVM v151 default limits (10M compute, 11k eval budget).
    pub fn new() -> Self {
        Self {
            gas_compute_limit: 10_000_000,
            gas_store_limit: 64_000_000,
            chain: BlockchainInput {
                scid: [0u8; 32],
                blid: [0u8; 32],
                txid: [0u8; 32],
                signer: String::new(),
                bl_height: 0,
                bl_topoheight: 0,
                bl_timestamp: 0,
            },
            store: TXStorage {
                scid: [0u8; 32],
                raw_keys: HashMap::new(),
                transfers: HashMap::new(),
                state: None,
            },
        }
    }

    /// Execute `entry_point` of a contract with string params.
    ///
    /// Returns the function's return value, or an error if the DVM rejects it.
    pub fn execute(
        &mut self,
        contract: &crate::Contract,
        entry_point: &str,
        params: &HashMap<String, String>,
    ) -> Result<ExecResult, String> {
        let mut state = SharedState {
            scid_zero: [0u8; 32],
            scid_self: [0u8; 32],
            gas_compute_used: 0,
            gas_compute_limit: self.gas_compute_limit,
            gas_compute_check: self.gas_compute_limit != 0,
            gas_store_used: 0,
            gas_store_limit: self.gas_store_limit,
            gas_store_check: self.gas_store_limit != 0,
            chain_inputs: self.chain.clone(),
            assets: HashMap::new(),
            assets_transfer: HashMap::new(),
            ram_store: HashMap::new(),
            monitor_recursion: 0,
            monitor_lines_interpreted: 0,
            monitor_ops: 0,
            store: Box::new(self.store.clone()),
            rnd: dero_rs::dvm::initialize_rnd(&[0u8; 32], &[0u8; 32], &[0u8; 32]),
        };

        // The interpreter's storage helpers charge gas through this back-reference.
        // Keep the same initialized snapshot used by dero-crypto's reference harness.
        state.store.state = Some(state.clone());
        let ret = run_smart_contract(&contract.parsed, entry_point, &mut state, params)?;
        self.store = *state.store;
        Ok(ExecResult {
            value: ret,
            gas_compute_used: state.gas_compute_used,
            gas_store_used: state.gas_store_used,
        })
    }
}

/// The result of executing a contract function.
#[derive(Debug, Clone)]
pub struct ExecResult {
    /// The function's return value.
    pub value: Variable,
    /// Compute gas consumed.
    pub gas_compute_used: i64,
    /// Storage gas consumed.
    pub gas_store_used: i64,
}
