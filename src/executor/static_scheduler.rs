use crate::executor::{
    ExecutionConfig, ExecutionReport, Executor, count_outcomes, sequential_reference,
};
use crate::metrics::RunMetrics;
use crate::model::{StateKey, Transaction};
use crate::state::State;
use crate::tx::execute_transaction;
use rayon::prelude::*;
use std::collections::HashSet;
use std::time::Instant;

pub struct StaticSchedulerExecutor;

impl Executor for StaticSchedulerExecutor {
    fn name(&self) -> &'static str {
        "static"
    }

    fn execute(
        &self,
        initial_state: &State,
        block: &[Transaction],
        config: &ExecutionConfig,
    ) -> ExecutionReport {
        let started = Instant::now();
        let scheduling_started = Instant::now();
        let (batches, conflicts_detected) = build_static_batches_with_conflicts(block);
        let scheduling_ns = scheduling_started.elapsed().as_nanos();

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.threads.max(1))
            .build()
            .expect("rayon pool must build");

        let mut state = initial_state.clone();
        let mut outcomes = Vec::with_capacity(block.len());
        let mut execution_ns = 0;
        let mut commit_ns = 0;

        for batch in batches {
            let snapshot = state.clone();
            let mut executions = pool.install(|| {
                batch
                    .par_iter()
                    .map(|tx| execute_transaction(tx, &snapshot))
                    .collect::<Vec<_>>()
            });
            executions.sort_by_key(|execution| execution.outcome.index);

            for execution in executions {
                execution_ns += execution.execution_ns;
                let commit_started = Instant::now();
                state
                    .apply_diff(&execution.diff)
                    .expect("prototype diffs must be valid");
                commit_ns += commit_started.elapsed().as_nanos();
                outcomes.push(execution.outcome);
            }
        }

        outcomes.sort_by_key(|outcome| outcome.index);
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
            scheduling_ns,
            execution_ns,
            commit_ns,
            conflicts_detected,
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

pub fn build_static_batches(block: &[Transaction]) -> Vec<Vec<Transaction>> {
    build_static_batches_with_conflicts(block).0
}

fn build_static_batches_with_conflicts(block: &[Transaction]) -> (Vec<Vec<Transaction>>, u64) {
    let mut batches: Vec<Vec<Transaction>> = Vec::new();
    let mut current: Vec<Transaction> = Vec::new();
    let mut batch_reads: HashSet<StateKey> = HashSet::new();
    let mut batch_writes: HashSet<StateKey> = HashSet::new();
    let mut conflicts_detected = 0;

    for tx in block {
        if tx.is_barrier() {
            if !current.is_empty() {
                batches.push(current);
                current = Vec::new();
                batch_reads.clear();
                batch_writes.clear();
            }
            batches.push(vec![tx.clone()]);
            continue;
        }

        let tx_reads = tx.read_keys().into_iter().collect::<HashSet<_>>();
        let tx_writes = tx.write_keys().into_iter().collect::<HashSet<_>>();
        let batch_conflicts = count_intersections(&tx_writes, &batch_writes)
            + count_intersections(&tx_writes, &batch_reads)
            + count_intersections(&tx_reads, &batch_writes);
        if batch_conflicts > 0 {
            conflicts_detected += batch_conflicts;
            batches.push(current);
            current = vec![tx.clone()];
            batch_reads = tx_reads;
            batch_writes = tx_writes;
        } else {
            batch_reads.extend(tx_reads);
            batch_writes.extend(tx_writes);
            current.push(tx.clone());
        }
    }

    if !current.is_empty() {
        batches.push(current);
    }
    (batches, conflicts_detected)
}

pub fn conflict(left: &Transaction, right: &Transaction) -> bool {
    if left.is_barrier() || right.is_barrier() {
        return true;
    }
    let left_reads = left.read_keys().into_iter().collect::<HashSet<_>>();
    let left_writes = left.write_keys().into_iter().collect::<HashSet<_>>();
    let right_reads = right.read_keys().into_iter().collect::<HashSet<_>>();
    let right_writes = right.write_keys().into_iter().collect::<HashSet<_>>();

    intersects(&left_writes, &right_writes)
        || intersects(&left_writes, &right_reads)
        || intersects(&left_reads, &right_writes)
        || nullifier_overlap(left, right)
}

fn intersects(left: &HashSet<StateKey>, right: &HashSet<StateKey>) -> bool {
    left.iter().any(|key| right.contains(key))
}

fn count_intersections(left: &HashSet<StateKey>, right: &HashSet<StateKey>) -> u64 {
    left.iter().filter(|key| right.contains(*key)).count() as u64
}

fn nullifier_overlap(left: &Transaction, right: &Transaction) -> bool {
    left.consumed_nullifiers
        .iter()
        .any(|nullifier| right.consumed_nullifiers.contains(nullifier))
}
