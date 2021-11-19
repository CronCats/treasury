use crate::*;

pub const GAS_STAKE_DEPOSIT_AND_STAKE: Gas = 10_000_000_000_000;
pub const GAS_STAKE_UNSTAKE: Gas = 10_000_000_000_000;
pub const GAS_STAKE_WITHDRAW_ALL: Gas = 10_000_000_000_000;
pub const GAS_STAKE_GET_STAKE_BALANCE: Gas = 10_000_000_000_000;
pub const GAS_STAKE_GET_STAKE_BALANCE_CALLBACK: Gas = 10_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolBalance {
    pub account_id: AccountId,
    pub unstaked_balance: U128,
    pub staked_balance: U128,
    pub can_withdraw: bool,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidUnstakeResult {
    pub near: U128String,
    pub fee: U128String,
    pub meta: U128String,
}

#[near_bindgen]
impl Contract {
    /// Manage which pools can get used
    ///
    /// ```bash
    /// near call treasury.testnet add_staking_pool '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn add_staking_pool(
        &mut self,
        pool_account_id: AccountId,
    ) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");
        let current_pool = self.stake_pools.get(&pool_account_id);

        // Insert ONLY if there isn't a record of this pool already
        // NOTE: Only managing the stake_pools, as stake_pending_pools is used for active balance movements
        assert!(current_pool.is_none(), "Stake pool exists already");
        self.stake_pools.insert(&pool_account_id, &0);
    }

    /// Remove a pool, if all balances have been withdrawn
    ///
    /// ```bash
    /// near call treasury.testnet remove_staking_pool '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn remove_staking_pool(
        &mut self,
        pool_account_id: AccountId,
    ) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");
        let current_pool = self.stake_pools.get(&pool_account_id);

        // Insert ONLY if there isn't a record of this pool already
        // NOTE: Only managing the stake_pools, as stake_pending_pools is used for active balance movements
        assert!(current_pool.is_some(), "Stake pool doesnt exist");
        assert_eq!(current_pool.unwrap(), 0, "Stake pool has a balance");
        self.stake_pools.remove(&pool_account_id);
    }

    /// Send NEAR to a staking pool and stake.
    /// 
    /// Logic:
    /// - if Attached deposit: Attached deposit will be used to stake
    /// - if Amount: Check if enough balance and then stake
    ///
    /// ```bash
    /// near call treasury.testnet deposit_and_stake '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet --amount 100000000000000000000000000
    /// ```
    ///
    /// OR
    ///
    /// ```bash
    /// near call treasury.testnet deposit_and_stake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    #[payable]
    pub fn deposit_and_stake(
        &mut self,
        pool_account_id: AccountId,
        amount: Option<Balance>,
    ) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");
        let mut stake_amount: Balance = 0;
        let pool_balance = self.stake_pools.get(&pool_account_id);
        assert!(pool_balance.is_some(), "Stake pool doesnt exist");

        if env::attached_deposit() > 0 {
            stake_amount = env::attached_deposit();
        } else {
            assert!(env::account_balance() > STAKE_BALANCE_MIN, "Account Balance Under Minimum Balance");
            if let Some(amount) = amount {
                stake_amount = amount;
            }
        }

        // Stop if somehow we made it this far and have nothing to stake... RUDE
        assert_ne!(stake_amount, 0, "Nothing to stake");

        // Update our local balance values, so we can keep track if we're using the pool (but not caring about interest earned)
        self.stake_pools.insert(&pool_account_id, &(pool_balance.unwrap() + stake_amount));

        // Lastly, make the cross-contract call to DO the staking :D
        let p = env::promise_create(
            pool_account_id,
            b"deposit_and_stake",
            json!({}).to_string().as_bytes(),
            stake_amount,
            GAS_STAKE_DEPOSIT_AND_STAKE
        );

        env::promise_return(p);
    }

    /// Unstake from a pool, works in metapool and traditional validator pools
    /// TODO: Consider amount optional, and setup a way to get current balance to signal unstake of ALL
    /// 
    /// ```bash
    /// near call treasury.testnet unstake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    pub fn unstake(
        &mut self,
        pool_account_id: AccountId,
        amount: Balance,
    ) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");
        let pool_balance = self.stake_pools.get(&pool_account_id);
        assert!(pool_balance.is_some(), "Stake pool doesnt exist");

        // Stop if somehow we made it this far and have nothing to stake... RUDE
        assert_ne!(amount, 0, "Nothing to stake");

        // Update our local balance values, so we know whats in process of long-form unstaking
        // TODO: Consider storing a timestamp here, so we know WHEN to revisit and withdraw
        self.stake_pending_pools.insert(&pool_account_id, &amount);

        // Lastly, make the cross-contract call to DO the unstaking :D
        let p = env::promise_create(
            pool_account_id,
            b"unstake",
            json!({
                "amount": amount,
            }).to_string().as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_UNSTAKE
        );

        env::promise_return(p);
    }

    /// Unstake from a pool, works in metapool and traditional validator pools
    /// 
    /// ```bash
    /// near call treasury.testnet withdraw_all '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn withdraw_all(
        &mut self,
        pool_account_id: AccountId,
    ) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");
        let pending_pool_balance = self.stake_pending_pools.get(&pool_account_id).expect("Withdraw pool doesnt exist");
        let pool_balance = self.stake_pending_pools.get(&pool_account_id).expect("Stake pool doesnt exist");

        // Clear the pending amount, update main pool amount
        self.stake_pending_pools.remove(&pool_account_id);
        self.stake_pools.insert(&pool_account_id, &(pool_balance.saturating_sub(pending_pool_balance)));

        // Lastly, make the cross-contract call to DO the withdraw :D
        let p = env::promise_create(
            pool_account_id,
            b"withdraw_all",
            json!({}).to_string().as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_WITHDRAW_ALL
        );

        env::promise_return(p);
    }

    /// Get the staked balance from a pool for THIS account
    /// 
    /// ```bash
    /// near call treasury.testnet get_staked_balance '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn get_staked_balance(
        &mut self,
        pool_account_id: AccountId,
    ) {
        assert!(self.stake_pools.get(&pool_account_id).is_some(), "Pool doesnt exist");

        // Lastly, make the cross-contract call to get the balance
        let p1 = env::promise_create(
            pool_account_id.clone(),
            b"get_account",
            json!({
                "account": env::current_account_id(),
            }).to_string().as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_GET_STAKE_BALANCE
        );

        let p2 = env::promise_then(
            p1,
            env::current_account_id(),
            b"callback_get_staked_balance",
            json!({
                "pool_account_id": pool_account_id,
            }).to_string().as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_GET_STAKE_BALANCE_CALLBACK
        );

        env::promise_return(p2);
    }

    /// CALLBACK for get_staked_balance
    #[private]
    pub fn callback_get_staked_balance(
        &mut self,
        pool_account_id: AccountId,
    ) -> (Balance, Balance, bool) {
        assert_eq!(env::promise_results_count(), 1, "Expected 1 promise result.");

        // Return balance or 0
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Successful(result) => {
                // Attempt to parse the returned balance amount
                let pool_balance: PoolBalance = serde_json::de::from_slice(&result)
                    .expect("Could not get balance from stake pool");

                // Update the balances of pool
                self.stake_pools.insert(&pool_account_id, &pool_balance.staked_balance.0);
                self.stake_pending_pools.insert(&pool_account_id, &pool_balance.unstaked_balance.0);
                (pool_balance.staked_balance.0, pool_balance.unstaked_balance.0, pool_balance.can_withdraw)
            }
            PromiseResult::Failed => {
                (0, 0, false)
            }
        }
    }

    // TODO: Harvest ("harvest_meta", 1 yocto)
    // TODO: Liquid unstake ("liquid_unstake", Figure out: {"st_near_to_burn": "NEAR","min_expected_near": "NEAR"} (NOTE: Used 2% fee. Might need to get the current fee?))
}