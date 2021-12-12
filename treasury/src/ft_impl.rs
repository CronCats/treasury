use crate::*;

pub const GAS_FT_TRANSFER: Gas = Gas(10_000_000_000_000);
pub const GAS_FT_BALANCE_OF: Gas = Gas(10_000_000_000_000);
pub const GAS_FT_BALANCE_OF_CALLBACK: Gas = Gas(10_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleTokenBalance {
    pub account_id: AccountId,
    pub balance: Balance,
}

// TODO:
// * storage deposit???

#[near_bindgen]
impl Contract {
    /// Supported Fungible Tokens
    /// 
    /// ```bash
    /// near call treasury.testnet get_ft_list
    /// ```
    pub fn get_ft_list(&self) -> Vec<AccountId> {
        self.ft_balances.keys_as_vector().to_vec()
    }

    /// Fungible Token Balances
    /// 
    /// ```bash
    /// near call treasury.testnet ft_balances '{"from_index": 0, "limit": 10}'
    /// ```
    pub fn ft_balances(
        &self,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<FungibleTokenBalance> {
        let mut result: Vec<FungibleTokenBalance> = Vec::new();
        let mut start = 0;
        let mut end = 10;
        
        if let Some(from_index) = from_index {
            start = from_index.0;
        }
        if let Some(limit) = limit {
            end = u64::min(start + limit.0, self.ft_balances.len());
        }

        // Return all tasks within range
        let keys = self.ft_balances.keys_as_vector();
        for i in start..end {
            if let Some(account_id) = keys.get(i) {
                if let Some(balance) = self.ft_balances.get(&account_id) {
                    result.push(FungibleTokenBalance { account_id, balance });
                }
            }
        }

        result
    }

    /// Single Fungible Token Balance
    /// NOTE: Unlike the FT standard, this account_id is the "fungible token account id"
    /// 
    /// ```bash
    /// near call treasury.testnet ft_balance_of '{"account_id": "wrap.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn ft_balance_of(
        &self,
        account_id: AccountId,
    ) -> U128 {
        U128::from(self.ft_balances.get(&account_id).unwrap_or(0))
    }

    /// Transfer Fungible Token
    /// NOTE: Assumes storage deposit has occurred for recipient
    /// 
    /// ```bash
    /// near call treasury.testnet ft_transfer '{"ft_account_id": "wrap.testnet", "to_account_id": "user.account.testnet", "to_amount": "100000000000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    pub fn ft_transfer(
        &mut self,
        ft_account_id: AccountId,
        to_amount: U128,
        to_account_id: AccountId,
    ) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");

        // Check if treasury holds the ft, and has enough balance
        let ft_balance = self.ft_balances.get(&ft_account_id).expect("No token balance found");
        assert!(ft_balance >= to_amount.0, "Transfer amount too high");

        // NOTE: Lame accounting here, should be changed to callback
        let mut total = to_amount.0;
        total = total.saturating_add(ft_balance);
        self.ft_balances.insert(&env::predecessor_account_id(), &total);
        
        let p = env::promise_create(
            ft_account_id,
            "ft_transfer",
            json!({
                "receiver_id": to_account_id,
                "amount": to_amount.0,
            }).to_string().as_bytes(),
            ONE_YOCTO,
            GAS_FT_TRANSFER
        );

        env::promise_return(p);
    }

    /// Get & Store Fungible Token Balance
    /// Note: Would be epic if we could get auto-notified of this...
    /// 
    /// ```bash
    /// near call treasury.testnet store_ft_balance_of '{"ft_account_id": "wrap.testnet"}' --accountId treasury.testnet
    /// ```
    pub fn store_ft_balance_of(&mut self, ft_account_id: AccountId) {
        let p1 = env::promise_create(
            ft_account_id.clone().into(),
            "ft_balance_of",
            json!({
                "account_id": env::current_account_id().to_string(),
            }).to_string().as_bytes(),
            NO_DEPOSIT,
            GAS_FT_BALANCE_OF
        );

        env::promise_then(
            p1,
            env::current_account_id(),
            "store_ft_balance_of_callback",
            json!({
                "ft_account_id": ft_account_id.to_string(),
            }).to_string().as_bytes(),
            NO_DEPOSIT,
            GAS_FT_BALANCE_OF_CALLBACK
        );
    }

    #[private]
    pub fn store_ft_balance_of_callback(&mut self, ft_account_id: AccountId) {
        assert_eq!(env::promise_results_count(), 1, "Expected 1 promise result.");

        // Return balance or 0
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Successful(result) => {
                // Attempt to parse the returned balance amount
                let amount: U128 = serde_json::de::from_slice(&result)
                    .expect("Could not get balance from fungible token");

                // Update the token balance
                self.ft_balances.insert(&ft_account_id, &amount.0);
            }
            PromiseResult::Failed => {}
        }
    }

    /// Compute Fungible Token Balances for Supported FTs
    /// 
    /// ```bash
    /// near call treasury.testnet compute_ft_balances '{"from_index": 0, "limit": 10}'
    /// ```
    pub fn compute_ft_balances(
        &mut self,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) {
        let mut start = 0;
        let mut end = 10;
        
        if let Some(from_index) = from_index {
            start = from_index.0;
        }
        if let Some(limit) = limit {
            end = u64::min(start + limit.0, self.ft_balances.len());
        }

        // Return all tasks within range
        let keys = self.ft_balances.keys_as_vector();
        for i in start..end {
            if let Some(account_id) = keys.get(i) {
                // TODO: Trigger promise to get FT balance
                log!("get balance for {:?}", account_id);
            }
        }
    }
}