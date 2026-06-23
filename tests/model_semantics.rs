use cape::model::{Transaction, TxStatus};
use cape::state::State;
use cape::tx::execute_transaction;

#[test]
fn private_transfer_rejects_reused_nullifier() {
    let mut state = State::default();
    state.set_account_balance(1, 100);

    let first = Transaction::private_transfer(1, 0, 1, 999, 5000, 1);
    let first_execution = execute_transaction(&first, &state);
    assert_eq!(first_execution.outcome.status, TxStatus::Accepted);
    state.apply_diff(&first_execution.diff).unwrap();

    let second = Transaction::private_transfer(2, 1, 1, 999, 5001, 1);
    let second_execution = execute_transaction(&second, &state);
    assert!(matches!(
        second_execution.outcome.status,
        TxStatus::Rejected(reason) if reason.contains("already used")
    ));
}

#[test]
fn program_deployment_rejects_duplicate_program() {
    let mut state = State::default();

    let deploy = Transaction::program_deployment(1, 0, 1, 77, 1);
    let first_execution = execute_transaction(&deploy, &state);
    assert_eq!(first_execution.outcome.status, TxStatus::Accepted);
    state.apply_diff(&first_execution.diff).unwrap();

    let duplicate = Transaction::program_deployment(2, 1, 1, 77, 1);
    let duplicate_execution = execute_transaction(&duplicate, &state);
    assert!(matches!(
        duplicate_execution.outcome.status,
        TxStatus::Rejected(reason) if reason.contains("already deployed")
    ));
}

#[test]
fn program_call_rejects_missing_program_and_accepts_existing_program() {
    let mut state = State::default();

    let missing = Transaction::program_call(1, 0, 1, 99, 7, 3, 1);
    let missing_execution = execute_transaction(&missing, &state);
    assert!(matches!(
        missing_execution.outcome.status,
        TxStatus::Rejected(reason) if reason.contains("not deployed")
    ));

    let deploy = Transaction::program_deployment(2, 1, 1, 99, 1);
    let deploy_execution = execute_transaction(&deploy, &state);
    state.apply_diff(&deploy_execution.diff).unwrap();

    let call = Transaction::program_call(3, 2, 1, 99, 7, 3, 1);
    let call_execution = execute_transaction(&call, &state);
    assert_eq!(call_execution.outcome.status, TxStatus::Accepted);
    state.apply_diff(&call_execution.diff).unwrap();

    let stored = state
        .programs
        .get(&99)
        .and_then(|program| program.storage.get(&7))
        .copied();
    assert_eq!(stored, Some(3));
}
