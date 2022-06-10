use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, VestingInfo, CONFIG, VESTING_INFO};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "crates.io:terra-emergency-vesting";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Amount of seconds in each period.
pub const SECONDS_PER_PERIOD: u64 = 60u64 * 60u64 * 24u64 * 30u64;

// Number of periods in each Tollgate.
pub const PERIODS_PER_TOLL: u64 = 3;

/// ## Description
/// Creates a new contract with the specified parameters packed in the `msg` variable.
/// Returns a [`Response`] with the specified attributes if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **_env** is an object of type [`Env`].
///
/// - **_info** is an object of type [`MessageInfo`].
///
/// - **msg**  is a message of type [`InstantiateMsg`] which contains the parameters used for creating the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Set `master_address` as specified; otherwise, the instantiator
    let master_address = if msg.master_address.is_none() {
        info.sender.to_string()
    } else {
        msg.master_address.unwrap()
    };

    // Check sent vesting asset denom
    if info.funds.len() != 1 || info.funds[0].denom != msg.denom {
        return Err(ContractError::MismatchedAssetType {});
    }
    let sent_amount = info.funds[0].amount;

    // Check sent vesting asset amount
    let sum_vesting_amount: Uint128 = msg
        .vestings
        .iter()
        .fold(Uint128::zero(), |sum, recipient| sum + recipient.amount);
    if sent_amount != sum_vesting_amount {
        return Err(ContractError::MismatchedAssetAmount {});
    }

    // Store each recipient's vesting info
    for vesting in msg.vestings {
        match VESTING_INFO.may_load(deps.storage, &deps.api.addr_validate(&vesting.recipient)?) {
            Ok(None) => (),
            Ok(Some(_)) => return Err(ContractError::DuplicatedRecipient {}),
            Err(e) => return Err(ContractError::Std(e)),
        }

        // Get each recipient's total vesting periods based on the vesting amount
        let total_periods = if vesting.amount > Uint128::new(300_000_000_000u128) {
            12u64
        } else if vesting.amount > Uint128::new(150_000_000_000) {
            9u64
        } else if vesting.amount > Uint128::new(75_000_000_000) {
            6u64
        } else {
            3u64
        };

        let vesting_info = VestingInfo {
            recipient: deps.api.addr_validate(&vesting.recipient)?,
            active: true,
            approved_periods: PERIODS_PER_TOLL, // all vestings start with one approved tollgate
            total_periods,
            last_claimed_period: 0u64,
            total_amount: vesting.amount,
            claimed_amount: Uint128::zero(),
            vested_amount: vesting.amount,
            amount_per_period: vesting.amount / Uint128::from(total_periods),
        };

        VESTING_INFO.save(
            deps.storage,
            &deps.api.addr_validate(&vesting.recipient)?,
            &vesting_info,
        )?;
    }

    CONFIG.save(
        deps.storage,
        &Config {
            master_address: deps.api.addr_validate(&master_address)?,
            denom: msg.denom,
            vesting_start_time: env.block.time.seconds(),
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("master_address", master_address)
        .add_attribute("vesting_start_time", env.block.time.seconds().to_string()))
}

/// ## Description
/// Exposes all the execute functions available in the contract.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **env** is an object of type [`Env`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **msg** is an object of type [`ExecuteMsg`].
///
/// ## Commands
/// - **ExecuteMsg::ApproveTollgate {
///             recipient,
///             approve,
///         }** Updates the tollgate / approve status of a recipient's vesting status.
///
/// - **ExecuteMsg::Claim {}** Claims any eligible vesting amount.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ApproveTollgate { recipient, approve } => {
            try_approve_tollgate(deps, env, info, recipient, approve)
        }
        ExecuteMsg::Claim {} => try_claim(deps, env, info),
    }
}

/// ## Description
/// Updates the tollgate / approve status of a recipient's vesting status.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **env** is an object of type [`Env`].
///
/// - **info** is an object of type [`MessageInfo`].
pub fn try_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut vesting_info = VESTING_INFO.load(deps.storage, &info.sender)?;

    // Compute the number of periods has passed since genesis
    let periods_since_genesis =
        (env.block.time.seconds() - config.vesting_start_time) / SECONDS_PER_PERIOD;
    // Calculate the total eligible periods -- including claimed and unclaimed periods
    let eligible_periods = std::cmp::min(periods_since_genesis, vesting_info.approved_periods);
    // Get only unclaimed periods
    let claimable_periods = eligible_periods - vesting_info.last_claimed_period;
    // Compute claimable amounts according to the unclaimed periods
    let claimable_amount = vesting_info.amount_per_period * Uint128::from(claimable_periods);
    if claimable_amount == Uint128::zero() {
        return Err(ContractError::NoClaimable {});
    }

    // Update recipient's vesting info
    vesting_info.claimed_amount += claimable_amount;
    vesting_info.vested_amount -= claimable_amount;
    vesting_info.last_claimed_period = eligible_periods;

    VESTING_INFO.save(deps.storage, &info.sender, &vesting_info)?;
    Ok(Response::new()
        .add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: vesting_info.recipient.to_string(),
            amount: coins(claimable_amount.into(), config.denom),
        })))
        .add_attribute("method", "try_claim")
        .add_attribute("recipient", info.sender)
        .add_attribute("claimed_amount", claimable_amount)
        .add_attribute("claimed_periods", eligible_periods.to_string()))
}

/// ## Description
/// Claims any eligible vesting amount.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **env** is an object of type [`Env`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **recipient** is an object of type [`String`] which the address of a protocol's recipient address.
///
/// - **approve** is an object of type [`bool`] which is the new vesting status.
pub fn try_approve_tollgate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    approve: bool,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // Can only be called by master_address
    if info.sender != config.master_address {
        return Err(ContractError::Unauthorized {});
    }
    // Validate address and load its vesting information
    let validated_recipient = deps.api.addr_validate(&recipient)?;
    let mut vesting_info = VESTING_INFO.load(deps.storage, &validated_recipient)?;

    // Revert if vesting for recipient is no longer active (last tollgate not approved)
    if !vesting_info.active {
        return Err(ContractError::VestingNotActive {});
    }

    // Compute how many periods have passed
    let periods_elapsed =
        (env.block.time.seconds() - config.vesting_start_time) / SECONDS_PER_PERIOD;

    // Check if the additional periods do not exceed the vesting total periods
    // and the tollgate is less than the current time.
    if vesting_info.approved_periods + PERIODS_PER_TOLL > vesting_info.total_periods {
        return Err(ContractError::NoTollgateRequired {});
    } else if vesting_info.approved_periods > periods_elapsed {
        return Err(ContractError::NextTollgateTimeNotReached {});
    }

    let mut msgs: Vec<SubMsg> = vec![];
    // Increase the tollgate if the new approve status is true
    // Otherwise, set the vesting to be inactive
    if approve {
        vesting_info.approved_periods += PERIODS_PER_TOLL;
    } else {
        vesting_info.active = false;
        let claimable_amount = vesting_info.amount_per_period
            * Uint128::from(vesting_info.approved_periods - vesting_info.last_claimed_period);
        msgs.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.master_address.to_string(),
            amount: coins(
                (vesting_info.vested_amount - claimable_amount).into(),
                config.denom,
            ),
        })));
        vesting_info.vested_amount = claimable_amount;
    }
    VESTING_INFO.save(deps.storage, &validated_recipient, &vesting_info)?;
    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("method", "try_approve_tollgate")
        .add_attribute("recipient", info.sender)
        .add_attribute("vesting_status", vesting_info.active.to_string())
        .add_attribute(
            "approved_periods",
            vesting_info.approved_periods.to_string(),
        ))
}

/// ## Description
/// Exposes all the queries available in the contract.
///
/// ## Params
/// - **deps** is an object of type [`Deps`].
///
/// - **_env** is an object of type [`Env`].
///
/// - **msg** is an object of type [`QueryMsg`].
///
/// ## Commands
/// - **QueryMsg::VestingInfo {
///                 recipient,
///             }** Returns the vesting information of the specified recipient.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VestingInfo { recipient } => to_binary(&query_vesting_info(deps, recipient)?),
    }
}

/// ## Description
/// Returns the vesting information of the specified recipient.
///
/// ## Params
/// - **deps** is an object of type [`Deps`].
///
/// - **recipient** is an object of type [`String`] which is the address used to query vesting information.
fn query_vesting_info(deps: Deps, recipient: String) -> StdResult<VestingInfo> {
    let vesting_info = VESTING_INFO.load(deps.storage, &deps.api.addr_validate(&recipient)?)?;
    Ok(vesting_info)
}

/// ## Description
/// Exposes the migrate functionality in the contract.
///
/// ## Params
/// - **_deps** is an object of type [`DepsMut`].
///
/// - **_env** is an object of type [`Env`].
///
/// - **_msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
