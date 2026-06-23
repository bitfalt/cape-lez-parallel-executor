use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunMetrics {
    pub run_id: String,
    pub seed: u64,
    pub executor: String,
    pub workload: String,
    pub block_size: usize,
    pub num_accounts: usize,
    pub threads: usize,
    pub compute_cost: u64,
    pub skew: f64,
    pub elapsed_ns: u128,
    pub throughput_tps: f64,
    pub scheduling_ns: u128,
    pub execution_ns: u128,
    pub validation_ns: u128,
    pub commit_ns: u128,
    pub conflicts_detected: u64,
    pub reexecutions: u64,
    pub accepted_txs: usize,
    pub rejected_txs: usize,
    pub final_state_hash: String,
    pub correct_vs_sequential: Option<bool>,
}

impl RunMetrics {
    pub fn finalize_timing(&mut self) {
        if self.elapsed_ns > 0 {
            self.throughput_tps =
                self.block_size as f64 / (self.elapsed_ns as f64 / 1_000_000_000.0);
        }
    }
}
