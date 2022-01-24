use crate::*;

pub fn calc_percent(numerator: u64, denominator: u64, value: u128) -> u128 {
    (U256::from(numerator) * U256::from(value) / U256::from(denominator)).as_u128()
}
