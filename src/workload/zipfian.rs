use crate::model::Transaction;
use crate::workload::{WorkloadConfig, WorkloadGenerator, WorkloadScenario, initial_state};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

pub struct ZipfianWorkload;

impl WorkloadGenerator for ZipfianWorkload {
    fn name(&self) -> &'static str {
        "zipfian"
    }

    fn generate(&self, config: &WorkloadConfig) -> WorkloadScenario {
        let accounts = config.accounts.max(2);
        let mut rng = StdRng::seed_from_u64(config.seed);
        let cumulative = cumulative_zipf(accounts, config.skew.max(0.1));
        let mut block = Vec::with_capacity(config.block_size);
        for i in 0..config.block_size {
            let sender = sample_weighted(&cumulative, &mut rng);
            let mut receiver = sample_weighted(&cumulative, &mut rng);
            if receiver == sender {
                receiver = (receiver + 1) % accounts as u64;
            }
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

fn cumulative_zipf(accounts: usize, skew: f64) -> Vec<f64> {
    let mut total = 0.0;
    let mut cumulative = Vec::with_capacity(accounts);
    for rank in 1..=accounts {
        total += 1.0 / (rank as f64).powf(skew);
        cumulative.push(total);
    }
    cumulative
}

fn sample_weighted(cumulative: &[f64], rng: &mut StdRng) -> u64 {
    let total = *cumulative.last().unwrap_or(&1.0);
    let target = rng.random_range(0.0..total);
    cumulative
        .binary_search_by(|value| value.partial_cmp(&target).unwrap())
        .unwrap_or_else(|idx| idx) as u64
}
