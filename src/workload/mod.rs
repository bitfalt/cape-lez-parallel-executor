pub mod hotspot;
pub mod no_conflict;
pub mod private_state;
pub mod token_transfer;
pub mod zipfian;

use crate::model::Transaction;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct WorkloadConfig {
    pub block_size: usize,
    pub accounts: usize,
    pub seed: u64,
    pub compute_cost: u64,
    pub skew: f64,
    pub duplicate_nullifier_rate: f64,
}

impl Default for WorkloadConfig {
    fn default() -> Self {
        Self {
            block_size: 1000,
            accounts: 1000,
            seed: 42,
            compute_cost: 1000,
            skew: 1.1,
            duplicate_nullifier_rate: 0.15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkloadScenario {
    pub name: String,
    pub initial_state: State,
    pub block: Vec<Transaction>,
    pub accounts: usize,
}

pub trait WorkloadGenerator {
    fn name(&self) -> &'static str;
    fn generate(&self, config: &WorkloadConfig) -> WorkloadScenario;
}

pub fn initial_state(accounts: usize) -> State {
    let mut state = State::default();
    for account in 0..accounts as u64 {
        state.set_account_balance(account, 1_000_000);
    }
    state
}

pub fn generator_by_name(name: &str) -> Box<dyn WorkloadGenerator> {
    match name {
        "no-conflict" => Box::new(no_conflict::NoConflictWorkload),
        "hotspot" => Box::new(hotspot::HotspotWorkload),
        "zipfian" => Box::new(zipfian::ZipfianWorkload),
        "token-transfer" => Box::new(token_transfer::TokenTransferWorkload),
        "private-state" => Box::new(private_state::PrivateStateWorkload),
        other => panic!("unknown workload: {other}"),
    }
}

pub fn workload_names(selection: &str) -> Vec<String> {
    if selection == "all" {
        vec![
            "no-conflict".into(),
            "hotspot".into(),
            "zipfian".into(),
            "token-transfer".into(),
            "private-state".into(),
        ]
    } else {
        selection
            .split(',')
            .map(|item| item.trim().to_string())
            .collect()
    }
}
