use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cosmwasm_std::{StdResult, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read};

/// config: Config
static KEY_CONFIG: &[u8] = b"config";
/// asset data: VestingInfo
pub static VESTING_INFO_KEY: &[u8] = b"vesting_info";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub master_address: Addr,
    pub community_pool_address: Addr,
    pub denom: String,
    pub vesting_start_time: u64,
    pub seconds_per_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vesting {
    pub recipient: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingInfo {
    pub recipient: Addr,
    pub total_amount: Uint128,
    pub vesting_duration: u64,
    pub last_claimed: u64,
    pub claimed_amount: Uint128,
    pub vested_amount: Uint128,
    pub active: bool,
    pub tollgates_required: u64,
    pub tollgates_approved: u64
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn store_vesting_info(
    storage: &mut dyn Storage,
    recipient: &Addr,
    vesting_info: &VestingInfo,
) -> StdResult<()> {
    bucket(storage, VESTING_INFO_KEY).save(recipient.as_bytes(), vesting_info)
}

pub fn read_vesting_info(storage: &dyn Storage, recipient: &Addr) -> StdResult<VestingInfo> {
    Ok(bucket_read(storage, VESTING_INFO_KEY)
        .load(recipient.as_bytes()).unwrap())
}