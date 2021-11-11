use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, TreeMap, UnorderedMap, Vector},
    env,
    json_types::{Base64VecU8, ValidAccountId, U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json::json,
    AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise, StorageUsage,
};

mod owner;
mod utils;
mod views;
mod storage_impl;
mod ft_impl;
mod nft_impl;

near_sdk::setup_alloc!();

// Balance & Fee Definitions
pub const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const GAS_BASE_PRICE: Balance = 100_000_000;
pub const GAS_BASE_FEE: Gas = 3_000_000_000_000;
pub const STAKE_BALANCE_MIN: u128 = 10 * ONE_NEAR;


// #[derive(BorshStorageKey, BorshSerialize)]
// pub enum StorageKeys {
//     Tasks,
// }

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // Runtime
    paused: bool,
    owner_id: AccountId, // single or DAO entity
    approved_signees: Option<UnorderedSet<AccountId>>, // Allows potential multisig instance
    signer_threshold: Option<[u32; 2]>, // allows definitions of threshold for signatures, example: 3/5 signatures

    // Collateral Pools (NOTE: Serious WIP here :D )
    ft_accounts: UnorderedSet<AccountId>,
    ft_balances: UnorderedMap<AccountId, u128>,
    nft_accounts: UnorderedSet<AccountId>,
    nft_holdings: UnorderedMap<AccountId, Vector<u128>>,

    // Staking
    stake_threshold_percentage: u128,
    stake_eval_period: u128, // Decide on time delay, in seconds
    stake_eval_cadence: String, // OR cron cadence
    stake_pools: UnorderedMap<AccountId, u128>, // for near staking, can be metapool, or other pools directly
    stake_pending_pools: UnorderedMap<AccountId, u128>, // for withdraw near staking
    stake_ft: UnorderedMap<AccountId, u128>, // for yield farming
    stake_pending_ft: UnorderedMap<AccountId, u128>, // for withdraw yield farming

    // Collateral Adapters
    // TODO: Figure out how these can become more like plugins for others to extend
    dex_approved_accounts: UnorderedSet<AccountId>, // set accounts that are approved for making swaps
    dex_approved_ft: UnorderedSet<AccountId>, // set the tokens that are approved for making swaps

    // Storage
    // ft_storage_usage: StorageUsage,
    // nft_storage_usage: StorageUsage,
}

#[near_bindgen]
impl Treasury {
    /// ```bash
    /// near call cron.testnet new --accountId cron.testnet
    /// ```
    #[init]
    pub fn new() -> Self {
        let mut this = Treasury {
            paused: false,
            owner_id: env::signer_account_id(),
        };
        // this.measure_account_storage_usage();
        this
    }

    // /// Measure the storage an agent will take and need to provide
    // fn measure_account_storage_usage(&mut self) {
    //     let initial_storage_usage = env::storage_usage();
    //     // Create a temporary, dummy entry and measure the storage used.
    //     let tmp_account_id = "a".repeat(64);
    //     let tmp_agent = Agent {
    //         status: agent::AgentStatus::Pending,
    //         payable_account_id: tmp_account_id.clone(),
    //         balance: U128::from(0),
    //         total_tasks_executed: U128::from(0),
    //         last_missed_slot: 0,
    //     };
    //     self.agents.insert(&tmp_account_id, &tmp_agent);
    //     self.agent_storage_usage = env::storage_usage() - initial_storage_usage;
    //     // Remove the temporary entry.
    //     self.agents.remove(&tmp_account_id);
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::json_types::ValidAccountId;
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::{testing_env, MockedBlockchain};

//     const BLOCK_START_BLOCK: u64 = 52_201_040;
//     const BLOCK_START_TS: u64 = 1_624_151_503_447_000_000;

//     fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .signer_account_pk(b"ed25519:4ZhGmuKTfQn9ZpHCQVRwEr4JnutL8Uu3kArfxEqksfVM".to_vec())
//             .predecessor_account_id(predecessor_account_id)
//             .block_index(BLOCK_START_BLOCK)
//             .block_timestamp(BLOCK_START_TS);
//         builder
//     }

//     #[test]
//     fn test_contract_new() {
//         let mut context = get_context(accounts(1));
//         testing_env!(context.build());
//         let contract = Contract::new();
//         testing_env!(context.is_view(true).build());
//         assert!(contract.get_tasks(None, None, None).is_empty());
//     }
// }
