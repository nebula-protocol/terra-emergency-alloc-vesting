use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    read_config, read_vesting_info, store_config, store_vesting_info, Config, VestingInfo,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-emergency-vesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let master_address = if msg.master_address.is_none() {
        info.sender.to_string()
    } else {
        msg.master_address.unwrap()
    };

    // check sent vesting asset denom
    let mut sent_amount = Uint128::zero();
    for coin in info.funds.iter() {
        if coin.denom != msg.denom.to_string() {
            return Err(ContractError::MismatchedAssetType {});
        } else {
            sent_amount = coin.amount;
        }
    }
    // check sent vesting asset denom
    let sum_vesting_amount: Uint128 = msg
        .vestings
        .iter()
        .fold(Uint128::zero(), |sum, recipient| sum + recipient.amount);
    if sent_amount != sum_vesting_amount {
        return Err(ContractError::MismatchedAssetAmount {});
    }

    // store each recipient's vesting info
    for vesting in msg.vestings {
        let vesting_periods = if vesting.amount > Uint128::new(300_000_000_000u128) {
            12u64
        } else if vesting.amount > Uint128::new(150_000_000_000) {
            9u64
        } else if vesting.amount > Uint128::new(75_000_000_000) {
            6u64
        } else {
            3u64
        };

        let vesting_seconds = vesting_periods * msg.seconds_per_period;
        let tollgates_required = if vesting_periods == 12u64 {
            3u64
        } else {
            vesting_periods / 3u64
        };
        let vesting_info = VestingInfo {
            recipient: deps.api.addr_validate(&vesting.recipient)?,
            vesting_duration: vesting_seconds,
            total_amount: vesting.amount,
            claimed_amount: Uint128::zero(),
            vested_amount: vesting.amount,
            last_claimed: env.block.time.seconds(),
            active: true,
            tollgates_required,
            tollgates_approved: 0u64,
        };
        store_vesting_info(
            deps.storage,
            &deps.api.addr_validate(&vesting.recipient)?,
            &vesting_info,
        )?;
    }

    store_config(
        deps.storage,
        &Config {
            master_address: deps.api.addr_validate(&master_address)?,
            community_pool_address: deps.api.addr_validate(&msg.community_pool_address)?,
            denom: msg.denom,
            vesting_start_time: env.block.time.seconds(),
            seconds_per_period: msg.seconds_per_period,
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("master_address", master_address)
        .add_attribute("community_pool_address", msg.community_pool_address)
        .add_attribute("vesting_start_time", env.block.time.seconds().to_string()))
}

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

pub fn try_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let seconds_per_period = config.seconds_per_period;
    let mut vesting_info = read_vesting_info(deps.storage, &info.sender)?;

    // calcualte claimable amount
    let periods_since_genesis =
        (env.block.time.seconds() - config.vesting_start_time) / seconds_per_period;
    let approved_periods = (vesting_info.tollgates_approved + 1) * 3;
    let eligible_periods = std::cmp::min(periods_since_genesis, approved_periods);

    let amount_per_month = vesting_info.total_amount
        / Uint128::new((vesting_info.vesting_duration / seconds_per_period) as u128);
    let claimable_periods = (config.vesting_start_time + eligible_periods * seconds_per_period
        - vesting_info.last_claimed)
        / seconds_per_period;
    let claimable_amount = amount_per_month * Uint128::new(claimable_periods as u128);

    // update recipient's vesting info
    vesting_info.claimed_amount += claimable_amount;
    vesting_info.vested_amount -= claimable_amount;
    vesting_info.last_claimed = env.block.time.seconds();

    Ok(
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: vesting_info.recipient.to_string(),
            amount: coins(claimable_amount.into(), config.denom.to_string()),
        }))),
    )
}

pub fn try_approve_tollgate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    approve: bool,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let seconds_per_period = config.seconds_per_period;

    // msg only callable by master_address
    if info.sender != config.master_address {
        return Err(ContractError::Unauthorized {});
    }

    let validated_recipient = deps.api.addr_validate(&recipient)?;
    let mut vesting_info = read_vesting_info(deps.storage, &validated_recipient)?;

    // revert if vesting for recipient is no longer active (last tollgate not approved)
    if !vesting_info.active {
        return Err(ContractError::VestingNotActive {});
    }

    let periods_elapsed =
        (env.block.time.seconds() - config.vesting_start_time) / seconds_per_period;
    let total_periods = vesting_info.vesting_duration / seconds_per_period;
    let periods_left = total_periods - periods_elapsed;
    let quarters_left = periods_left / 3;

    if vesting_info.tollgates_required == 0 {
        return Err(ContractError::NoTollgateRequired {});
    } else if vesting_info.tollgates_required - vesting_info.tollgates_approved <= quarters_left {
        return Err(ContractError::NextTollgateTimeNotReached {});
    }

    if approve {
        vesting_info.tollgates_approved += 1;
    } else {
        vesting_info.active = false;
        vesting_info.vested_amount = Uint128::new(0u128);

        // TODO: send remaining amount back to master_address
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VestingInfo { recipient } => to_binary(&query_vesting_info(deps, recipient)?),
    }
}

fn query_vesting_info(deps: Deps, recipient: String) -> StdResult<VestingInfo> {
    let vesting_info = read_vesting_info(deps.storage, &deps.api.addr_validate(&recipient)?)?;
    Ok(vesting_info)
}
