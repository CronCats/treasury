use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base58CryptoHash, Base64VecU8, U128, U64};
use near_sdk::AccountId;
use utils::assert_owner;

use crate::*;

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    method_name: String,
    args: Base64VecU8,
    deposit: U128,
    gas: U64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
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

    /// Budget is similar to Transfer but with a time component, it also has restrictions for balance boundaries
    /// msg is used to display any optional description or metadata needed for external services (EX: if a budget item is used in a subscription to a service)
    Budget {
        cadence: Option<String>,
        /// Can be "" for $NEAR or a valid token account id.
        #[serde(with = "serde_with::rust::string_empty_as_none")]
        token_id: Option<AccountId>,
        /// NOTE: If sending FT, storage deposit needs to be done outside this budget or it will fail.
        receiver_id: AccountId,
        /// For a whole number to be paid each time
        amount: Option<U128>,
        /// Percent Amount of total account balance at time of payment
        /// For example: If account has 1000 NEAR, amount_percentile is 5%, then payouts would look like: (50, 47.5, 45.125, ...)
        /// NOTE: This does not take into account balance thats staked
        amount_percentile: Option<U128>,
        /// Description of what this budget is for or why, could be metadata if paying a subscription service
        msg: Option<String>,
    },

    // TODO: Add Stake/Unstake as action

    /// Swaps can be made to approved DEXs
    /// NOTE: must comply with storage payments before action can be taken
    Swap {
        // REF Example:
        // "pool_id": 79,
        // "token_in": "token.v2.ref-finance.near",
        // "token_out": "wrap.near",
        // "amount_in": "142445118507604278183",
        // "min_amount_out": "33286939953575500000000000"
        contract_id: AccountId,
        pool_id: u64,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: U128,
        min_amount_out: U128,
    },

    /// Yield/Harvest functionality can be an action based on staked/LP allocations
    /// NOTE: must comply with storage payments before action can be taken
    Harvest {
        contract_id: AccountId,
        method_name: String,
        args: Base64VecU8,
        deposit: U128,
        gas: U64,
    },

    /// Calls `receiver_id` with list of method names in a single promise.
    /// Allows this contract to execute any arbitrary set of actions in other contracts.
    /// NOTE: Should be considered unsafe, as this could lead to many edge cases for bad behaviour
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
}

impl ActionType {
    /// Returns label of policy for given type of proposal.
    pub fn to_label(&self) -> &str {
        match self {
            ActionType::Transfer { .. } => "transfer",
            ActionType::Budget { .. } => "budget",
            ActionType::Swap { .. } => "swap",
            ActionType::Harvest { .. } => "harvest",
            ActionType::FunctionCall { .. } => "function_call",
            ActionType::UpgradeSelf { .. } => "upgrade_self",
            ActionType::UpgradeRemote { .. } => "upgrade_remote",
        }
    }
}

impl Contract {
    pub fn get_action_label(&self, action: ActionType) -> String {
        action.to_label().to_string()
    }

    pub fn add_allowed_actions(&mut self, actions: Vec<ActionType>) {
        assert_owner(&self.owner_id);
        for action in actions.iter() {
            self.approved_action_types.insert(&self.get_action_label(action.clone()));
        }
    }

    pub fn remove_allowed_action(&mut self, action: ActionType) {
        assert_owner(&self.owner_id);
        self.approved_action_types.remove(&self.get_action_label(action));
    }

    /// Returns list of approved actions
    fn get_approved_action_types(&self) -> Vec<String> {
        self.approved_action_types.to_vec()
    }

    /// Returns if an action is allowed or not
    pub fn is_allowed_action(&self, action: ActionType) -> bool {
        self.approved_action_types.contains(&self.get_action_label(action))
    }

    // TODO:
    pub fn create_action(&mut self, action: ActionType) {}

    // TODO:
    pub fn remove_action(&mut self, action: ActionType) {}

    // TODO:
    pub fn get_action(&self, action_id: Base64VecU8) {}

    // TODO:
    pub fn get_actions(&self, from_index: Option<U64>, limit: Option<U64>) {}

    // TODO: Also setup a way for multiple budgets to batch create
    pub fn create_action_budget(&self, action: ActionType) {
        // NOTE: There are 3 scenarios to cover: NEAR/FT 1 time, NEAR/FT recurring, NEAR percentile
        // TODO:
        // - Compute the amount: whole number or percent into whole number
        // - match transfer type
        // - schedule future budget: If no same cadence, create new scheduled task, otherwise slot into cadence bucket
    }
}
