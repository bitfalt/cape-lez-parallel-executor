use crate::model::Transaction;
use crate::workload::{WorkloadConfig, WorkloadGenerator, WorkloadScenario, initial_state};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

pub struct TokenTransferWorkload;

impl WorkloadGenerator for TokenTransferWorkload {
    fn name(&self) -> &'static str {
        "token-transfer"
    }

    fn generate(&self, config: &WorkloadConfig) -> WorkloadScenario {
        let accounts = config.accounts.max(2);
        let mut rng = StdRng::seed_from_u64(config.seed);
        let mut block = Vec::with_capacity(config.block_size);
        for i in 0..config.block_size {
            let sender = rng.random_range(0..accounts as u64);
            let mut receiver = rng.random_range(0..accounts as u64);
            if receiver == sender {
                receiver = (receiver + 1) % accounts as u64;
            }
            let amount = rng.random_range(1..25);
            block.push(Transaction::public_transfer(
                i as u64,
                i,
                sender,
                receiver,
                amount,
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
