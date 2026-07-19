//! Pair Equality Predicate Alkane Contract
//!
//! A secure and efficient predicate alkane contract that enforces the quantities
//! of alkanes sent to it in a two-party trade. This contract follows current best
//! practices and security patterns.

use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, message::MessageDispatch};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use alkanes_support::id::AlkaneId;
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;

/// EqualityPredicate trait provides common predicate functionality
pub trait EqualityPredicate: AlkaneResponder {
    /// Observe initialization to prevent multiple initializations
    fn observe_initialization(&self) -> Result<()> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().len() == 0 {
            pointer.set_value::<u8>(0x01);
            Ok(())
        } else {
            Err(anyhow!("already initialized"))
        }
    }
}

/// EqualityPredicate implements a predicate contract that enforces equality in a two-party trade
#[derive(Default)]
pub struct EqualityPredicateAlkane(());

impl EqualityPredicate for EqualityPredicateAlkane {}

/// Message enum for opcode-based dispatch
#[derive(MessageDispatch)]
enum EqualityPredicateAlkaneMessage {
    /// Initialize the contract
    #[opcode(0)]
    Initialize,
    
    /// Filter alkanes based on sequence and amount
    #[opcode(7)]
    Filter { 
        sequence_left: u128, 
        amount_left: u128, 
        sequence_right: u128, 
        amount_right: u128 
    },
}

impl EqualityPredicateAlkane {
    fn initialize(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        EqualityPredicate::observe_initialization(self)
            .map_err(|_| anyhow!("Contract already initialized"))?;
        Ok(response)
    }
    
    // Make the filter method public for testing
    pub fn filter(&self, sequence_left: u128, amount_left: u128, sequence_right: u128, amount_right: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let incoming_alkanes = &context.incoming_alkanes;
        if incoming_alkanes.0.len() != 2 {
            return Err(anyhow!("EqualityPredicate only handles 2 alkanes"));
        }
        
        let left_id = AlkaneId { block: 2, tx: sequence_left };
        let right_id = AlkaneId { block: 2, tx: sequence_right };
        
        if incoming_alkanes.0[0].id == left_id && 
           incoming_alkanes.0[0].value == amount_left && 
           incoming_alkanes.0[1].id == right_id && 
           incoming_alkanes.0[1].value == amount_right {
            Ok(CallResponse::forward(incoming_alkanes))
        } else {
            Err(anyhow!("EqualityPredicate failed: alkanes do not match required parameters"))
        }
    }
}

impl AlkaneResponder for EqualityPredicateAlkane {}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for EqualityPredicateAlkane {
        type Message = EqualityPredicateAlkaneMessage;
    }
}