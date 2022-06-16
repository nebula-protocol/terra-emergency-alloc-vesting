use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Mismatched asset type sent and distributed")]
    MismatchedAssetType {},

    #[error("Mismatched asset amount sent and distributed")]
    MismatchedAssetAmount {},

    #[error("Duplicated recipients")]
    DuplicatedRecipient {},

    #[error("Vesting amount for address {address:?} is 0")]
    ZeroVestingAmount { address: String },

    #[error("Nothing to be claimed")]
    NoClaimable {},

    #[error("Vesting no longer active")]
    VestingNotActive {},

    #[error("No tollgates required")]
    NoTollgateRequired {},

    #[error("Next tollgate time not reached")]
    NextTollgateTimeNotReached {},
}
