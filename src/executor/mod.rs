pub mod optimistic;
pub mod sequential;
pub mod static_scheduler;

use crate::metrics::RunMetrics;
use crate::model::{Transaction, TxOutcome, TxStatus};
use crate::state::State;

pub use optimistic::OptimisticExecutor;
pub use sequential::SequentialExecutor;
pub use static_scheduler::StaticSchedulerExecutor;

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub threads: usize,
    pub fallback_reexecution_rate: f64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            threads: 1,
            fallback_reexecution_rate: 0.75,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionReport {
    pub final_state: State,
    pub final_state_hash: String,
    pub outcomes: Vec<TxOutcome>,
    pub metrics: RunMetrics,
}

pub trait Executor {
    fn name(&self) -> &'static str;
    fn execute(
        &self,
        initial_state: &State,
        block: &[Transaction],
        config: &ExecutionConfig,
    ) -> ExecutionReport;
}

pub fn count_outcomes(outcomes: &[TxOutcome]) -> (usize, usize) {
    let accepted = outcomes
        .iter()
        .filter(|outcome| matches!(outcome.status, TxStatus::Accepted))
        .count();
    (accepted, outcomes.len() - accepted)
}

pub fn sequential_reference(
    initial_state: &State,
    block: &[Transaction],
) -> (String, Vec<TxOutcome>) {
    let report = SequentialExecutor.execute(initial_state, block, &ExecutionConfig::default());
    (report.final_state_hash, report.outcomes)
}
