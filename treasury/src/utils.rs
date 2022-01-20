use crate::*;
use near_sdk::{env, PromiseResult};
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

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

pub fn calc_percent(numerator: u64, denominator: u64, value: u128) -> u128 {
    (U256::from(numerator) * U256::from(value) / U256::from(denominator)).as_u128()
}
