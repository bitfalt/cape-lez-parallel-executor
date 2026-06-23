use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type AccountId = u64;
pub type ProgramId = u64;
pub type TxId = u64;
pub type Nullifier = u64;
pub type Commitment = u64;
pub type Amount = i64;
pub type Version = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StateKey {
    AccountBalance(AccountId),
    AccountNonce(AccountId),
    Nullifier(Nullifier),
    CommitmentSlot(u64),
    CommitmentRoot,
    ProgramCode(ProgramId),
    ProgramState { program: ProgramId, key: u64 },
    SystemClock,
    GlobalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub balance: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramState {
    pub storage: HashMap<u64, i64>,
    pub deployed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TxKind {
    PublicTransfer,
    PrivateTransfer,
    ProgramCall,
    ProgramDeployment,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub id: TxId,
    pub index: usize,
    pub kind: TxKind,
    pub signer: AccountId,
    pub receiver: Option<AccountId>,
    pub declared_reads: Vec<StateKey>,
    pub declared_writes: Vec<StateKey>,
    pub consumed_nullifiers: Vec<Nullifier>,
    pub produced_commitments: Vec<Commitment>,
    pub program_id: Option<ProgramId>,
    pub program_key: Option<u64>,
    pub amount: Amount,
    pub compute_cost: u64,
}

impl Transaction {
    pub fn public_transfer(
        id: TxId,
        index: usize,
        signer: AccountId,
        receiver: AccountId,
        amount: Amount,
        compute_cost: u64,
    ) -> Self {
        Self {
            id,
            index,
            kind: TxKind::PublicTransfer,
            signer,
            receiver: Some(receiver),
            declared_reads: vec![
                StateKey::AccountBalance(signer),
                StateKey::AccountBalance(receiver),
                StateKey::AccountNonce(signer),
            ],
            declared_writes: vec![
                StateKey::AccountBalance(signer),
                StateKey::AccountBalance(receiver),
                StateKey::AccountNonce(signer),
            ],
            consumed_nullifiers: vec![],
            produced_commitments: vec![],
            program_id: None,
            program_key: None,
            amount,
            compute_cost,
        }
    }

    pub fn private_transfer(
        id: TxId,
        index: usize,
        signer: AccountId,
        nullifier: Nullifier,
        commitment: Commitment,
        compute_cost: u64,
    ) -> Self {
        Self {
            id,
            index,
            kind: TxKind::PrivateTransfer,
            signer,
            receiver: None,
            declared_reads: vec![
                StateKey::Nullifier(nullifier),
                StateKey::CommitmentRoot,
                StateKey::AccountNonce(signer),
            ],
            declared_writes: vec![
                StateKey::Nullifier(nullifier),
                StateKey::CommitmentRoot,
                StateKey::CommitmentSlot(index as u64),
                StateKey::AccountNonce(signer),
            ],
            consumed_nullifiers: vec![nullifier],
            produced_commitments: vec![commitment],
            program_id: None,
            program_key: None,
            amount: 0,
            compute_cost,
        }
    }

    pub fn program_call(
        id: TxId,
        index: usize,
        signer: AccountId,
        program_id: ProgramId,
        key: u64,
        delta: Amount,
        compute_cost: u64,
    ) -> Self {
        Self {
            id,
            index,
            kind: TxKind::ProgramCall,
            signer,
            receiver: None,
            declared_reads: vec![
                StateKey::ProgramCode(program_id),
                StateKey::ProgramState {
                    program: program_id,
                    key,
                },
                StateKey::AccountNonce(signer),
            ],
            declared_writes: vec![
                StateKey::ProgramState {
                    program: program_id,
                    key,
                },
                StateKey::AccountNonce(signer),
            ],
            consumed_nullifiers: vec![],
            produced_commitments: vec![],
            program_id: Some(program_id),
            program_key: Some(key),
            amount: delta,
            compute_cost,
        }
    }

    pub fn program_deployment(
        id: TxId,
        index: usize,
        signer: AccountId,
        program_id: ProgramId,
        compute_cost: u64,
    ) -> Self {
        Self {
            id,
            index,
            kind: TxKind::ProgramDeployment,
            signer,
            receiver: None,
            declared_reads: vec![StateKey::ProgramCode(program_id), StateKey::GlobalConfig],
            declared_writes: vec![StateKey::ProgramCode(program_id), StateKey::GlobalConfig],
            consumed_nullifiers: vec![],
            produced_commitments: vec![],
            program_id: Some(program_id),
            program_key: None,
            amount: 0,
            compute_cost,
        }
    }

    pub fn system(id: TxId, index: usize, compute_cost: u64) -> Self {
        Self {
            id,
            index,
            kind: TxKind::System,
            signer: 0,
            receiver: None,
            declared_reads: vec![StateKey::SystemClock, StateKey::GlobalConfig],
            declared_writes: vec![StateKey::SystemClock, StateKey::GlobalConfig],
            consumed_nullifiers: vec![],
            produced_commitments: vec![],
            program_id: None,
            program_key: None,
            amount: 0,
            compute_cost,
        }
    }

    pub fn read_keys(&self) -> Vec<StateKey> {
        self.declared_reads.clone()
    }

    pub fn write_keys(&self) -> Vec<StateKey> {
        self.declared_writes.clone()
    }

    pub fn is_barrier(&self) -> bool {
        matches!(self.kind, TxKind::ProgramDeployment | TxKind::System)
            || self
                .declared_writes
                .iter()
                .any(|key| matches!(key, StateKey::SystemClock | StateKey::GlobalConfig))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    Accepted,
    Rejected(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutcome {
    pub tx_id: TxId,
    pub index: usize,
    pub status: TxStatus,
}

impl TxOutcome {
    pub fn accepted(tx: &Transaction) -> Self {
        Self {
            tx_id: tx.id,
            index: tx.index,
            status: TxStatus::Accepted,
        }
    }

    pub fn rejected(tx: &Transaction, reason: impl Into<String>) -> Self {
        Self {
            tx_id: tx.id,
            index: tx.index,
            status: TxStatus::Rejected(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateDiff {
    pub balance_delta: HashMap<AccountId, Amount>,
    pub nonce_delta: HashMap<AccountId, u64>,
    pub nullifiers_to_add: Vec<Nullifier>,
    pub commitments_to_append: Vec<Commitment>,
    pub program_writes: Vec<(ProgramId, u64, i64)>,
    pub programs_to_deploy: Vec<ProgramId>,
    pub system_clock_delta: u64,
    pub written_keys: Vec<StateKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxExecution {
    pub outcome: TxOutcome,
    pub diff: StateDiff,
    pub read_versions: HashMap<StateKey, Version>,
    pub read_keys: Vec<StateKey>,
    pub write_keys: Vec<StateKey>,
    pub execution_ns: u128,
}
