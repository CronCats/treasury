use crate::*;

#[near_bindgen]
impl Contract {
    // NOTE: For large state transitions, needs to be able to migrate over paginated sets?
    /// Migrate State
    /// Safely upgrade contract storage
    ///
    /// ```bash
    /// near call cron.testnet migrate_state --accountId cron.testnet
    /// ```
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: Contract = env::state_read().expect("Old state doesn't exist");
        // Verify that the migration can only be done by the owner.
        // This is not necessary, if the upgrade is done internally.
        assert_eq!(
            &env::predecessor_account_id(),
            &old_contract.owner_id,
            "Can only be called by the owner"
        );

        // Create the new contract using the data from the old contract.
        // Contract { owner_id: old_contract.owner_id, data: old_contract.data, new_data }
        Contract {
            paused: false,
            owner_id: old_contract.owner_id,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::json_types::ValidAccountId;
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::{testing_env, MockedBlockchain};

//     const BLOCK_START_TS: u64 = 1633759320000000000;

//     fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .signer_account_pk(b"ed25519:4ZhGmuKTfQn9ZpHCQVRwEr4JnutL8Uu3kArfxEqksfVM".to_vec())
//             .predecessor_account_id(predecessor_account_id)
//             .block_timestamp(BLOCK_START_TS);
//         builder
//     }

//     // TODO: Add test for checking pending agent here.
//     #[test]
//     fn test_tick() {
//         let mut context = get_context(accounts(1));
//         testing_env!(context.is_view(false).build());
//         let mut contract = Contract::new();
//         testing_env!(context.is_view(true).build());
//         testing_env!(context
//             .is_view(false)
//             .block_timestamp(1633759440000000000)
//             .build());
//         contract.tick();
//         testing_env!(context
//             .is_view(false)
//             .block_timestamp(1633760160000000000)
//             .build());
//         contract.tick();
//         testing_env!(context
//             .is_view(false)
//             .block_timestamp(1633760460000000000)
//             .build());
//         testing_env!(context.is_view(true).build());
//     }
// }
