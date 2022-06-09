use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub master_address: Addr,
    pub community_pool_address: Addr,
    pub denom: String,
    pub vesting_start_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vesting {
    pub recipient: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingInfo {
    pub recipient: Addr,
    pub active: bool,
    pub approved_periods: u64,
    pub total_periods: u64,
    pub last_claimed_period: u64,
    pub total_amount: Uint128,
    pub claimed_amount: Uint128,
    pub vested_amount: Uint128,
    pub amount_per_period: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const VESTING_INFO: Map<&Addr, VestingInfo> = Map::new("loan_info");
