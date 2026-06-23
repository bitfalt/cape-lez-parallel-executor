use crate::model::Transaction;
use crate::workload::{WorkloadConfig, WorkloadGenerator, WorkloadScenario, initial_state};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

pub struct PrivateStateWorkload;

impl WorkloadGenerator for PrivateStateWorkload {
    fn name(&self) -> &'static str {
        "private-state"
    }

    fn generate(&self, config: &WorkloadConfig) -> WorkloadScenario {
        let accounts = config.accounts.max(2);
        let mut rng = StdRng::seed_from_u64(config.seed);
        let mut block = Vec::with_capacity(config.block_size);
        let mut previous_nullifiers = Vec::new();

        for i in 0..config.block_size {
            let signer = rng.random_range(0..accounts as u64);
            let duplicate = !previous_nullifiers.is_empty()
                && rng.random::<f64>() < config.duplicate_nullifier_rate;
            let nullifier = if duplicate {
                let idx = rng.random_range(0..previous_nullifiers.len());
                previous_nullifiers[idx]
            } else {
                let value = config.seed.wrapping_mul(1_000_003).wrapping_add(i as u64);
                previous_nullifiers.push(value);
                value
            };
            let commitment = config.seed.wrapping_mul(2_000_003).wrapping_add(i as u64);
            block.push(Transaction::private_transfer(
                i as u64,
                i,
                signer,
                nullifier,
                commitment,
                config.compute_cost,
            ));
        }

        WorkloadScenario {
            name: self.name().into(),
            initial_state: initial_state(accounts),
            block,
            accounts,
        }
    }
}
