use crate::model::Transaction;
use crate::workload::{WorkloadConfig, WorkloadGenerator, WorkloadScenario, initial_state};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

pub struct HotspotWorkload;

impl WorkloadGenerator for HotspotWorkload {
    fn name(&self) -> &'static str {
        "hotspot"
    }

    fn generate(&self, config: &WorkloadConfig) -> WorkloadScenario {
        let accounts = config.accounts.max(2);
        let mut rng = StdRng::seed_from_u64(config.seed);
        let mut block = Vec::with_capacity(config.block_size);
        for i in 0..config.block_size {
            let receiver = rng.random_range(1..accounts as u64);
            block.push(Transaction::public_transfer(
                i as u64,
                i,
                0,
                receiver,
                1,
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
