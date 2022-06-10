use crate::contract::*;
use crate::error::ContractError;
use crate::msg::*;
use crate::state::{Vesting, VestingInfo};
use crate::testing::mock_env::{mock_dependencies, mock_env_time, mock_init};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::*;

fn query_vesting(deps: Deps, recipient: String) -> VestingInfo {
    let msg = QueryMsg::VestingInfo {
        recipient: recipient.to_string(),
    };
    let res = query(deps, mock_env(), msg).unwrap();
    let decoded_res: VestingInfo = from_binary(&res).unwrap();
    decoded_res
}

#[test]
fn proper_initialization() {
    let (deps, _res) = mock_init();
    // TODO: test res

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 3u64,
            total_periods: 12u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(300000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient2".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient2"),
            active: true,
            approved_periods: 3u64,
            total_periods: 9u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(300000000000u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(300000000000u128),
            amount_per_period: Uint128::from(33333333333u128),
        }
    );

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient3".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient3"),
            active: true,
            approved_periods: 3u64,
            total_periods: 9u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(150000000001u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(150000000001u128),
            amount_per_period: Uint128::from(16666666666u128),
        }
    );

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient4".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient4"),
            active: true,
            approved_periods: 3u64,
            total_periods: 6u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(150000000000u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(150000000000u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient5".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient5"),
            active: true,
            approved_periods: 3u64,
            total_periods: 6u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(75000000001u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(75000000001u128),
            amount_per_period: Uint128::from(12500000000u128),
        }
    );

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient6".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient6"),
            active: true,
            approved_periods: 3u64,
            total_periods: 3u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(75000000000u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(75000000000u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient7".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient7"),
            active: true,
            approved_periods: 3u64,
            total_periods: 3u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(1u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(1u128),
            amount_per_period: Uint128::from(0u128),
        }
    );
}

#[test]
pub fn test_fail_init() {
    let mut deps = mock_dependencies(&[]);

    let vestings = vec![
        Vesting {
            recipient: "recipient1".to_string(),
            amount: Uint128::from(300_000_000_001u128),
        },
        Vesting {
            recipient: "recipient2".to_string(),
            amount: Uint128::from(300_000_000_000u128),
        },
    ];

    let msg = InstantiateMsg {
        master_address: Some("master_address".to_string()),
        community_pool_address: "community_pool_address".to_string(),
        denom: "uluna".to_string(),
        vestings,
    };

    let info = mock_info("addr0000", &[coin(1u128, "uluna")]);
    let res = instantiate(deps.as_mut(), mock_env_time(0), info, msg.clone()).unwrap_err();
    assert_eq!(res, ContractError::MismatchedAssetAmount {});

    let info = mock_info("addr0000", &[coin(600_000_000_001u128, "uasset")]);
    let res = instantiate(deps.as_mut(), mock_env_time(0), info, msg).unwrap_err();
    assert_eq!(res, ContractError::MismatchedAssetType {});

    let vestings = vec![
        Vesting {
            recipient: "recipient1".to_string(),
            amount: Uint128::from(300_000_000_001u128),
        },
        Vesting {
            recipient: "recipient1".to_string(),
            amount: Uint128::from(300_000_000_000u128),
        },
    ];

    let msg = InstantiateMsg {
        master_address: Some("master_address".to_string()),
        community_pool_address: "community_pool_address".to_string(),
        denom: "uluna".to_string(),
        vestings,
    };

    let info = mock_info("addr0000", &[coin(600_000_000_001u128, "uluna")]);
    let res = instantiate(deps.as_mut(), mock_env_time(0), info, msg).unwrap_err();
    assert_eq!(res, ContractError::DuplicatedRecipient {});
}

#[test]
fn test_approve_tollgate_and_claim() {
    let (mut deps, _) = mock_init();

    let env = mock_env_time(1);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::NextTollgateTimeNotReached {});

    let env = mock_env_time(SECONDS_PER_PERIOD * 3);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 6u64,
            total_periods: 12u64,
            last_claimed_period: 0u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::zero(),
            vested_amount: Uint128::from(300000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 3 + 10);
    let info = mock_info("recipient1", &[]);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient1".to_string(),
            amount: coins(75000000000u128, "uluna"),
        }))),
    );

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 6u64,
            total_periods: 12u64,
            last_claimed_period: 3u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(75000000000u128),
            vested_amount: Uint128::from(225000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 12);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 9u64,
            total_periods: 12u64,
            last_claimed_period: 3u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(75000000000u128),
            vested_amount: Uint128::from(225000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 12);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 12u64,
            total_periods: 12u64,
            last_claimed_period: 3u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(75000000000u128),
            vested_amount: Uint128::from(225000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 12);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::NoTollgateRequired {});

    let env = mock_env_time(SECONDS_PER_PERIOD * 12);
    let info = mock_info("recipient1", &[]);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient1".to_string(),
            amount: coins(225000000000u128, "uluna"),
        }))),
    );
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 12u64,
            total_periods: 12u64,
            last_claimed_period: 12u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(300000000000u128),
            vested_amount: Uint128::from(1u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );
}

#[test]
fn test_disapprove_tollgate_and_claim() {
    let (mut deps, _) = mock_init();

    let env = mock_env_time(SECONDS_PER_PERIOD * 3);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let env = mock_env_time(SECONDS_PER_PERIOD * 3 + 1);
    let info = mock_info("recipient1", &[]);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient1".to_string(),
            amount: coins(75000000000u128, "uluna"),
        }))),
    );
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 6u64,
            total_periods: 12u64,
            last_claimed_period: 3u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(75000000000u128),
            vested_amount: Uint128::from(225000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 6);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    let env = mock_env_time(SECONDS_PER_PERIOD * 9);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: false,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "community_pool_address".to_string(),
            amount: coins(75000000001, "uluna"),
        }))),
    );
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: false,
            approved_periods: 9u64,
            total_periods: 12u64,
            last_claimed_period: 3u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(75000000000u128),
            vested_amount: Uint128::from(150000000000u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 12);
    let info = mock_info("recipient1", &[]);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient1".to_string(),
            amount: coins(150000000000u128, "uluna"),
        }))),
    );
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: false,
            approved_periods: 9u64,
            total_periods: 12u64,
            last_claimed_period: 9u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(225000000000u128),
            vested_amount: Uint128::zero(),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );
}

#[test]
fn test_first_period_claim() {
    let (mut deps, _) = mock_init();

    let env = mock_env_time(SECONDS_PER_PERIOD * 2);
    let info = mock_info("recipient1", &[]);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient1".to_string(),
            amount: coins(50000000000u128, "uluna"),
        }))),
    );
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 3u64,
            total_periods: 12u64,
            last_claimed_period: 2u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(50000000000u128),
            vested_amount: Uint128::from(250000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 3);
    let info = mock_info("recipient1", &[]);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient1".to_string(),
            amount: coins(25000000000u128, "uluna"),
        }))),
    );
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 3u64,
            total_periods: 12u64,
            last_claimed_period: 3u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(75000000000u128),
            vested_amount: Uint128::from(225000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );
}

#[test]
fn test_spam_approve() {
    let (mut deps, _) = mock_init();

    let env = mock_env_time(SECONDS_PER_PERIOD * 100);
    let info = mock_info("master_address", &[]);
    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: false,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::ApproveTollgate {
        recipient: "recipient1".to_string(),
        approve: true,
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

    assert_eq!(res, ContractError::VestingNotActive {})
}

#[test]
fn test_no_claimable() {
    let (mut deps, _) = mock_init();

    let env = mock_env_time(SECONDS_PER_PERIOD * 2);
    let info = mock_info("recipient1", &[]);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient1".to_string(),
            amount: coins(50000000000u128, "uluna"),
        }))),
    );
    assert_eq!(
        query_vesting(deps.as_ref(), "recipient1".to_string()),
        VestingInfo {
            recipient: Addr::unchecked("recipient1"),
            active: true,
            approved_periods: 3u64,
            total_periods: 12u64,
            last_claimed_period: 2u64,
            total_amount: Uint128::from(300000000001u128),
            claimed_amount: Uint128::from(50000000000u128),
            vested_amount: Uint128::from(250000000001u128),
            amount_per_period: Uint128::from(25000000000u128),
        }
    );

    let env = mock_env_time(SECONDS_PER_PERIOD * 2 + 5);
    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::NoClaimable {});
}
