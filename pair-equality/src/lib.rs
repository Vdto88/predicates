//! Pair Equality Predicate Alkane Contract
//!
//! A secure and efficient predicate alkane contract that enforces the quantities
//! of alkanes sent to it in a two-party trade.
//!
//! ABI is declared in `contract.wit` + `alkanes.toml` (opcode 0 = initialize,
//! opcode 7 = filter); dispatch/entry points are generated at build time by
//! `alkanes-wit-build` (see build.rs) — no hand-rolled opcode enum.

// Include the generated code from WIT codegen
#[allow(unused_imports, dead_code, clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::id::AlkaneId;
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};

use generated::PairEqualityInterface;

/// EqualityPredicate implements a predicate contract that enforces equality
/// in a two-party trade. The struct name must match `[contract] name` in
/// alkanes.toml — the generated entry points reference `super::PairEquality`.
#[derive(Default)]
pub struct PairEquality(());

/// Back-compat alias for the pre-WIT name (kept for the Rust test target).
pub type EqualityPredicateAlkane = PairEquality;

impl PairEquality {
    /// Filter logic, kept as a public inherent method for testing.
    /// Error strings are LOAD-BEARING: the subfrost-app devnet e2e suite
    /// asserts on them — do not rephrase.
    pub fn filter(
        &self,
        sequence_left: u128,
        amount_left: u128,
        sequence_right: u128,
        amount_right: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let incoming_alkanes = &context.incoming_alkanes;
        if incoming_alkanes.0.len() != 2 {
            return Err(anyhow!("EqualityPredicate only handles 2 alkanes"));
        }

        let left_id = AlkaneId { block: 2, tx: sequence_left };
        let right_id = AlkaneId { block: 2, tx: sequence_right };

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

impl AlkaneResponder for PairEquality {}

impl PairEqualityInterface for PairEquality {
    fn initialize(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        self.observe_initialization()
            .map_err(|_| anyhow!("Contract already initialized"))?;
        Ok(response)
    }

    fn filter(
        &self,
        sequence_left: u128,
        amount_left: u128,
        sequence_right: u128,
        amount_right: u128,
    ) -> Result<CallResponse> {
        PairEquality::filter(self, sequence_left, amount_left, sequence_right, amount_right)
    }
}
