//! Generic Pair Equality Predicate Alkane Contract
//!
//! Same Filter semantics as `pair-equality`, but each leg carries its FULL
//! alkane id — (block, sequence) — instead of a bare sequence the contract
//! pairs with a hardcoded `block: 2`. This is what lets the predicate
//! validate pairs with a non-block-2 leg (frBTC [32:0] et al): the compact
//! contract reconstructed [32:0] as {2, 0} = DIESEL and reverted (mainnet
//! settlement d2c3291b…, block 959087 — refunded).
//!
//! ABI is declared in `contract.wit` + `alkanes.toml` (opcode 0 = initialize,
//! opcode 7 = filter — SAME opcodes as the compact contract); dispatch/entry
//! points are generated at build time by `alkanes-wit-build` (see build.rs).
//!
//! Per flex's directive (2026-07-23): "same everything though, same error
//! strings etc" — only the two extra positional block params differ.

// Include the generated code from WIT codegen
#[allow(unused_imports, dead_code, clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::id::AlkaneId;
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};

use generated::PairEqualityGenericInterface;

/// Generic equality predicate: enforces equality in a two-party trade with
/// the full (block, tx) id per leg. The struct name must match `[contract]
/// name` in alkanes.toml — the generated entry points reference
/// `super::PairEqualityGeneric`.
#[derive(Default)]
pub struct PairEqualityGeneric(());

impl PairEqualityGeneric {
    /// Filter logic, kept as a public inherent method for testing.
    /// Error strings are LOAD-BEARING (identical to `pair-equality` per
    /// flex's "same error strings" directive): the subfrost-app devnet e2e
    /// suite asserts on them — do not rephrase.
    #[allow(clippy::too_many_arguments)]
    pub fn filter(
        &self,
        block_left: u128,
        sequence_left: u128,
        amount_left: u128,
        block_right: u128,
        sequence_right: u128,
        amount_right: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let incoming_alkanes = &context.incoming_alkanes;
        if incoming_alkanes.0.len() != 2 {
            return Err(anyhow!("EqualityPredicate only handles 2 alkanes"));
        }

        let left_id = AlkaneId { block: block_left, tx: sequence_left };
        let right_id = AlkaneId { block: block_right, tx: sequence_right };

        if incoming_alkanes.0[0].id == left_id
            && incoming_alkanes.0[0].value == amount_left
            && incoming_alkanes.0[1].id == right_id
            && incoming_alkanes.0[1].value == amount_right
        {
            Ok(CallResponse::forward(incoming_alkanes))
        } else {
            Err(anyhow!("EqualityPredicate failed: alkanes do not match required parameters"))
        }
    }
}

impl AlkaneResponder for PairEqualityGeneric {}

impl PairEqualityGenericInterface for PairEqualityGeneric {
    fn initialize(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        self.observe_initialization()
            .map_err(|_| anyhow!("Contract already initialized"))?;
        Ok(response)
    }

    fn filter(
        &self,
        block_left: u128,
        sequence_left: u128,
        amount_left: u128,
        block_right: u128,
        sequence_right: u128,
        amount_right: u128,
    ) -> Result<CallResponse> {
        PairEqualityGeneric::filter(
            self,
            block_left,
            sequence_left,
            amount_left,
            block_right,
            sequence_right,
            amount_right,
        )
    }
}
