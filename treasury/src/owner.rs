use crate::*;

// // Staking
// stake_threshold_percentage: u128,
// stake_eval_period: u128, // Decide on time delay, in seconds
// stake_eval_cadence: String, // OR cron cadence

#[near_bindgen]
impl Contract {
    /// Changes core configurations
    /// Should only be updated by owner -- in best case DAO based :)
    pub fn update_settings(
        &mut self,
        owner_id: Option<AccountId>,
        stake_threshold_percentage: Option<U128>,
        stake_eval_period: Option<U128>,
        stake_eval_cadence: Option<String>,
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

        // Staking Settings
        if let Some(stake_threshold_percentage) = stake_threshold_percentage {
            self.stake_threshold_percentage = stake_threshold_percentage.0;
        }
        if let Some(stake_eval_period) = stake_eval_period {
            self.stake_eval_period = stake_eval_period.0;
        }
        if let Some(stake_eval_cadence) = stake_eval_cadence {
            self.stake_eval_cadence = stake_eval_cadence;
        }
    }
}
