use cape::executor::{
    ExecutionConfig, Executor, OptimisticExecutor, SequentialExecutor, StaticSchedulerExecutor,
};
use cape::model::{Transaction, TxStatus};
use cape::state::State;
use cape::tx::execute_transaction;
use cape::workload::token_transfer::TokenTransferWorkload;
use cape::workload::{WorkloadConfig, WorkloadGenerator};

#[test]
fn state_hash_is_independent_of_insertion_order() {
    let mut left = State::default();
    left.set_account_balance(1, 100);
    left.set_account_balance(2, 250);
    left.set_nonce(1, 3);

    let mut right = State::default();
    right.set_nonce(1, 3);
    right.set_account_balance(2, 250);
    right.set_account_balance(1, 100);

    assert_eq!(left.state_hash(), right.state_hash());
}

#[test]
fn public_transfer_updates_balances_nonce_and_versions() {
    let mut state = State::default();
    state.set_account_balance(1, 100);
    state.set_account_balance(2, 0);

    let tx = Transaction::public_transfer(7, 0, 1, 2, 40, 1);
    let execution = execute_transaction(&tx, &state);

    assert_eq!(execution.outcome.status, TxStatus::Accepted);
    state.apply_diff(&execution.diff).unwrap();

    assert_eq!(state.balance_of(1), 60);
    assert_eq!(state.balance_of(2), 40);
    assert_eq!(state.nonce_of(1), 1);
    assert!(state.version_of(&tx.write_keys()[0]) > 0);
}

#[test]
fn static_and_optimistic_executors_match_sequential_on_token_transfers() {
    let scenario = TokenTransferWorkload.generate(&WorkloadConfig {
        block_size: 128,
        accounts: 64,
        seed: 42,
        compute_cost: 25,
        skew: 1.1,
        duplicate_nullifier_rate: 0.0,
    });
    let config = ExecutionConfig {
        threads: 4,
        fallback_reexecution_rate: 0.75,
    };

    let seq = SequentialExecutor.execute(&scenario.initial_state, &scenario.block, &config);
    let sta = StaticSchedulerExecutor.execute(&scenario.initial_state, &scenario.block, &config);
    let opt = OptimisticExecutor.execute(&scenario.initial_state, &scenario.block, &config);

    assert_eq!(sta.final_state_hash, seq.final_state_hash);
    assert_eq!(opt.final_state_hash, seq.final_state_hash);
    assert_eq!(sta.outcomes, seq.outcomes);
    assert_eq!(opt.outcomes, seq.outcomes);
    assert!(sta.metrics.correct_vs_sequential.unwrap());
    assert!(opt.metrics.correct_vs_sequential.unwrap());
}
