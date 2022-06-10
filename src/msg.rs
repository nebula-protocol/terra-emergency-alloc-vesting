use crate::state::Vesting;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure stores the basic settings for creating a new vesting contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Master address who can update tollgate / status of all vestings
    pub master_address: Option<String>,
    /// Specific vesting denom
    pub denom: String,
    /// A list of vestings
    pub vestings: Vec<Vesting>,
}

/// ## Description
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /////////////////////
    /// MASTER CALLABLE
    /////////////////////

    /// ApproveTollgate either increments tollgate or deactivate a vesting.
    ApproveTollgate {
        /// Recipient address of a protocol
        recipient: String,
        /// New vesting status
        approve: bool,
    },

    /////////////////////
    /// USER CALLABLE
    /////////////////////

    /// Claim unlocked vesting
    Claim {},
}

/// ## Description
/// This structure describes the available query messages for the vesting contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// VestingInfo returns the vesting information of the specified recipient
    VestingInfo {
        /// Recipient address of a protocol
        recipient: String,
    },
}

/// ## Description
/// A struct used for migrating contracts.
/// Currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
