use crate::*;

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolBalance {
    pub account_id: AccountId,
    pub unstaked_balance: U128,
    pub staked_balance: U128,
    pub can_withdraw: bool,
}

/// REF: https://github.com/Narwallets/meta-pool/blob/master/metapool/src/types.rs#L117
#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MetaPoolBalance {
    pub account_id: AccountId,
    pub available: U128,
    pub st_near: U128,
    pub valued_st_near: U128, // st_near * stNEAR_price
    pub meta: U128,
    pub realized_meta: U128,
    pub unstaked: U128,
    pub unstaked_requested_unlock_epoch: U64,
    pub unstake_full_epochs_wait_left: u16,
    pub can_withdraw: bool,
    pub total: U128,
    pub trip_start: U64,
    pub trip_start_stnear: U128,
    pub trip_accum_stakes: U128,
    pub trip_accum_unstakes: U128,
    pub trip_rewards: U128,
    pub nslp_shares: U128,
    pub nslp_share_value: U128,
    pub nslp_share_bp: u16,
}


/// CRONCAT
#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct CroncatTask {
    pub contract_id: AccountId,
    pub function_id: String,
    pub cadence: String,
    pub recurring: bool,
    pub deposit: U128,
    pub gas: Gas,
    pub arguments: Vec<u8>,
}

#[ext_contract(croncat)]
pub trait Croncat {
    fn get_slot_tasks(&self, offset: Option<u64>) -> (Vec<Base64VecU8>, U128);
    fn get_tasks(
        &self,
        slot: Option<U128>,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<CroncatTask>;
    // fn get_task(&self, task_hash: Base64VecU8) -> Task;
    fn get_task(&self, task_hash: String) -> CroncatTask;
    fn create_task(
        &mut self,
        contract_id: String,
        function_id: String,
        cadence: String,
        recurring: Option<bool>,
        deposit: Option<U128>,
        gas: Option<Gas>,
        arguments: Option<Vec<u8>>,
    ) -> Base64VecU8;
    fn remove_task(&mut self, task_hash: Base64VecU8);
    fn proxy_call(&mut self);
    fn get_info(
        &mut self,
    ) -> (
        bool,
        AccountId,
        U64,
        U64,
        [u64; 2],
        U128,
        U64,
        U64,
        U128,
        U128,
        U128,
        U128,
        U64,
        U64,
        U64,
        U128,
    );
}