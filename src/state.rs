use crate::hash::hash_state;
use crate::model::{
    Account, AccountId, Amount, Nullifier, ProgramId, ProgramState, StateDiff, StateKey, Version,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub accounts: HashMap<AccountId, Account>,
    pub nonces: HashMap<AccountId, u64>,
    pub nullifiers: HashSet<Nullifier>,
    pub commitments: Vec<u64>,
    pub programs: HashMap<ProgramId, ProgramState>,
    pub system_clock: u64,
    pub global_config: u64,
    pub versions: HashMap<StateKey, Version>,
}

impl State {
    pub fn set_account_balance(&mut self, account: AccountId, balance: Amount) {
        self.accounts.insert(account, Account { balance });
    }

    pub fn balance_of(&self, account: AccountId) -> Amount {
        self.accounts.get(&account).map(|a| a.balance).unwrap_or(0)
    }

    pub fn set_nonce(&mut self, account: AccountId, nonce: u64) {
        self.nonces.insert(account, nonce);
    }

    pub fn nonce_of(&self, account: AccountId) -> u64 {
        *self.nonces.get(&account).unwrap_or(&0)
    }

    pub fn version_of(&self, key: &StateKey) -> Version {
        *self.versions.get(key).unwrap_or(&0)
    }

    pub fn bump_version(&mut self, key: StateKey) {
        let entry = self.versions.entry(key).or_insert(0);
        *entry += 1;
    }

    pub fn state_hash(&self) -> String {
        hash_state(self)
    }

    pub fn apply_diff(&mut self, diff: &StateDiff) -> Result<(), String> {
        for (account, delta) in &diff.balance_delta {
            let current = self.balance_of(*account);
            self.set_account_balance(*account, current + *delta);
        }

        for (account, delta) in &diff.nonce_delta {
            let current = self.nonce_of(*account);
            self.set_nonce(*account, current + *delta);
        }

        for nullifier in &diff.nullifiers_to_add {
            self.nullifiers.insert(*nullifier);
        }

        self.commitments
            .extend(diff.commitments_to_append.iter().copied());

        for program_id in &diff.programs_to_deploy {
            self.programs.entry(*program_id).or_insert(ProgramState {
                storage: HashMap::new(),
                deployed_at: self.system_clock,
            });
        }

        for (program_id, key, delta) in &diff.program_writes {
            let program = self.programs.entry(*program_id).or_insert(ProgramState {
                storage: HashMap::new(),
                deployed_at: self.system_clock,
            });
            let value = program.storage.entry(*key).or_insert(0);
            *value += *delta;
        }

        self.system_clock += diff.system_clock_delta;

        for key in &diff.written_keys {
            self.bump_version(key.clone());
        }

        Ok(())
    }
}
