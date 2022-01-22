use std::collections::VecDeque;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{TreeMap, UnorderedMap, UnorderedSet},
    env, ext_contract,
    json_types::{Base64VecU8, U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json,
    serde_json::json,
    AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseOrValue,
    PromiseResult,
    utils::is_promise_success,
};
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

mod actions;
mod external;
mod owner;
mod staking;
mod utils;
mod views;
// mod storage_impl;
mod ft_impl;
mod nft_impl;

use actions::Action;
use staking::{StakeDelegation, StakeThreshold};

// Balance & Fee Definitions
pub const NO_DEPOSIT: Balance = 0;
/// 1 yN to prevent access key fraud.
pub const ONE_YOCTO: Balance = 1;
pub const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const GAS_BASE_PRICE: Balance = 100_000_000;
pub const GAS_BASE_FEE: Gas = Gas(3_000_000_000_000);
/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);
pub const STAKE_BALANCE_MIN: u128 = 10 * ONE_NEAR;
pub const MIN_BALANCE_FOR_STORAGE: u128 = 20 * ONE_NEAR;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    ActionsCadence,
    ActionsTimeout,
    ActionsApproved,
    FungibleTokenBalances,
    NonFungibleTokenHoldings,
    StakePools,
    StakePoolsPending,
    YieldFunctions,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // Runtime
    paused: bool,
    owner_id: AccountId, // single or DAO entity
    // approved_signees: Option<UnorderedSet<AccountId>>, // Allows potential multisig instance, can be DAO or members
    // signer_threshold: Option<[u32; 2]>, // allows definitions of threshold for signatures, example: 3/5 signatures

    // General Config
    approved_action_types: UnorderedSet<String>,

    // Croncat Scheduling Config
    croncat_id: Option<AccountId>,
    cadence_actions: UnorderedMap<String, Action>, // recurring items, using croncat cadence, only allowing 1 action per cadence for simplicity of non-pagination
    timeout_actions: TreeMap<u128, VecDeque<Action>>, // single trigger items, using croncat trigger upon a timeout/future timestamp

    // Token Standards
    ft_balances: UnorderedMap<AccountId, u128>,
    nft_holdings: UnorderedMap<AccountId, Vec<String>>,

    // Staking
    stake_threshold: StakeThreshold,
    stake_delegations: UnorderedMap<AccountId, StakeDelegation>, // for near staking, can be metapool, or other pools directly
    stake_pending_delegations: UnorderedMap<AccountId, StakeDelegation>, // for withdraw near staking

                                                                         // Yield harvesting
                                                                         // yield_functions: LookupMap<AccountId, String>, // Storage
                                                                         // ft_storage_usage: StorageUsage,
                                                                         // nft_storage_usage: StorageUsage
}

#[near_bindgen]
impl Contract {
    /// Initialize the contracts defaults, should be done from deploy
    ///
    /// ```bash
    /// near call treasury.testnet new --accountId treasury.testnet
    /// ```
    #[init]
    pub fn new() -> Self {
        Contract {
            paused: false,
            owner_id: env::signer_account_id(),
            approved_action_types: UnorderedSet::new(StorageKeys::ActionsApproved),
            ft_balances: UnorderedMap::new(StorageKeys::FungibleTokenBalances),
            nft_holdings: UnorderedMap::new(StorageKeys::NonFungibleTokenHoldings),
            croncat_id: None,
            cadence_actions: UnorderedMap::new(StorageKeys::ActionsCadence),
            timeout_actions: TreeMap::new(StorageKeys::ActionsTimeout),
            stake_threshold: StakeThreshold::default(),
            stake_delegations: UnorderedMap::new(StorageKeys::StakePools), // for near staking, can be metapool, or other pools directly
            stake_pending_delegations: UnorderedMap::new(StorageKeys::StakePoolsPending), // for withdraw near staking
                                                                                          // yield_functions: LookupMap::new(StorageKeys::YieldFunctions),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::json_types::AccountId;
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::{testing_env, MockedBlockchain};

//     const BLOCK_START_BLOCK: u64 = 52_201_040;
//     const BLOCK_START_TS: u64 = 1_624_151_503_447_000_000;

//     fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
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
