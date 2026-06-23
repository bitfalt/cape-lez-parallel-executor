use crate::model::{StateDiff, Transaction, TxExecution, TxKind, TxOutcome};
use crate::state::State;
use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

pub fn execute_transaction(tx: &Transaction, state: &State) -> TxExecution {
    let start = Instant::now();
    burn_compute(tx.compute_cost, tx.id);

    let read_keys = tx.read_keys();
    let write_keys = tx.write_keys();
    let read_versions = read_keys
        .iter()
        .map(|key| (key.clone(), state.version_of(key)))
        .collect::<HashMap<_, _>>();

    let (outcome, diff) = match tx.kind {
        TxKind::PublicTransfer => execute_public_transfer(tx, state),
        TxKind::PrivateTransfer => execute_private_transfer(tx, state),
        TxKind::ProgramCall => execute_program_call(tx, state),
        TxKind::ProgramDeployment => execute_program_deployment(tx, state),
        TxKind::System => execute_system(tx),
    };

    TxExecution {
        outcome,
        diff,
        read_versions,
        read_keys,
        write_keys,
        execution_ns: start.elapsed().as_nanos(),
    }
}

pub fn read_versions_still_valid(execution: &TxExecution, state: &State) -> bool {
    execution
        .read_versions
        .iter()
        .all(|(key, version)| state.version_of(key) == *version)
}

fn execute_public_transfer(tx: &Transaction, state: &State) -> (TxOutcome, StateDiff) {
    let Some(receiver) = tx.receiver else {
        return (
            TxOutcome::rejected(tx, "missing receiver"),
            StateDiff::default(),
        );
    };
    if tx.amount <= 0 {
        return (
            TxOutcome::rejected(tx, "transfer amount must be positive"),
            StateDiff::default(),
        );
    }
    if state.balance_of(tx.signer) < tx.amount {
        return (
            TxOutcome::rejected(tx, "insufficient balance"),
            StateDiff::default(),
        );
    }

    let mut diff = StateDiff {
        written_keys: tx.write_keys(),
        ..StateDiff::default()
    };
    *diff.balance_delta.entry(tx.signer).or_insert(0) -= tx.amount;
    *diff.balance_delta.entry(receiver).or_insert(0) += tx.amount;
    diff.nonce_delta.insert(tx.signer, 1);
    (TxOutcome::accepted(tx), diff)
}

fn execute_private_transfer(tx: &Transaction, state: &State) -> (TxOutcome, StateDiff) {
    if let Some(used) = tx
        .consumed_nullifiers
        .iter()
        .find(|nullifier| state.nullifiers.contains(nullifier))
    {
        return (
            TxOutcome::rejected(tx, format!("nullifier {used} already used")),
            StateDiff::default(),
        );
    }

    let mut diff = StateDiff {
        nullifiers_to_add: tx.consumed_nullifiers.clone(),
        commitments_to_append: tx.produced_commitments.clone(),
        written_keys: tx.write_keys(),
        ..StateDiff::default()
    };
    diff.nonce_delta.insert(tx.signer, 1);
    (TxOutcome::accepted(tx), diff)
}

fn execute_program_call(tx: &Transaction, state: &State) -> (TxOutcome, StateDiff) {
    let Some(program_id) = tx.program_id else {
        return (
            TxOutcome::rejected(tx, "missing program id"),
            StateDiff::default(),
        );
    };
    let Some(key) = tx.program_key else {
        return (
            TxOutcome::rejected(tx, "missing program key"),
            StateDiff::default(),
        );
    };
    if !state.programs.contains_key(&program_id) {
        return (
            TxOutcome::rejected(tx, "program is not deployed"),
            StateDiff::default(),
        );
    }
    let mut diff = StateDiff {
        program_writes: vec![(program_id, key, tx.amount)],
        written_keys: tx.write_keys(),
        ..StateDiff::default()
    };
    diff.nonce_delta.insert(tx.signer, 1);
    (TxOutcome::accepted(tx), diff)
}

fn execute_program_deployment(tx: &Transaction, state: &State) -> (TxOutcome, StateDiff) {
    let Some(program_id) = tx.program_id else {
        return (
            TxOutcome::rejected(tx, "missing program id"),
            StateDiff::default(),
        );
    };
    if state.programs.contains_key(&program_id) {
        return (
            TxOutcome::rejected(tx, "program already deployed"),
            StateDiff::default(),
        );
    }
    let diff = StateDiff {
        programs_to_deploy: vec![program_id],
        written_keys: tx.write_keys(),
        ..StateDiff::default()
    };
    (TxOutcome::accepted(tx), diff)
}

fn execute_system(tx: &Transaction) -> (TxOutcome, StateDiff) {
    let diff = StateDiff {
        system_clock_delta: 1,
        written_keys: tx.write_keys(),
        ..StateDiff::default()
    };
    (TxOutcome::accepted(tx), diff)
}

fn burn_compute(iterations: u64, seed: u64) {
    let mut value = seed ^ 0x9e37_79b9_7f4a_7c15;
    for i in 0..iterations {
        value = value
            .wrapping_mul(6364136223846793005)
            .wrapping_add(i ^ 1442695040888963407);
    }
    black_box(value);
}
