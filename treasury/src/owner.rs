use crate::*;

#[near_bindgen]
impl Contract {
    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            self.owner_id,
            env::predecessor_account_id(),
            "Must be owner"
        );
    }

    /// Changes core configurations
    /// Should only be updated by owner -- in best case DAO based :)
    pub fn update_settings(
        &mut self,
        owner_id: Option<AccountId>,
        croncat_id: Option<AccountId>,
        stake_threshold: Option<StakeThreshold>,
    ) {
        self.assert_owner();

        // BE CAREFUL!
        if let Some(owner_id) = owner_id {
            self.owner_id = owner_id;
        }
        if let Some(croncat_id) = croncat_id {
            self.croncat_id = Some(croncat_id);
        }

        // Staking Settings
        if let Some(stake_threshold) = stake_threshold {
            self.stake_threshold = stake_threshold;
        }
    }

    /// Manage payable account
    /// NOTE: Not specifying means ANY account can be paid
    ///
    /// ```bash
    /// near call treasury.testnet add_payable_account '{"account_id": "steak.testnet"}' --accountId treasury.testnet --depositYocto 1
    /// ```
    #[payable]
    pub fn add_payable_account(&mut self, account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        let account = self.stake_delegations.get(&account_id);

        // Insert ONLY if there isn't a record of this already
        assert!(account.is_none(), "Payable account exists already");
        self.approved_accounts_payable.insert(&account_id);
    }

    /// Remove a payable account
    /// NOTE: Why 1 yocto? I feel it to have the same reason as add.
    ///
    /// ```bash
    /// near call treasury.testnet remove_payable_account '{"account_id": "steak.testnet"}' --accountId treasury.testnet --depositYocto 1
    /// ```
    #[payable]
    pub fn remove_payable_account(&mut self, account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        let account = self.stake_delegations.get(&account_id);

        // Insert ONLY if there isn't a record of this already
        assert!(account.is_none(), "Payable account exists already");
        self.approved_accounts_payable.remove(&account_id);
    }

    /// Transfer funds away from treasury
    /// NOTE: Only used for native currency (NEAR)
    /// NOTE: Should be used with caution, as this method could drain all funds easily
    ///
    /// ```bash
    /// near call treasuriy.testnet transfer '{"receiver_id": "steak.testnet", "amount": "1000000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    pub fn transfer(&mut self, receiver_id: AccountId, amount: U128) -> Promise {
        self.assert_owner();

        // Check approved accounts if any are specified, otherwise allow any
        if self.approved_accounts_payable.len() > 0 {
            assert!(
                self.approved_accounts_payable.contains(&receiver_id),
                "Account restricted, needs approval"
            );
        }

        Promise::new(receiver_id).transfer(amount.0)
    }
}
