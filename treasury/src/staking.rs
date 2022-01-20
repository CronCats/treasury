use crate::*;

use near_sdk::BlockHeight;
use utils::{assert_owner};

/// Amount of blocks needed before withdraw is available
pub const GAS_STAKE_DEPOSIT_AND_STAKE: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_UNSTAKE: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_WITHDRAW_ALL: Gas = Gas(40_000_000_000_000);
pub const GAS_STAKE_GET_STAKE_BALANCE: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_GET_STAKE_BALANCE_CALLBACK: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_VIEW: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_CALLBACK: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_POOL_CALL: Gas = Gas(10_000_000_000_000);
pub const GAS_YIELD_HARVEST: Gas = Gas(10_000_000_000_000);
pub const GAS_CRONCAT_CREATE_TASK: Gas = Gas(30_000_000_000_000);

/// Stake Buckets keep track of staked amounts per-pool
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeDelegation {
    /// The starting balance, to compute future gains
    pub init_balance: Balance,
    /// non-realtime balance, updated by CALLs, useful in some scenarios
    pub balance: Balance,
    /// To keep track of how long at stake
    pub start_block: BlockHeight,
    /// To keep track of when withdraw is available for stake
    pub withdraw_ts: Option<u64>,
    /// For computing how much is available for withdraw upon ready
    pub withdraw_balance: Option<Balance>,
    /// Some providers have diff implementations
    pub withdraw_function: String,
    /// For enabling short term staking
    pub liquid_unstake_function: Option<String>,
    /// For enabling yield from harvesting solutions (EX: Metapool $META)
    pub yield_function: Option<String>,
}

/// Stake Buckets keep track of staked amounts per-pool
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeThreshold {
    pub liquid: u64,
    pub staked: u64,
    pub deviation: u64,
    pub extreme_deviation: u64,
    pub eval_period: u128,    // Decide on time delay, in seconds
    pub eval_cadence: String, // OR cron cadence
}

// TODO:
// 1. check threshold
// 2. trigger check threshold upon balance change
// 3. stake more: 1 or many?
// 4. unstake: 1 or many?
// 5. withdraw: Schedule then execute, immediate
// - Change some fns to be allowed to be called by approved accounts (owners or croncat)
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
        // If a pool has liquid staking abilities, you can provide here. Examples: MetaPool: liquid_unstake
        liquid_unstake_function: Option<String>,
        // If a pool has harvesting abilities, you can provide here. Examples: MetaPool: harvest_meta, CheddarFarm: withdraw_crop
        yield_function: Option<String>,
        // IF the withdraw function is different than standard
        withdraw_function: Option<String>,
    ) {
        assert_owner(&self.owner_id);
        let current_pool = self.stake_delegations.get(&pool_account_id);

        // Insert ONLY if there isn't a record of this pool already
        // NOTE: Only managing the stake_delegations, as stake_pending_delegations is used for active balance movements
        assert!(current_pool.is_none(), "Stake pool exists already");
        self.stake_delegations.insert(&pool_account_id, &StakeDelegation {
            init_balance: 0,
            balance: 0,
            start_block: 0, // 0 indicates that the staking has not started yet
            withdraw_ts: None,
            withdraw_balance: None,
            withdraw_function: withdraw_function.unwrap_or("withdraw_all".to_string()),
            liquid_unstake_function,
            yield_function,
        });
    }

    /// Remove a pool, if all balances have been withdrawn
    ///
    /// ```bash
    /// near call treasury.testnet remove_staking_pool '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn remove_staking_pool(&mut self, pool_account_id: AccountId) {
        assert_owner(&self.owner_id);
        let current_pool = self.stake_delegations.get(&pool_account_id);

        // Insert ONLY if there isn't a record of this pool already
        // NOTE: Only managing the stake_delegations, as stake_pending_delegations is used for active balance movements
        assert!(current_pool.is_some(), "Stake pool doesnt exist");
        assert_eq!(current_pool.unwrap().balance, 0, "Stake pool has a balance");
        self.stake_delegations.remove(&pool_account_id);
    }

    /// Check staking threshold for eval
    /// Logic:
    /// - Checks if current balance is above defined threshold (Example if account total balance is 100 near, liquid 40 with a setting of staking 80%, go ahead and stake)
    /// - If threshold is out of proportion, trigger one of the following:
    ///     - Stake: If above threshold
    ///     - UnStake: If below threshold
    ///     - Liquid UnStake: If below extreme threshold
    /// 
    /// ```bash
    /// near call treasury.testnet auto_stake --accountId manager_v1.croncat.testnet
    /// ```
    pub fn auto_stake(&mut self) {
        // TODO: Adjust as needed:
        // stake_threshold_percentage: 3000,              // 30%
        // stake_eval_period: 86400,                      // Daily eval delay, in seconds
        // stake_eval_cadence: "0 0 * * * *".to_string(), // Every hour cadence
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
    pub fn deposit_and_stake(&mut self, pool_account_id: AccountId, amount: Option<Balance>) {
        assert_owner(&self.owner_id);
        let mut stake_amount: Balance = 0;
        let pool_delegation = self.stake_delegations.get(&pool_account_id);
        assert!(pool_delegation.is_some(), "Stake delegation doesnt exist");

        if env::attached_deposit() > 0 {
            stake_amount = env::attached_deposit();
        } else {
            assert!(
                u128::from(env::account_balance()).saturating_sub(amount.unwrap_or(0)) > MIN_BALANCE_FOR_STORAGE,
                "Account Balance Under Minimum Balance"
            );
            if let Some(amount) = amount {
                stake_amount = amount;
            }
        }

        // Stop if somehow we made it this far and have nothing to stake... RUDE
        assert_ne!(stake_amount, 0, "Nothing to stake");

        let delegation = pool_delegation.unwrap();
        let updated_delegation = StakeDelegation {
            init_balance: stake_amount,
            balance: stake_amount,
            start_block: env::block_height(),
            withdraw_ts: None,
            withdraw_balance: None,
            withdraw_function: delegation.withdraw_function,
            liquid_unstake_function: delegation.liquid_unstake_function,
            yield_function: delegation.yield_function,
        };

        // Update our local balance values, so we can keep track if we're using the pool (but not caring about interest earned)
        self.stake_delegations
            .insert(&pool_account_id, &updated_delegation);

        // Lastly, make the cross-contract call to DO the staking :D
        let p = env::promise_create(
            pool_account_id,
            "deposit_and_stake",
            json!({}).to_string().as_bytes(),
            stake_amount,
            GAS_STAKE_DEPOSIT_AND_STAKE,
        );

        env::promise_return(p);
    }

    /// Get the staked balance from a pool for THIS account
    /// NOTE: This is a CALL because it updates internal balances
    ///
    /// ```bash
    /// near call treasury.testnet get_staked_balance '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    pub fn get_staked_balance(&mut self, pool_account_id: AccountId) {
        assert!(
            self.stake_delegations.get(&pool_account_id).is_some(),
            "Delegation doesnt exist"
        );

        // make the cross-contract call to get the balance
        let p1 = env::promise_create(
            pool_account_id.clone(),
            "get_account",
            json!({
                "account_id": env::current_account_id(),
            })
            .to_string()
            .as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_GET_STAKE_BALANCE,
        );

        let p2 = env::promise_then(
            p1,
            env::current_account_id(),
            "callback_get_staked_balance",
            json!({
                "pool_account_id": pool_account_id,
            })
            .to_string()
            .as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_GET_STAKE_BALANCE_CALLBACK,
        );

        env::promise_return(p2);
    }

    /// CALLBACK for get_staked_balance
    /// Returns:
    /// (
    /// staked balance,
    /// unstaking balance,
    /// can withdraw,
    /// )
    #[private]
    pub fn callback_get_staked_balance(
        &mut self,
        pool_account_id: AccountId,
    ) -> (Balance, Balance, bool) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Expected 1 promise result."
        );

        // Return balance or 0
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Successful(result) => {
                // Attempt to parse the returned balance amount
                let pool_balance: external::PoolBalance = serde_json::de::from_slice(&result)
                    .expect("Could not get balance from stake delegation");
                let mut delegation = self.stake_delegations.get(&pool_account_id).expect("No delegation found");

                // Update internal balances
                delegation.balance = pool_balance.staked_balance.0;
                delegation.withdraw_balance = Some(pool_balance.unstaked_balance.0);

                // If its known, immediately make withdraw available, otherwise compute when withdraw is available
                if pool_balance.can_withdraw {
                    let unstake_duration: u64 = utils::get_epoch_withdrawal_time(None);
                    delegation.withdraw_ts = Some(env::block_timestamp().saturating_sub(unstake_duration * 1_000_000));
                }

                // Update the balances of pool
                self.stake_delegations
                    .insert(&pool_account_id, &delegation);
                self.stake_pending_delegations
                    .insert(&pool_account_id, &delegation);
                (
                    pool_balance.staked_balance.0,
                    pool_balance.unstaked_balance.0,
                    pool_balance.can_withdraw,
                )
            }
            PromiseResult::Failed => (0, 0, false),
        }
    }

    /// Unstake from a pool, works in metapool and traditional validator pools
    /// NOTE: Unstaking here will schedule the automatic withdrawal in future epoch
    ///
    /// ```bash
    /// near call treasury.testnet unstake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
    /// 
    /// OR, to unstake ALL:
    /// 
    /// near call treasury.testnet unstake '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn unstake(&mut self, pool_account_id: AccountId, amount: Option<Balance>) {
        assert_owner(&self.owner_id);
        let pool_delegation = self.stake_delegations.get(&pool_account_id);
        assert!(pool_delegation.is_some(), "Stake delegation doesnt exist");
        let mut unstake_function = "unstake_all";

        // Stop if somehow we made it this far and have nothing to unstake... RUDE
        if amount.is_some() {
            assert_ne!(amount.unwrap(), 0, "Nothing to unstake");
            unstake_function = "unstake";
        }

        // Update our local balance values, so we know whats in process of long-form unstaking
        let mut delegation = pool_delegation.unwrap();
        let withdraw_balance = amount.unwrap_or(0);
        delegation.withdraw_ts = Some(env::block_timestamp());
        delegation.withdraw_balance = Some(withdraw_balance);
        self.stake_pending_delegations.insert(&pool_account_id, &delegation);

        // Lastly, make the cross-contract call to DO the unstaking :D
        let p = env::promise_create(
            pool_account_id,
            &unstake_function,
            json!({
                "amount": amount,
            })
            .to_string()
            .as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_UNSTAKE,
        );

        // Add withdraw scheduler, if croncat is configured
        if self.croncat_id.is_some() {
            external::croncat::create_task(
                env::current_account_id().to_string(),
                "withdraw_all".to_string(),
                // TODO: what cadence is needed here? (this sets it to every sunday at minute 0), ideally can set a block height start
                "0 0 * * 0 *".to_string(),
                Some(false),
                Some(U128::from(NO_DEPOSIT)),
                Some(GAS_STAKE_WITHDRAW_ALL + GAS_CRONCAT_CREATE_TASK), // 70 Tgas
                None,
                self.croncat_id.clone().unwrap(),
                env::attached_deposit(),
                GAS_CRONCAT_CREATE_TASK,
            );
        }

        env::promise_return(p);
    }

    /// Withdraw unstaked balance from a pool, works in metapool and traditional validator pools
    ///
    /// ```bash
    /// near call treasury.testnet withdraw_all '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn withdraw_all(&mut self, pool_account_id: AccountId) {
        assert_owner(&self.owner_id);
        let pending_pool_delegation = self
            .stake_pending_delegations
            .get(&pool_account_id)
            .expect("Withdraw delegation doesnt exist");
        let mut pool_delegation = self
            .stake_delegations
            .get(&pool_account_id)
            .expect("Stake delegation doesnt exist");

        // Clear the pending amount, update main pool amount
        // NOTE: would be great to do this on a callback, but seems withdraw functions dont provide how much was withdrawn on response
        // NOTE: Could try getting current account balance and balance after callback, however it doesnt work if this is used for FT staking
        pool_delegation.balance = pool_delegation.balance.saturating_sub(pending_pool_delegation.withdraw_balance.unwrap_or(0));
        self.stake_pending_delegations.remove(&pool_account_id);
        self.stake_delegations.insert(&pool_account_id, &pool_delegation);

        // Lastly, make the cross-contract call to DO the withdraw :D
        let p = env::promise_create(
            pool_account_id,
            &pool_delegation.withdraw_function,
            json!({}).to_string().as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_WITHDRAW_ALL,
        );

        env::promise_return(p);
    }

    /// Unstake any liquid staked near tokens for NEAR. Useful for situations that require immediate access to NEAR.
    ///
    /// ```bash
    /// near call treasury.testnet liquid_unstake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    pub fn liquid_unstake(&mut self, pool_account_id: AccountId, amount: Option<Balance>) {
        assert_owner(&self.owner_id);
        let delegated_stake = self.stake_delegations.get(&pool_account_id);
        assert!(delegated_stake.is_some(), "Delegation doesnt exist");
        let delegation = delegated_stake.unwrap();
        assert!(
            delegation.liquid_unstake_function.is_some(),
            "Liquid unstake unsupported for this pool"
        );

        // First check if there are any staked balances
        let p1 = env::promise_create(
            pool_account_id.clone(),
            "get_account_info",
            json!({
                "account_id": env::current_account_id(),
            })
            .to_string()
            .as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_LIQUID_UNSTAKE_VIEW,
        );

        let p2 = env::promise_then(
            p1,
            env::current_account_id(),
            "callback_liquid_unstake",
            json!({
                "pool_account_id": pool_account_id,
                "amount": amount,
            })
            .to_string()
            .as_bytes(),
            NO_DEPOSIT,
            GAS_STAKE_LIQUID_UNSTAKE_CALLBACK,
        );

        env::promise_return(p2);
    }

    /// CALLBACK for get_staked_balance
    #[private]
    pub fn callback_liquid_unstake(&mut self, pool_account_id: AccountId, amount: Option<Balance>) {
        utils::is_promise_success();

        // Return balance or 0
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Successful(result) => {
                // Attempt to parse the returned account balances
                let pool_balance: external::MetaPoolBalance = serde_json::de::from_slice(&result)
                    .expect("Could not get balance from stake pool");

                // Double check values before going forward
                assert!(pool_balance.st_near.0 > 0, "No st_near balance");
                assert!(
                    pool_balance.valued_st_near.0 > 0,
                    "No valued_st_near balance"
                );
                let mut st_near_to_burn = pool_balance.st_near;
                let mut min_expected_near = pool_balance.valued_st_near;

                // If no amount specified, simply unstake all
                if amount.is_some() {
                    // Get st_near / near price, and compute st_near amount
                    // TODO: Check this division isnt naive
                    let st_near_price = pool_balance
                        .st_near
                        .0
                        .div_euclid(pool_balance.valued_st_near.0);
                    st_near_to_burn = U128::from(amount.unwrap().div_euclid(st_near_price));
                    min_expected_near = U128::from(amount.unwrap());
                }

                // We have some balances, attempt to unstake
                // TODO: No fee was calculated, does that cause issues on min_expected_near?
                let delegation = self.stake_delegations.get(&pool_account_id).expect("Delegation doesnt exist");
                let p1 = env::promise_create(
                    pool_account_id.clone(),
                    &delegation.liquid_unstake_function.unwrap(),
                    json!({
                        "st_near_to_burn": st_near_to_burn,
                        "min_expected_near": min_expected_near,
                    })
                    .to_string()
                    .as_bytes(),
                    ONE_YOCTO,
                    GAS_STAKE_LIQUID_UNSTAKE_POOL_CALL,
                );

                env::promise_return(p1);
            }
            PromiseResult::Failed => {
                // Fail me not, please
            }
        }
    }

    /// Execute a yield harvest for staking pools that support it.
    ///
    /// ```bash
    /// near call treasury.testnet yield_harvest '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn yield_harvest(&mut self, pool_account_id: AccountId) {
        assert_owner(&self.owner_id);
        let delegated_stake = self.stake_delegations.get(&pool_account_id);
        assert!(delegated_stake.is_some(), "Delegation doesnt exist");
        let delegation = delegated_stake.unwrap();
        assert!(
            delegation.yield_function.is_some(),
            "Yield unsupported for this pool"
        );

        // Make a yield harvest call, including yocto since most include FT that needs txns with priveledges
        let p = env::promise_create(
            pool_account_id,
            &delegation.yield_function.unwrap(),
            json!({}).to_string().as_bytes(),
            ONE_YOCTO,
            GAS_YIELD_HARVEST,
        );

        env::promise_return(p);
    }
}
