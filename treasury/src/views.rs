use std::str::FromStr;

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

    /// Returns owner config of this contract.
    ///
    /// ```bash
    /// near view treasury.testnet get_config
    /// ```
    pub fn get_config(&self) -> (
        bool,   // paused
        String, // owner
        String, // croncat
        StakeThreshold, // stake config
    ) {
        (
            self.paused,
            self.owner_id.to_string(),
            self.croncat_id.clone().unwrap_or(AccountId::from_str("no_croncat_account").unwrap()).to_string(),
            self.stake_threshold.clone(),
        )
    }

    /// Returns helpful info & stats.
    ///
    /// ```bash
    /// near view treasury.testnet get_info
    /// ```
    pub fn get_info(&self) -> (
        u64,
        u64,
        u64,
        u64,
        u64,
        u64,
        u64,
    ) {
        (
            self.approved_action_types.len(),
            self.cadence_actions.keys_as_vector().len(),
            self.timeout_actions.len(),
            self.ft_balances.keys_as_vector().len(),
            self.nft_holdings.keys_as_vector().len(),
            self.stake_delegations.len(),
            self.stake_pending_delegations.len(),
        )
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
    ) -> Vec<StakeDelegationHumanFriendly> {
        let mut ret: Vec<StakeDelegationHumanFriendly> = Vec::new();
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
                    let mut delegation_info = StakeDelegationHumanFriendly {
                        pool_account_id: pool_account_id.clone(),
                        init_balance: U128::from(delegation.init_balance),
                        balance: U128::from(delegation.balance),
                        start_block: delegation.start_block,
                        withdraw_epoch: delegation.withdraw_epoch,
                        withdraw_balance: Some(U128::from(delegation.withdraw_balance.unwrap_or(0))),
                        withdraw_function: delegation.withdraw_function,
                        liquid_unstake_function: delegation.liquid_unstake_function,
                        yield_function: delegation.yield_function,
                    };

                    // Adjust pending info if exists
                    if let Some(pending_delegation) =
                        self.stake_pending_delegations.get(&pool_account_id)
                    {
                        delegation_info.withdraw_epoch = pending_delegation.withdraw_epoch;
                        delegation_info.withdraw_balance = Some(U128::from(pending_delegation.withdraw_balance.unwrap_or(0)));
                    }

                    ret.push(delegation_info);
                }
            }
        }
        ret
    }
}
