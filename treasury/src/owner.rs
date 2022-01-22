use crate::*;

// // Staking
// stake_threshold_percentage: u128,
// stake_eval_period: u128, // Decide on time delay, in seconds
// stake_eval_cadence: String, // OR cron cadence

#[near_bindgen]
impl Contract {

    pub(crate) fn assert_owner(&self) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");
    }

    /// Changes core configurations
    /// Should only be updated by owner -- in best case DAO based :)
    pub fn update_settings(
        &mut self,
        owner_id: Option<AccountId>,
        croncat_id: Option<AccountId>,
        stake_threshold: Option<StakeThreshold>,
    ) {
        assert_eq!(
            self.owner_id,
            env::predecessor_account_id(),
            "Must be owner"
        );

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
}
