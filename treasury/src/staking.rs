use crate::*;

use near_sdk::BlockHeight;

/// Amount of blocks needed before withdraw is available
pub const GAS_STAKE_DEPOSIT_AND_STAKE: Gas = Gas(70_000_000_000_000);
pub const GAS_STAKE_UNSTAKE: Gas = Gas(40_000_000_000_000);
pub const GAS_STAKE_WITHDRAW_ALL: Gas = Gas(40_000_000_000_000);
pub const GAS_STAKE_GET_STAKE_BALANCE: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_GET_STAKE_BALANCE_CALLBACK: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_VIEW: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_FEE_VIEW: Gas = Gas(10_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_CALLBACK: Gas = Gas(80_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_CALLBACK_FINAL: Gas = Gas(40_000_000_000_000);
pub const GAS_STAKE_LIQUID_UNSTAKE_POOL_CALL: Gas = Gas(30_000_000_000_000);
pub const GAS_YIELD_HARVEST: Gas = Gas(120_000_000_000_000);
pub const GAS_CRONCAT_CREATE_TASK: Gas = Gas(30_000_000_000_000);

pub const CRONCAT_CREATE_TASK_FEE: Balance = 17500000000000000000000;

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
    /// To keep track of the epoch when withdraw is available from unstake
    pub withdraw_epoch: Option<u64>,
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
pub struct StakeDelegationHumanFriendly {
    pub pool_account_id: AccountId,
    pub init_balance: U128,
    pub balance: U128,
    pub start_block: BlockHeight,
    pub withdraw_epoch: Option<u64>,
    pub withdraw_balance: Option<U128>,
    pub withdraw_function: String,
    pub liquid_unstake_function: Option<String>,
    pub yield_function: Option<String>,
}

/// Stake Buckets keep track of staked amounts per-pool
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeThreshold {
    // TODO: Bool for turning on/off auto-staking
    pub denominator: u64,       // default 100
    pub liquid: u64,            // default 30%
    pub staked: u64,            // default 70%
    pub deviation: u64,         // default 5%
    pub extreme_deviation: u64, // default 15%
    pub eval_period: u128,      // Decide on time delay, in seconds
    pub eval_cadence: String,   // OR cron cadence
}

impl Default for StakeThreshold {
    fn default() -> Self {
        StakeThreshold {
            denominator: 100,
            liquid: 30,                              // 30%
            staked: 70,                              // 70%
            deviation: 5,                            // 5%
            extreme_deviation: 15,                   // 15%
            eval_period: 12 * 60 * 60 * 1000, // Decide on time delay, in seconds, default 12hrs
            eval_cadence: "0 0 * * * *".to_string(), // OR cron cadence
        }
    }
}

// TODO:
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
        self.assert_owner();
        let current_pool = self.stake_delegations.get(&pool_account_id);

        // Insert ONLY if there isn't a record of this pool already
        // NOTE: Only managing the stake_delegations, as stake_pending_delegations is used for active balance movements
        assert!(current_pool.is_none(), "Stake pool exists already");
        self.stake_delegations.insert(
            &pool_account_id,
            &StakeDelegation {
                init_balance: 0,
                balance: 0,
                start_block: 0, // 0 indicates that the staking has not started yet
                withdraw_epoch: None,
                withdraw_balance: None,
                withdraw_function: withdraw_function.unwrap_or("withdraw_all".to_string()),
                liquid_unstake_function,
                yield_function,
            },
        );
    }

    /// Remove a pool, if all balances have been withdrawn
    ///
    /// ```bash
    /// near call treasury.testnet remove_staking_pool '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn remove_staking_pool(&mut self, pool_account_id: AccountId) {
        self.assert_owner();
        let current_pool = self.stake_delegations.get(&pool_account_id);

        // Insert ONLY if there isn't a record of this pool already
        // NOTE: Only managing the stake_delegations, as stake_pending_delegations is used for active balance movements
        assert!(current_pool.is_some(), "Stake pool doesnt exist");
        assert_eq!(current_pool.unwrap().balance, 0, "Stake pool has a balance");
        self.stake_delegations.remove(&pool_account_id);
    }

    /// Returns all staking delegations
    ///
    /// ```bash
    /// near view treasury.testnet has_delegation_to_withdraw
    /// ```
    pub fn has_delegation_to_withdraw(&self) -> (bool, Vec<AccountId>) {
        let mut ret: Vec<AccountId> = Vec::new();
        let mut has_withdraw = false;

        // Return all data within range
        let keys = self.stake_pending_delegations.keys_as_vector();
        for pool_account_id in keys.iter() {
            if let Some(delegation) = self.stake_pending_delegations.get(&pool_account_id) {
                // Check if any of the pending delegations have a withdraw epoch older than THIS epoch
                if delegation.withdraw_epoch.expect("No withdraw epoch") < env::epoch_height()
                    && delegation.withdraw_balance.expect("No withdraw balance") > 0
                {
                    has_withdraw = true;
                }

                ret.push(pool_account_id);
            }
        }
        (has_withdraw, ret)
    }

    /// Check staking threshold to find if an auto_stake rebalance should occur
    ///
    /// ```bash
    /// near view treasury.testnet needs_stake_rebalance --accountId manager_v1.croncat.testnet
    /// ```
    pub fn needs_stake_rebalance(&self) -> (bool, u128, u128, u128, u128) {
        let threshold = &self.stake_threshold;
        let current_balance = env::account_balance();
        let mut staked_balance: Balance = 0;
        let mut unstaking_balance: Balance = 0;

        // get total staked balance
        for (_, stake) in self.stake_delegations.iter() {
            staked_balance = staked_balance.saturating_add(stake.balance);
        }
        // get total unstaking balance
        for (_, unstake) in self.stake_pending_delegations.iter() {
            unstaking_balance =
                unstaking_balance.saturating_add(unstake.withdraw_balance.unwrap_or(0));
        }

        // update total balance, so we can check thresholds
        // TODO: Check if taking unstaking balance into consideration is a bad idea realistically
        let total_balance: Balance =
            current_balance.saturating_add(staked_balance.saturating_add(unstaking_balance));

        // Compute threshold values
        let liquid_ideal: u128 =
            utils::calc_percent(threshold.liquid, threshold.denominator, total_balance);
        let liquid_actual: u128 = utils::calc_percent(1, threshold.denominator, current_balance);
        let liquid_deviation: u128 =
            utils::calc_percent(threshold.deviation, threshold.denominator, total_balance);
        let liquid_extreme_deviation: u128 = utils::calc_percent(
            threshold.extreme_deviation,
            threshold.denominator,
            total_balance,
        );

        // Check if liquid balance is above threshold deviation
        if liquid_actual > liquid_ideal.saturating_add(liquid_deviation) {
            return (
                true,
                liquid_actual,
                liquid_ideal,
                liquid_deviation,
                liquid_extreme_deviation,
            );
        }

        // Check if liquid balance is below threshold deviation
        if (liquid_actual < liquid_ideal.saturating_sub(liquid_deviation))
            || (liquid_actual < liquid_ideal.saturating_sub(liquid_extreme_deviation))
        {
            return (
                true,
                liquid_actual,
                liquid_ideal,
                liquid_deviation,
                liquid_extreme_deviation,
            );
        }

        return (
            false,
            liquid_actual,
            liquid_ideal,
            liquid_deviation,
            liquid_extreme_deviation,
        );
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
        // Check if approved caller
        assert!(
            env::predecessor_account_id() == self.owner_id
                || env::predecessor_account_id() == self.croncat_id.clone().unwrap(),
            "Not an approved caller"
        );
        let (
            needs_rebalance,
            liquid_actual,
            liquid_ideal,
            liquid_deviation,
            liquid_extreme_deviation,
        ) = self.needs_stake_rebalance();
        if !needs_rebalance {
            return;
        }

        // Get the staking pool(s) to do things
        // NOTE: For simplicity, just going to get 1 pool
        let pool_id = self
            .stake_delegations
            .keys()
            .next()
            .expect("No pool id found");

        // Check if liquid balance is above threshold deviation
        if liquid_actual > liquid_ideal.saturating_add(liquid_deviation) {
            // Time to restake some amount
            self.deposit_and_stake(
                pool_id.clone(),
                Some(U128::from(liquid_ideal.saturating_sub(liquid_actual))),
            );
        }

        // Check if liquid balance is below threshold deviation
        if liquid_actual < liquid_ideal.saturating_sub(liquid_deviation) {
            // Time to unstake some amount
            if liquid_actual < liquid_ideal.saturating_sub(liquid_extreme_deviation) {
                let unstake_amount = Some(U128::from(
                    liquid_extreme_deviation.saturating_sub(liquid_actual),
                ));
                let pool = self
                    .stake_delegations
                    .get(&pool_id.clone())
                    .expect("No delegation found for pool");
                // If pool supports liquid unstaking, otherwise go to regular
                if pool.liquid_unstake_function.is_some() {
                    self.liquid_unstake(pool_id, unstake_amount);
                } else {
                    self.unstake(pool_id, unstake_amount);
                }
            } else {
                self.unstake(
                    pool_id,
                    Some(U128::from(
                        liquid_extreme_deviation.saturating_sub(liquid_actual),
                    )),
                );
            }
        }
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
    pub fn deposit_and_stake(&mut self, pool_account_id: AccountId, amount: Option<U128>) {
        self.assert_owner();
        let mut stake_amount: Balance = 0;
        let pool_delegation = self.stake_delegations.get(&pool_account_id);
        assert!(pool_delegation.is_some(), "Stake delegation doesnt exist");

        if env::attached_deposit() > 0 {
            stake_amount = env::attached_deposit();
        } else {
            assert!(
                u128::from(env::account_balance())
                    .saturating_sub(amount.unwrap_or(U128::from(0)).0)
                    > MIN_BALANCE_FOR_STORAGE,
                "Account Balance Under Minimum Balance"
            );
            if let Some(amount) = amount {
                stake_amount = amount.0;
            }
        }

        // Stop if somehow we made it this far and have nothing to stake... RUDE
        assert_ne!(stake_amount, 0, "Nothing to stake");
        log!("Amount to stake {:?}", &stake_amount);

        let delegation = pool_delegation.unwrap();
        let updated_delegation = StakeDelegation {
            init_balance: stake_amount,
            balance: stake_amount,
            start_block: env::block_height(),
            withdraw_epoch: None,
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
    /// near call treasury.testnet get_staked_balance '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
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
                let mut delegation = self
                    .stake_delegations
                    .get(&pool_account_id)
                    .expect("No delegation found");

                // Update internal balances
                delegation.balance = pool_balance.staked_balance.0;
                delegation.withdraw_balance = Some(pool_balance.unstaked_balance.0);

                // If its known, immediately make withdraw available, otherwise compute when withdraw is available
                if pool_balance.can_withdraw {
                    delegation.withdraw_epoch = Some(env::epoch_height() + 4);
                }

                // Update the balances of pool
                self.stake_delegations.insert(&pool_account_id, &delegation);
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
    pub fn unstake(&mut self, pool_account_id: AccountId, amount: Option<U128>) {
        self.assert_owner();
        let pool_delegation = self.stake_delegations.get(&pool_account_id);
        assert!(pool_delegation.is_some(), "Stake delegation doesnt exist");
        let mut unstake_function = "unstake_all";

        // Stop if somehow we made it this far and have nothing to unstake... RUDE
        if amount.is_some() {
            assert_ne!(amount.unwrap().0, 0, "Nothing to unstake");
            unstake_function = "unstake";
        }

        // Update our local balance values, so we know whats in process of long-form unstaking
        let mut delegation = pool_delegation.unwrap();
        let withdraw_balance = amount.unwrap_or(U128::from(0)).0;
        delegation.withdraw_epoch = Some(env::epoch_height() + 4);
        delegation.withdraw_balance = Some(withdraw_balance);
        self.stake_pending_delegations
            .insert(&pool_account_id, &delegation);

        // Lastly, make the cross-contract call to DO the unstaking :D
        let p = env::promise_create(
            pool_account_id.clone(),
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
        // NOTE: Could also add this as a timeout action!
        if self.croncat_id.is_some() {
            external::croncat::create_task(
                env::current_account_id().to_string(),
                "withdraw".to_string(),
                // TODO: what cadence is needed here? (this sets it to every friday at minute 0), ideally can set a block height start
                "* * * * * Fri".to_string(),
                Some(false),
                Some(U128::from(NO_DEPOSIT)),
                Some(u64::from(GAS_STAKE_WITHDRAW_ALL + GAS_CRONCAT_CREATE_TASK)), // 70 Tgas
                Some(Base64VecU8::from(
                    json!({
                        "pool_account_id": pool_account_id,
                    })
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                )),
                self.croncat_id.clone().unwrap(),
                CRONCAT_CREATE_TASK_FEE,
                GAS_CRONCAT_CREATE_TASK,
            );
        }

        env::promise_return(p);
    }

    /// Withdraw unstaked balance from a pool, works in metapool and traditional validator pools
    ///
    /// ```bash
    /// near call treasury.testnet withdraw '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn withdraw(&mut self, pool_account_id: AccountId) {
        self.assert_owner();
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
        pool_delegation.balance = pool_delegation
            .balance
            .saturating_sub(pending_pool_delegation.withdraw_balance.unwrap_or(0));
        log!("pool_delegation.balance {:?}", pool_delegation.balance);
        self.stake_pending_delegations.remove(&pool_account_id);
        self.stake_delegations
            .insert(&pool_account_id, &pool_delegation);

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
    /// NOTE: This is a 3 part process:
    ///       1. Get the staking balance
    ///       2. Get the amount available to withdraw
    ///       3. Make the actual liquid_unstaking
    /// NOTE: The amount is the liquid staked asset, not NEAR
    ///
    /// ```bash
    /// near call treasury.testnet liquid_unstake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    pub fn liquid_unstake(&mut self, pool_account_id: AccountId, amount: Option<U128>) {
        self.assert_owner();
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

    /// 2. CALLBACK for get_account_info - which returns the amount of near staked
    #[private]
    pub fn callback_liquid_unstake(&mut self, pool_account_id: AccountId, amount: Option<U128>) {
        is_promise_success();

        // Return balance or 0
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Successful(result) => {
                // Attempt to parse the returned account balances
                let pool_balance: external::MetaPoolBalance = serde_json::de::from_slice(&result)
                    .expect("Could not get balance from stake pool");
                log!("pool_balance {:?}", pool_balance);

                // Double check values before going forward
                assert!(pool_balance.st_near.0 > 0, "No st_near balance");
                assert!(
                    pool_balance.valued_st_near.0 > 0,
                    "No valued_st_near balance"
                );
                let st_near: U128 = pool_balance.st_near;

                // First check if there are any staked balances
                let p1 = env::promise_create(
                    pool_account_id.clone(),
                    "get_near_amount_sell_stnear",
                    json!({
                        "stnear_to_sell": st_near,
                    })
                    .to_string()
                    .as_bytes(),
                    NO_DEPOSIT,
                    GAS_STAKE_LIQUID_UNSTAKE_FEE_VIEW,
                );

                let p2 = env::promise_then(
                    p1,
                    env::current_account_id(),
                    "callback_liquid_unstake_final",
                    json!({
                        "pool_account_id": pool_account_id,
                        "amount": amount,
                        "st_near": st_near,
                    })
                    .to_string()
                    .as_bytes(),
                    NO_DEPOSIT,
                    GAS_STAKE_LIQUID_UNSTAKE_CALLBACK_FINAL,
                );

                env::promise_return(p2);
            }
            PromiseResult::Failed => {
                // Fail me not, please
            }
        }
    }

    /// 3. CALLBACK for get_near_amount_sell_stnear - which returns the amount of near that can be unstaked
    #[private]
    pub fn callback_liquid_unstake_final(
        &mut self,
        pool_account_id: AccountId,
        amount: Option<U128>,
        st_near: U128,
    ) {
        is_promise_success();

        // Return balance or 0
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Successful(result) => {
                // Attempt to parse the returned available balance
                let mut pool_min_expected_near: U128 = serde_json::de::from_slice(&result)
                    .expect("Could not get amount from stake pool");

                // Double check values before going forward
                assert!(pool_min_expected_near.0 > 0, "No st_near to unstake");
                let mut st_near_to_burn = st_near;

                // If no amount specified, simply unstake all st_near
                // IF amount, compare to use the maximum unstakable that matches user amounts
                // TODO: This is always returning with less NEAR than expected because of fees being taken out, need to add fee to amount so ratio can include fee
                if amount.is_some() {
                    // Get amount ratio, then desired st_near from ratio
                    // limit to maximum st_near balance
                    let denominator = 100;
                    let desired_amount = u128::from(amount.unwrap().0).saturating_mul(denominator);
                    let ratio = desired_amount.div_euclid(pool_min_expected_near.0);
                    let st_unstake_amount =
                        u128::from(st_near.0.saturating_mul(ratio)).div_euclid(denominator);
                    if st_near.0 > st_unstake_amount {
                        st_near_to_burn = U128::from(st_unstake_amount);
                        pool_min_expected_near = U128::from(
                            u128::from(pool_min_expected_near.0.saturating_mul(ratio))
                                .div_euclid(denominator),
                        );
                    }
                }

                // We have some balances, attempt to unstake
                let delegation = self
                    .stake_delegations
                    .get(&pool_account_id)
                    .expect("Delegation doesnt exist");
                let p1 = env::promise_create(
                    pool_account_id.clone(),
                    &delegation.liquid_unstake_function.unwrap(),
                    json!({
                        "st_near_to_burn": st_near_to_burn,
                        "min_expected_near": pool_min_expected_near,
                    })
                    .to_string()
                    .as_bytes(),
                    NO_DEPOSIT,
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
        self.assert_owner();
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
