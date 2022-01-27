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

    // TODO: Setup a way for approved accounts to get transfers of NEAR, consider config for approved accounts OR all
}
