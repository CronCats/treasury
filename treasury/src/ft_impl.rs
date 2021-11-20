use crate::*;

pub const GAS_FT_TRANSFER: Gas = 10_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleTokenBalance {
    pub account_id: AccountId,
    pub balance: Balance,
}

// TODO:
// * storage deposit???

// #[near_bindgen]
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
    /// NOTE: Unlike the FT standard, this account_id is the "fungible token account id"
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
            b"ft_transfer",
            json!({
                "receiver_id": to_account_id,
                "amount": to_amount.0,
            }).to_string().as_bytes(),
            ONE_YOCTO,
            GAS_FT_TRANSFER
        );

        env::promise_return(p);
    }
    
    /// Receive Fungible tokens
    /// keep track of the FTs sent to this treasury
    /// NOTE: Should only be triggered by FT standards
    pub fn ft_on_transfer(
        &mut self,
        // sender_id
        _: ValidAccountId,
        amount: U128,
        msg: String,
    ) {
        let ft_balance = self.ft_balances.get(&env::predecessor_account_id());
        log!("{} {}, msg: {:?}", amount.0, &env::predecessor_account_id(), msg);

        // NOTE: Could re-evaluate to just use the token contracts for balance totals
        if ft_balance.is_none() {
            self.ft_balances.insert(&env::predecessor_account_id(), &amount.0);
        } else {
            let mut total = amount.0;
            if let Some(ft_balance) = ft_balance {
                total = total.saturating_add(ft_balance);
            }
            self.ft_balances.insert(&env::predecessor_account_id(), &total);
        }
    }
}