use crate::*;

#[near_bindgen]
impl Contract {
    /// Returns semver of this contract.
    ///
    /// ```bash
    /// near view treasury.testnet version
    /// ```
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Returns all staking delegations
    ///
    /// ```bash
    /// near view treasury.testnet get_delegations '{"from_index": 0, "limit": 10}'
    /// ```
    pub fn get_delegations(
        &self,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<StakeDelegation> {
        let mut ret: Vec<StakeDelegation> = Vec::new();
        let mut start = 0;
        let mut end = 10;
        if let Some(from_index) = from_index {
            start = from_index.0;
        }
        if let Some(limit) = limit {
            end = u64::min(start + limit.0, self.stake_delegations.len());
        }

        // Return all data within range
        let keys = self.stake_delegations.keys_as_vector();
        for i in start..end {
            if let Some(pool_account_id) = keys.get(i) {
                if let Some(delegation) = self.stake_delegations.get(&pool_account_id) {
                    let mut delegation_info = StakeDelegation {
                        init_balance: delegation.init_balance,
                        balance: delegation.balance,
                        start_block: delegation.start_block,
                        withdraw_epoch: delegation.withdraw_epoch,
                        withdraw_balance: delegation.withdraw_balance,
                        withdraw_function: delegation.withdraw_function,
                        liquid_unstake_function: delegation.liquid_unstake_function,
                        yield_function: delegation.yield_function,
                    };

                    // Adjust pending info if exists
                    if let Some(pending_delegation) =
                        self.stake_pending_delegations.get(&pool_account_id)
                    {
                        delegation_info.withdraw_epoch = pending_delegation.withdraw_epoch;
                        delegation_info.withdraw_balance = pending_delegation.withdraw_balance;
                    }

                    ret.push(delegation_info);
                }
            }
        }
        ret
    }
}
