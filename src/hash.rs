use crate::model::{Account, ProgramState, StateKey, Version};
use crate::state::State;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize)]
struct CanonicalState {
    accounts: BTreeMap<u64, Account>,
    nonces: BTreeMap<u64, u64>,
    nullifiers: BTreeSet<u64>,
    commitments: Vec<u64>,
    programs: BTreeMap<u64, CanonicalProgramState>,
    system_clock: u64,
    global_config: u64,
    versions: Vec<(StateKey, Version)>,
}

#[derive(Serialize)]
struct CanonicalProgramState {
    storage: BTreeMap<u64, i64>,
    deployed_at: u64,
}

pub fn hash_state(state: &State) -> String {
    let accounts = state
        .accounts
        .iter()
        .map(|(key, value)| (*key, value.clone()))
        .collect();
    let nonces = state
        .nonces
        .iter()
        .map(|(key, value)| (*key, *value))
        .collect();
    let nullifiers = state.nullifiers.iter().copied().collect();
    let programs = state
        .programs
        .iter()
        .map(|(program_id, program)| (*program_id, canonical_program(program)))
        .collect();
    let mut versions: Vec<_> = state
        .versions
        .iter()
        .map(|(key, version)| (key.clone(), *version))
        .collect();
    versions.sort_by(|left, right| left.0.cmp(&right.0));

    let canonical = CanonicalState {
        accounts,
        nonces,
        nullifiers,
        commitments: state.commitments.clone(),
        programs,
        system_clock: state.system_clock,
        global_config: state.global_config,
        versions,
    };
    let bytes = serde_json::to_vec(&canonical).expect("canonical state serialization must succeed");
    blake3::hash(&bytes).to_hex().to_string()
}

fn canonical_program(program: &ProgramState) -> CanonicalProgramState {
    CanonicalProgramState {
        storage: program
            .storage
            .iter()
            .map(|(key, value)| (*key, *value))
            .collect(),
        deployed_at: program.deployed_at,
    }
}
