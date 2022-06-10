use cosmwasm_std::testing::{
    mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::*;

use crate::contract::*;
use crate::msg::*;
use crate::state::Vesting;

use std::marker::PhantomData;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let contract_addr = MOCK_CONTRACT_ADDR.to_string();
    let custom_querier: MockQuerier = MockQuerier::new(&[(&contract_addr, contract_balance)]);

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub fn mock_env_time(time: u64) -> Env {
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(time);
    env
}

pub fn mock_init() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Response) {
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

    let total = 300_000_000_001u128 + 300_000_000_000u128;

    let msg = InstantiateMsg {
        master_address: Some("master_address".to_string()),
        denom: "uluna".to_string(),
        vestings,
    };

    let info = mock_info("addr0000", &[coin(total, "uluna")]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env_time(0), info, msg).unwrap();
    (deps, res)
}

pub fn mock_full_init() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Response) {
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
        Vesting {
            recipient: "recipient3".to_string(),
            amount: Uint128::from(150_000_000_001u128),
        },
        Vesting {
            recipient: "recipient4".to_string(),
            amount: Uint128::from(150_000_000_000u128),
        },
        Vesting {
            recipient: "recipient5".to_string(),
            amount: Uint128::from(75_000_000_001u128),
        },
        Vesting {
            recipient: "recipient6".to_string(),
            amount: Uint128::from(75_000_000_000u128),
        },
        Vesting {
            recipient: "recipient7".to_string(),
            amount: Uint128::from(1u128),
        },
    ];

    let total = 300_000_000_001u128
        + 300_000_000_000u128
        + 150_000_000_001u128
        + 150_000_000_000u128
        + 75_000_000_001u128
        + 75_000_000_000u128
        + 1u128;

    let msg = InstantiateMsg {
        master_address: Some("master_address".to_string()),
        denom: "uluna".to_string(),
        vestings,
    };

    let info = mock_info("addr0000", &[coin(total, "uluna")]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env_time(0), info, msg).unwrap();
    (deps, res)
}
