use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

//////////////////////////////////////////////////////////////////////
/// CONFIG
//////////////////////////////////////////////////////////////////////

/// ## Description
/// This structure holds the contract parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Master address who can update tollgate / status of all vestings
    pub master_address: Addr,
    /// Specific vesting denom
    pub denom: String,
    /// Start time of this vesting contract, i.e. contract init time
    pub vesting_start_time: u64,
}

/// ## Description
/// A custom struct for each query response that returns general contract settings/configs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// Master address who can update tollgate / status of all vestings
    pub master_address: String,
    /// Specific vesting denom
    pub denom: String,
    /// Start time of this vesting contract, i.e. contract init time
    pub vesting_start_time: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

//////////////////////////////////////////////////////////////////////
/// VESTING
//////////////////////////////////////////////////////////////////////

/// ## Description
/// This structure holds the initial vesting parameters for each protocol.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vesting {
    /// The address of the recipient protocol to approve the tollgate for
    pub recipient: String,
    /// Vesting amount
    pub amount: Uint128,
}

//////////////////////////////////////////////////////////////////////
/// VESTING INFO
//////////////////////////////////////////////////////////////////////

/// ## Description
/// This structure holds the vesting information of each protocol.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingInfo {
    /// The address of the recipient protocol to approve the tollgate for
    pub recipient: Addr,
    /// Vesting valid status
    pub active: bool,
    /// Current approved tollgates, in periods
    pub approved_periods: u64,
    /// Total vesting periods
    pub total_periods: u64,
    /// Previously claimed period, start at 0
    pub last_claimed_period: u64,
    /// Total vesting amount
    pub total_amount: Uint128,
    /// Claimed vesting amount
    pub claimed_amount: Uint128,
    /// Unclaimed amount
    pub vested_amount: Uint128,
    /// Claimable amount for each period
    pub amount_per_period: Uint128,
}

pub const VESTING_INFO: Map<&Addr, VestingInfo> = Map::new("vesting_info");
