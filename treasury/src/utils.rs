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

    // TODO: Add a generic proxy, so future actions do not require custom integrations.
    // TODO: Could re-use the generic setup for approvals in multisig setup
}