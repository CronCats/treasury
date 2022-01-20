use near_sdk::{env, PromiseResult};
use crate::*;

pub const EPOCH_LENGTH: u64 = 43_200;

pub fn assert_owner(owner_id: &AccountId) {
    assert_eq!(owner_id, &env::predecessor_account_id(), "Must be owner");
}

pub fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}

pub fn get_epoch_withdrawal_time(epochs: Option<u64>) -> u64 {
    let epoch_multiple: u64 = epochs.unwrap_or(6); // default 72 hours
    epoch_multiple.saturating_mul(EPOCH_LENGTH)
}
