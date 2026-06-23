use crate::model::Transaction;
use crate::workload::{WorkloadConfig, WorkloadGenerator, WorkloadScenario, initial_state};

pub struct NoConflictWorkload;

impl WorkloadGenerator for NoConflictWorkload {
    fn name(&self) -> &'static str {
        "no-conflict"
    }

    fn generate(&self, config: &WorkloadConfig) -> WorkloadScenario {
        let accounts = config.accounts.max(config.block_size * 2 + 2);
        let mut block = Vec::with_capacity(config.block_size);
        for i in 0..config.block_size {
            let sender = (i * 2) as u64;
            let receiver = (i * 2 + 1) as u64;
            block.push(Transaction::public_transfer(
                i as u64,
                i,
                sender,
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
