use crate::executor::{ExecutionConfig, ExecutionReport, Executor, count_outcomes};
use crate::metrics::RunMetrics;
use crate::model::Transaction;
use crate::state::State;
use crate::tx::execute_transaction;
use std::time::Instant;

pub struct SequentialExecutor;

impl Executor for SequentialExecutor {
    fn name(&self) -> &'static str {
        "sequential"
    }

    fn execute(
        &self,
        initial_state: &State,
        block: &[Transaction],
        _config: &ExecutionConfig,
    ) -> ExecutionReport {
        let started = Instant::now();
        let mut execution_ns = 0;
        let mut commit_ns = 0;
        let mut state = initial_state.clone();
        let mut outcomes = Vec::with_capacity(block.len());

        for tx in block {
            let execution = execute_transaction(tx, &state);
            execution_ns += execution.execution_ns;
            let commit_started = Instant::now();
            state
                .apply_diff(&execution.diff)
                .expect("prototype diffs must be valid");
            commit_ns += commit_started.elapsed().as_nanos();
            outcomes.push(execution.outcome);
        }

        let final_state_hash = state.state_hash();
        let (accepted_txs, rejected_txs) = count_outcomes(&outcomes);
        let mut metrics = RunMetrics {
            executor: self.name().to_string(),
            block_size: block.len(),
            elapsed_ns: started.elapsed().as_nanos(),
            execution_ns,
            commit_ns,
            accepted_txs,
            rejected_txs,
            final_state_hash: final_state_hash.clone(),
            correct_vs_sequential: Some(true),
            ..RunMetrics::default()
        };
        metrics.finalize_timing();

        ExecutionReport {
            final_state: state,
            final_state_hash,
            outcomes,
            metrics,
        }
    }
}
