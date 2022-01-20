use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, Base58CryptoHash, U128, U64};
use near_sdk::{AccountId};

use crate::*;

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    method_name: String,
    args: Base64VecU8,
    deposit: U128,
    gas: U64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    Staking,
    Unstaking,
    Budget,
    Swap,
    Harvest,

    /// Calls `receiver_id` with list of method names in a single promise.
    /// Allows this contract to execute any arbitrary set of actions in other contracts.
    FunctionCall {
        receiver_id: AccountId,
        actions: Vec<ActionCall>,
    },

    /// Upgrade this contract with given hash from blob store.
    UpgradeSelf { hash: Base58CryptoHash },

    /// Upgrade another contract, by calling method with the code from given hash from blob store.
    UpgradeRemote {
        receiver_id: AccountId,
        method_name: String,
        hash: Base58CryptoHash,
    },

    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    /// If `msg` is not None, calls `ft_transfer_call` with given `msg`. Fails if this base token.
    /// For `ft_transfer` and `ft_transfer_call` `memo` is the `description` of the proposal.
    Transfer {
        /// Can be "" for $NEAR or a valid account id.
        #[serde(with = "serde_with::rust::string_empty_as_none")]
        token_id: Option<AccountId>,
        receiver_id: AccountId,
        amount: U128,
        msg: Option<String>,
    },
}

// impl Action {
//     pub fn log(&self) -> String {
//         format!("{:?}", self)
//     }
// }