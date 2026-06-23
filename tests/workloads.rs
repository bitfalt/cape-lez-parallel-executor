use cape::executor::static_scheduler::conflict;
use cape::workload::hotspot::HotspotWorkload;
use cape::workload::no_conflict::NoConflictWorkload;
use cape::workload::private_state::PrivateStateWorkload;
use cape::workload::token_transfer::TokenTransferWorkload;
use cape::workload::{WorkloadConfig, WorkloadGenerator};
use std::collections::HashSet;

#[test]
fn same_seed_generates_same_token_transfer_block() {
    let config = WorkloadConfig {
        block_size: 64,
        accounts: 128,
        seed: 42,
        compute_cost: 1,
        skew: 1.1,
        duplicate_nullifier_rate: 0.0,
    };

    let first = TokenTransferWorkload.generate(&config);
    let second = TokenTransferWorkload.generate(&config);

    assert_eq!(first.block, second.block);
    assert_eq!(
        first.initial_state.state_hash(),
        second.initial_state.state_hash()
    );
}

#[test]
fn different_seed_generates_different_token_transfer_block() {
    let mut config = WorkloadConfig {
        block_size: 64,
        accounts: 128,
        seed: 42,
        compute_cost: 1,
        skew: 1.1,
        duplicate_nullifier_rate: 0.0,
    };
    let first = TokenTransferWorkload.generate(&config);
    config.seed = 43;
    let second = TokenTransferWorkload.generate(&config);

    assert_ne!(first.block, second.block);
}

#[test]
fn no_conflict_workload_has_pairwise_independent_transactions() {
    let scenario = NoConflictWorkload.generate(&WorkloadConfig {
        block_size: 32,
        accounts: 128,
        seed: 42,
        compute_cost: 1,
        skew: 1.1,
        duplicate_nullifier_rate: 0.0,
    });

    for i in 0..scenario.block.len() {
        for j in (i + 1)..scenario.block.len() {
            assert!(!conflict(&scenario.block[i], &scenario.block[j]));
        }
    }
}

#[test]
fn hotspot_workload_creates_shared_key_conflicts() {
    let scenario = HotspotWorkload.generate(&WorkloadConfig {
        block_size: 32,
        accounts: 128,
        seed: 42,
        compute_cost: 1,
        skew: 1.1,
        duplicate_nullifier_rate: 0.0,
    });

    let conflict_count = scenario
        .block
        .windows(2)
        .filter(|pair| conflict(&pair[0], &pair[1]))
        .count();

    assert!(conflict_count > 0);
}

#[test]
fn private_workload_can_generate_duplicate_nullifiers() {
    let scenario = PrivateStateWorkload.generate(&WorkloadConfig {
        block_size: 32,
        accounts: 128,
        seed: 42,
        compute_cost: 1,
        skew: 1.1,
        duplicate_nullifier_rate: 1.0,
    });

    let mut seen = HashSet::new();
    let duplicates = scenario
        .block
        .iter()
        .flat_map(|tx| tx.consumed_nullifiers.iter().copied())
        .filter(|nullifier| !seen.insert(*nullifier))
        .count();

    assert!(duplicates > 0);
}
