use cape::executor::static_scheduler::{build_static_batches, conflict};
use cape::model::Transaction;

#[test]
fn conflict_detects_write_write_and_read_write_overlap() {
    let first = Transaction::public_transfer(1, 0, 10, 20, 5, 1);
    let independent = Transaction::public_transfer(2, 1, 30, 40, 5, 1);
    let overlapping = Transaction::public_transfer(3, 2, 20, 50, 5, 1);

    assert!(!conflict(&first, &independent));
    assert!(conflict(&first, &overlapping));
}

#[test]
fn static_batches_keep_conflicting_transactions_apart() {
    let block = vec![
        Transaction::public_transfer(1, 0, 10, 20, 5, 1),
        Transaction::public_transfer(2, 1, 30, 40, 5, 1),
        Transaction::public_transfer(3, 2, 10, 50, 5, 1),
    ];

    let batches = build_static_batches(&block);

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].len(), 2);
    assert_eq!(batches[1].len(), 1);
}

#[test]
fn system_transactions_form_barrier_batches() {
    let block = vec![
        Transaction::public_transfer(1, 0, 10, 20, 5, 1),
        Transaction::system(2, 1, 10),
        Transaction::public_transfer(3, 2, 30, 40, 5, 1),
    ];

    let batches = build_static_batches(&block);

    assert_eq!(batches.len(), 3);
    assert_eq!(batches[1].len(), 1);
    assert!(batches[1][0].is_barrier());
}
