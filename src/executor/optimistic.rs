use crate::executor::{
    ExecutionConfig, ExecutionReport, Executor, count_outcomes, sequential_reference,
};
use crate::metrics::RunMetrics;
use crate::model::Transaction;
use crate::state::State;
use crate::tx::{execute_transaction, read_versions_still_valid};
use rayon::prelude::*;
use std::time::Instant;

pub struct OptimisticExecutor;

impl Executor for OptimisticExecutor {
    fn name(&self) -> &'static str {
        "optimistic"
    }

    fn execute(
        &self,
        initial_state: &State,
        block: &[Transaction],
        config: &ExecutionConfig,
    ) -> ExecutionReport {
        let started = Instant::now();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.threads.max(1))
            .build()
            .expect("rayon pool must build");

        let snapshot = initial_state.clone();
        let mut execution_ns = 0;
        let speculative = pool.install(|| {
            block
                .par_iter()
                .map(|tx| execute_transaction(tx, &snapshot))
                .collect::<Vec<_>>()
        });
        for execution in &speculative {
            execution_ns += execution.execution_ns;
        }

        let mut state = initial_state.clone();
        let mut outcomes = Vec::with_capacity(block.len());
        let mut validation_ns = 0;
        let mut commit_ns = 0;
        let mut reexecutions = 0;

        for (idx, speculative_execution) in speculative.iter().enumerate() {
            let validation_started = Instant::now();
            let valid = read_versions_still_valid(speculative_execution, &state);
            validation_ns += validation_started.elapsed().as_nanos();

            let execution = if valid {
                speculative_execution.clone()
            } else {
                reexecutions += 1;
                let fresh = execute_transaction(&block[idx], &state);
                execution_ns += fresh.execution_ns;
                fresh
            };

            let commit_started = Instant::now();
            state
                .apply_diff(&execution.diff)
                .expect("prototype diffs must be valid");
            commit_ns += commit_started.elapsed().as_nanos();
            outcomes.push(execution.outcome);
        }

        let final_state_hash = state.state_hash();
        let elapsed_ns = started.elapsed().as_nanos();
        let (seq_hash, seq_outcomes) = sequential_reference(initial_state, block);
        let correct = final_state_hash == seq_hash && outcomes == seq_outcomes;
        let (accepted_txs, rejected_txs) = count_outcomes(&outcomes);
        let mut metrics = RunMetrics {
            executor: self.name().to_string(),
            block_size: block.len(),
            threads: config.threads,
            elapsed_ns,
            execution_ns,
            validation_ns,
            commit_ns,
            reexecutions,
            accepted_txs,
            rejected_txs,
            final_state_hash: final_state_hash.clone(),
            correct_vs_sequential: Some(correct),
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
