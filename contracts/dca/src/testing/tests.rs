use std::str::FromStr;
use std::vec;

use astroport::asset::Asset;
use astroport::router::{ExecuteMsg as RouterExecuteMsg, SwapOperation};
use astroport_dca::dca::{
    ConfigResponse, DcaInfo, DcaQueryInfo, ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg,
    TipAssetInfo,
};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{
    attr, coin, from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, OwnedDeps,
    ReplyOn, Response, StdError, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use serde::de::DeserializeOwned;

use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::state::State;
use crate::testing::helpers::{get_orders, native, token, token_amount};

use super::custom_querier::CustomQuerier;
use super::helpers::{mock_dependencies, mock_env_at_timestamp, native_amount, query_helper};

//--------------------------------------------------------------------------------------------------
// Test setup
//--------------------------------------------------------------------------------------------------

pub(super) fn setup_test() -> OwnedDeps<MockStorage, MockApi, CustomQuerier> {
    let mut deps = mock_dependencies();

    let res = instantiate(
        deps.as_mut(),
        mock_env_at_timestamp(10000),
        mock_info("deployer", &[]),
        InstantiateMsg {
            max_hops: 2,
            whitelisted_tokens: vec![native("uluna"), token("allowed1"), token("allowed2")],
            whitelisted_tip_tokens: vec![
                TipAssetInfo {
                    info: native("uluna"),
                    per_hop_fee: Uint128::new(1),
                },
                TipAssetInfo {
                    info: token("ibc/usdc"),
                    per_hop_fee: Uint128::new(2),
                },
            ],
            max_spread: Decimal::from_str("0.01").unwrap(),
            factory_addr: "factory".to_string(),
            router_addr: "router".to_string(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 0);

    deps
}

//--------------------------------------------------------------------------------------------------
// Execution
//--------------------------------------------------------------------------------------------------

#[test]
fn proper_instantiation() {
    let deps = setup_test();
    let state = State::default();

    let res: ConfigResponse = query_helper(deps.as_ref(), QueryMsg::Config {});
    assert_eq!(
        res,
        ConfigResponse {
            max_hops: 2,
            whitelisted_tokens: vec![native("uluna"), token("allowed1"), token("allowed2")],
            whitelisted_tip_tokens: vec![
                TipAssetInfo {
                    info: native("uluna"),
                    per_hop_fee: Uint128::new(1),
                },
                TipAssetInfo {
                    info: token("ibc/usdc"),
                    per_hop_fee: Uint128::new(2),
                },
            ],
            max_spread: Decimal::from_str("0.01").unwrap(),
            factory_addr: Addr::unchecked("factory"),
            router_addr: Addr::unchecked("router"),
        }
    );

    assert_eq!(state.dca_id.load(deps.as_ref().storage).unwrap(), 0u64);
}

#[test]
fn create_dca_native() {
    let mut deps = setup_test();

    let create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[coin(1000, "uluna")]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1000),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(100),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap();

    assert_eq!(
        create_dca.attributes,
        vec![
            attr("action", "create_dca_order"),
            attr("id", "1"),
            attr("initial_asset", "1000uluna"),
            attr("target_asset", "ibc/usdt"),
            attr("interval", (60 * 60 * 24).to_string()),
            attr("dca_amount", "100"),
            attr("max_hops", "1"),
            attr("start_purchase", "1000",),
            attr("max_spread", "0"),
            attr("user", "user")
        ]
    )
}

#[test]
fn create_dca_fails() {
    let mut deps = setup_test();

    deps.querier.set_cw20_allowance("token", "user", 999);

    let create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token"),
                amount: Uint128::new(1000),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(100),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap_err();

    assert_eq!(create_dca, ContractError::InvalidTokenDeposit {});

    let create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token"),
                amount: Uint128::new(999),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(100),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap_err();

    assert_eq!(create_dca, ContractError::IndivisibleDeposit {});

    let create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token"),
                amount: Uint128::new(1001),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(10005),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap_err();

    assert_eq!(create_dca, ContractError::DepositTooSmall {});

    let create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token"),
                amount: Uint128::new(1001),
            },
            target_asset: token("token"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(10005),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap_err();

    assert_eq!(create_dca, ContractError::DuplicateAsset {});

    let create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token"),
                amount: Uint128::new(1001),
            },
            target_asset: token("token"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(10005),
            start_purchase: Some(1),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap_err();

    assert_eq!(create_dca, ContractError::StartTimeInPast {});
}

#[test]
fn create_dca_token() {
    let mut deps = setup_test();

    deps.querier.set_cw20_allowance("token", "user", 1000);

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token"),
                amount: Uint128::new(1000),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(100),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap();

    deps.querier.set_cw20_allowance("token2", "user2", 100);

    let create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user2", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token2"),
                amount: Uint128::new(100),
            },
            target_asset: token("ibc/usdc"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(10),
            start_purchase: Some(1000),
            max_hops: None,
            max_spread: None,
        },
    )
    .unwrap();

    assert_eq!(
        create_dca.attributes,
        vec![
            attr("action", "create_dca_order"),
            attr("id", "2"),
            attr("initial_asset", "100token2"),
            attr("target_asset", "ibc/usdc"),
            attr("interval", (60 * 60 * 24).to_string()),
            attr("dca_amount", "10"),
            attr("max_hops", "0"),
            attr("start_purchase", "1000",),
            attr("max_spread", "0"),
            attr("user", "user2")
        ]
    )
}

#[test]
fn cancel_dca_fails() {
    let mut deps = setup_test();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[coin(1000, "uluna")]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1000),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(100),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap();

    let cancel = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user2", &[]),
        astroport_dca::dca::ExecuteMsg::CancelDcaOrder { id: 1 },
    )
    .unwrap_err();

    assert_eq!(cancel, ContractError::Unauthorized {});

    let cancel = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CancelDcaOrder { id: 2 },
    )
    .unwrap_err();

    assert_eq!(
        cancel,
        ContractError::Std(StdError::NotFound {
            kind: "astroport_dca::dca::DcaInfo".to_string()
        })
    );
}

#[test]
fn cancel_dca() {
    let mut deps = setup_test();
    let state = State::default();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[coin(1000, "uluna")]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1000),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(100),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap();

    deps.querier.set_cw20_allowance("token", "user2", 1000);

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user2", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token"),
                amount: Uint128::new(1000),
            },
            target_asset: token("ibc/usdt"),
            interval: 60 * 60 * 24,
            dca_amount: Uint128::new(100),
            start_purchase: Some(1000),
            max_hops: Some(1),
            max_spread: None,
        },
    )
    .unwrap();

    let orders = get_orders(&state, deps.as_ref().storage);
    assert_eq!(orders.len(), 2);

    let cancel = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user", &[]),
        astroport_dca::dca::ExecuteMsg::CancelDcaOrder { id: 1 },
    )
    .unwrap();

    assert_eq!(
        cancel.attributes,
        vec![attr("action", "cancel_dca_order"), attr("id", "1"),]
    );
    assert_eq!(cancel.messages.len(), 1);
    assert_eq!(
        cancel.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: "user".to_string(),
                amount: vec![coin(1000, "uluna")]
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let cancel = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user2", &[]),
        astroport_dca::dca::ExecuteMsg::CancelDcaOrder { id: 2 },
    )
    .unwrap();

    assert_eq!(
        cancel.attributes,
        vec![attr("action", "cancel_dca_order"), attr("id", "2"),]
    );
    assert_eq!(cancel.messages.len(), 0);

    let orders = get_orders(&state, deps.as_ref().storage);
    assert_eq!(orders.len(), 0)
}

#[test]
fn cancel_dca_after_execute_cw20() {
    let mut deps = setup_test();
    let _state = State::default();

    deps.querier.set_cw20_allowance("token-a", "alice", 2);

    let max_spread = Some(Decimal::from_str("0.1").unwrap());
    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-a"),
                amount: Uint128::new(2),
            },
            target_asset: token("astro"),
            interval: 12 * 60 * 60,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(1, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1),
            },
        },
    )
    .unwrap();

    let _perform_dca_id1_1 = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("astro"),
            }],
        },
    )
    .unwrap();

    let cancel = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CancelDcaOrder { id: 1 },
    )
    .unwrap();

    assert_eq!(
        cancel.attributes,
        vec![attr("action", "cancel_dca_order"), attr("id", "1"),]
    );
    assert_eq!(cancel.messages.len(), 0);
}

#[test]
fn cancel_dca_after_execute_native() {
    let mut deps = setup_test();
    let _state = State::default();

    let max_spread = Some(Decimal::from_str("0.1").unwrap());
    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(20, "uluna")]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(20),
            },
            target_asset: token("astro"),
            interval: 12 * 60 * 60,
            dca_amount: Uint128::new(5),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(1, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1),
            },
        },
    )
    .unwrap();

    let _perform_dca_id1_1 = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: native("uluna"),
                ask_asset_info: token("astro"),
            }],
        },
    )
    .unwrap();

    let cancel = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CancelDcaOrder { id: 1 },
    )
    .unwrap();

    assert_eq!(
        cancel.attributes,
        vec![attr("action", "cancel_dca_order"), attr("id", "1"),]
    );
    assert_eq!(cancel.messages.len(), 1);
    assert_eq!(
        cancel.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: "alice".to_string(),
                amount: vec![coin(15, "uluna")]
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn update_config_fails() {
    let mut deps = setup_test();

    let msg = ExecuteMsg::UpdateConfig {
        max_hops: None,
        whitelisted_tokens: None,
        whitelisted_tip_tokens: None,
        max_spread: None,
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {})
}

#[test]
fn update_config() {
    let mut deps = setup_test();

    let msg = ExecuteMsg::UpdateConfig {
        max_hops: None,
        whitelisted_tokens: None,
        whitelisted_tip_tokens: None,
        max_spread: None,
    };
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("factory_owner", &[]),
        msg,
    )
    .unwrap();

    let res: ConfigResponse = query_helper(deps.as_ref(), QueryMsg::Config {});
    assert_eq!(
        res,
        ConfigResponse {
            max_hops: 2,
            whitelisted_tokens: vec![native("uluna"), token("allowed1"), token("allowed2")],
            whitelisted_tip_tokens: vec![
                TipAssetInfo {
                    info: native("uluna"),
                    per_hop_fee: Uint128::new(1),
                },
                TipAssetInfo {
                    info: token("ibc/usdc"),
                    per_hop_fee: Uint128::new(2),
                },
            ],
            max_spread: Decimal::from_str("0.01").unwrap(),
            factory_addr: Addr::unchecked("factory"),
            router_addr: Addr::unchecked("router"),
        }
    );

    let msg = ExecuteMsg::UpdateConfig {
        max_hops: Some(10),
        whitelisted_tokens: Some(vec![token("allowed")]),
        whitelisted_tip_tokens: Some(vec![TipAssetInfo {
            info: token("ibc/usdt"),
            per_hop_fee: Uint128::zero(),
        }]),
        max_spread: Some(Decimal::from_str("0.5").unwrap()),
    };

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("factory_owner", &[]),
        msg,
    )
    .unwrap();

    assert_eq!(
        res,
        Response::default().add_attributes(vec![attr("action", "update_config")])
    );

    let res: ConfigResponse = query_helper(deps.as_ref(), QueryMsg::Config {});
    assert_eq!(
        res,
        ConfigResponse {
            max_hops: 10,
            whitelisted_tokens: vec![token("allowed")],
            whitelisted_tip_tokens: vec![TipAssetInfo {
                info: token("ibc/usdt"),
                per_hop_fee: Uint128::zero(),
            },],
            max_spread: Decimal::from_str("0.5").unwrap(),
            factory_addr: Addr::unchecked("factory"),
            router_addr: Addr::unchecked("router"),
        }
    );
}

#[test]
fn withdraw_tips_fails() {
    let mut deps = setup_test();

    let msg = ExecuteMsg::Withdraw {
        assets: Some(vec![native_amount("uluna", 1000)]),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::NonExistentTipJar {
            token: "*".to_string()
        }
    );

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(10, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(10),
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(10, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(10),
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("ibc/usdc", &[]),
        astroport_dca::dca::ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "alice".to_string(),
            amount: Uint128::new(4),
            msg: to_binary(&ReceiveMsg::AddBotTip {}).unwrap(),
        }),
    )
    .unwrap();

    let msg = ExecuteMsg::Withdraw {
        assets: Some(vec![token_amount("token", 1000)]),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::NonExistentTipJar {
            token: "token".to_string()
        }
    );

    let msg = ExecuteMsg::Withdraw {
        assets: Some(vec![native_amount("uluna", 1000)]),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap_err();
    assert_eq!(res, ContractError::InsufficientTipBalance {});
}

#[test]
fn withdraw_tips() {
    let mut deps = setup_test();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(10, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(10),
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(10, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(10),
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("ibc/usdc", &[]),
        astroport_dca::dca::ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "alice".to_string(),
            amount: Uint128::new(4),
            msg: to_binary(&ReceiveMsg::AddBotTip {}).unwrap(),
        }),
    )
    .unwrap();

    let msg = ExecuteMsg::Withdraw {
        assets: Some(vec![native_amount("uluna", 5)]),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap();

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: "alice".to_string(),
                amount: vec![coin(5, "uluna")]
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let msg = ExecuteMsg::Withdraw { assets: None };
    let res = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap();

    assert_eq!(res.messages.len(), 2);
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: "alice".to_string(),
                amount: vec![coin(15, "uluna")]
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    assert_execute_eq::<Cw20ExecuteMsg>(res.messages[1].clone(), |c, v, m| {
        assert_eq!(c, "ibc/usdc".to_string());
        assert_eq!(v, vec![]);
        assert_eq!(
            m,
            Cw20ExecuteMsg::Transfer {
                recipient: "alice".to_string(),
                amount: Uint128::new(4)
            }
        )
    })
}

#[test]
fn perform_dca_fail() {
    let mut deps = setup_test();
    let _state = State::default();

    deps.querier.set_cw20_allowance("token-a", "alice", 2);

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-a"),
                amount: Uint128::new(2),
            },
            target_asset: token("ibc/usdt"),
            interval: 10,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: Some(2),
            max_spread: None,
        },
    )
    .unwrap();

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(perform_dca, ContractError::EmptyHopRoute {});

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![
                SwapOperation::AstroSwap {
                    offer_asset_info: token("token-a"),
                    ask_asset_info: token("ibc/usdt"),
                },
                SwapOperation::AstroSwap {
                    offer_asset_info: token("token-a"),
                    ask_asset_info: token("ibc/usdt"),
                },
                SwapOperation::AstroSwap {
                    offer_asset_info: token("token-a"),
                    ask_asset_info: token("ibc/usdt"),
                },
            ],
        },
    )
    .unwrap_err();
    assert_eq!(perform_dca, ContractError::MaxHopsAssertion { hops: 3 });

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![
                SwapOperation::AstroSwap {
                    offer_asset_info: token("token-a"),
                    ask_asset_info: token("unknown"),
                },
                SwapOperation::AstroSwap {
                    offer_asset_info: token("unknown"),
                    ask_asset_info: token("ibc/usdt"),
                },
            ],
        },
    )
    .unwrap_err();
    assert_eq!(
        perform_dca,
        ContractError::InvalidHopRoute {
            token: "unknown".to_string()
        }
    );

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("ibc/usdt"),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(perform_dca, ContractError::NoTipBalance {});

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::NativeSwap {
                offer_denom: "uusd".to_string(),
                ask_denom: "ibc/usdt".to_string(),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(perform_dca, ContractError::NativeSwapNotSupported {});

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("ibc/usdc"),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(perform_dca, ContractError::TargetAssetAssertion {});

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(1, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1),
            },
        },
    )
    .unwrap();

    let _perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("ibc/usdt"),
            }],
        },
    )
    .unwrap();

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("ibc/usdt"),
            }],
        },
    )
    .unwrap_err();

    assert_eq!(perform_dca, ContractError::PurchaseTooEarly {});

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(20),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("ibc/usdt"),
            }],
        },
    )
    .unwrap_err();

    assert_eq!(perform_dca, ContractError::InsufficientTipBalance {});

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(10, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(10),
            },
        },
    )
    .unwrap();

    let _perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(20),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("ibc/usdt"),
            }],
        },
    )
    .unwrap();

    let perform_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(20),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("ibc/usdt"),
            }],
        },
    )
    .unwrap_err();

    // when dca is finished, no dca data remaining
    assert_eq!(perform_dca, ContractError::NonExistentDca {});
}

#[test]
fn example_bounty_happy_path() {
    let mut deps = setup_test();
    let state = State::default();

    deps.querier.set_cw20_allowance("token-a", "alice", 2);
    deps.querier.set_cw20_allowance("token-b", "alice", 5);
    deps.querier.set_cw20_allowance("token-c", "alice", 10);

    let max_spread = Some(Decimal::from_str("0.1").unwrap());
    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-a"),
                amount: Uint128::new(2),
            },
            target_asset: token("astro"),
            interval: 12 * 60 * 60,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-b"),
                amount: Uint128::new(5),
            },
            target_asset: token("astro"),
            interval: 24 * 60 * 60,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-c"),
                amount: Uint128::new(10),
            },
            target_asset: native("uluna"),
            interval: 7 * 24 * 60 * 60,
            dca_amount: Uint128::new(5),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(1, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1),
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("ibc/usdc", &[]),
        astroport_dca::dca::ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "alice".to_string(),
            amount: Uint128::new(4),
            msg: to_binary(&ReceiveMsg::AddBotTip {}).unwrap(),
        }),
    )
    .unwrap();

    let orders = get_orders(&state, deps.as_ref().storage);
    assert_eq!(orders.len(), 3);

    let jars = state
        .get_tip_jars(deps.as_ref().storage, Addr::unchecked("alice"))
        .unwrap();

    // jars are now filled
    assert_eq!(
        jars,
        vec![
            Asset {
                info: native("uluna"),
                amount: Uint128::new(1)
            },
            Asset {
                info: token("ibc/usdc"),
                amount: Uint128::new(4)
            },
        ]
    );

    let perform_dca_id1_1 = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("astro"),
            }],
        },
    )
    .unwrap();

    // Assert first perform DCA (Orderid = 1) --------------------------------------------------
    assert_eq!(perform_dca_id1_1.messages.len(), 3);
    assert_execute_eq::<Cw20ExecuteMsg>(perform_dca_id1_1.messages[0].clone(), |c, f, cw| {
        assert_eq!(c, "token-a");
        assert_eq!(f, vec![]);
        assert_eq!(
            cw,
            Cw20ExecuteMsg::TransferFrom {
                owner: "alice".to_string(),
                recipient: "router".to_string(),
                amount: Uint128::new(1)
            }
        )
    });
    assert_execute_eq::<RouterExecuteMsg>(perform_dca_id1_1.messages[1].clone(), |c, f, msg| {
        assert_eq!(c, "router");
        assert_eq!(f, vec![]);
        assert_eq!(
            msg,
            RouterExecuteMsg::ExecuteSwapOperations {
                to: Some("alice".to_string()),
                max_spread: Some(Decimal::from_str("0.1").unwrap()),
                minimum_receive: None,
                operations: vec![SwapOperation::AstroSwap {
                    offer_asset_info: token("token-a"),
                    ask_asset_info: token("astro"),
                }]
            }
        )
    });
    assert_eq!(
        perform_dca_id1_1.messages[2],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: "anyone".to_string(),
                amount: vec![coin(1, "uluna")]
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let perform_dca_id2_1 = execute(
        deps.as_mut(),
        mock_env_at_timestamp(20),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 2,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-b"),
                ask_asset_info: token("astro"),
            }],
        },
    )
    .unwrap();

    // Assert first perform DCA (Orderid = 2) --------------------------------------------------
    assert_eq!(perform_dca_id2_1.messages.len(), 3);
    assert_execute_eq::<Cw20ExecuteMsg>(perform_dca_id2_1.messages[0].clone(), |c, f, cw| {
        assert_eq!(c, "token-b");
        assert_eq!(f, vec![]);
        assert_eq!(
            cw,
            Cw20ExecuteMsg::TransferFrom {
                owner: "alice".to_string(),
                recipient: "router".to_string(),
                amount: Uint128::new(1)
            }
        )
    });
    assert_execute_eq::<RouterExecuteMsg>(perform_dca_id2_1.messages[1].clone(), |c, f, msg| {
        assert_eq!(c, "router");
        assert_eq!(f, vec![]);
        assert_eq!(
            msg,
            RouterExecuteMsg::ExecuteSwapOperations {
                to: Some("alice".to_string()),
                max_spread: Some(Decimal::from_str("0.1").unwrap()),
                minimum_receive: None,
                operations: vec![SwapOperation::AstroSwap {
                    offer_asset_info: token("token-b"),
                    ask_asset_info: token("astro"),
                }]
            }
        )
    });
    // instead of luna now usdc is sent
    assert_execute_eq::<Cw20ExecuteMsg>(perform_dca_id2_1.messages[2].clone(), |c, f, msg| {
        assert_eq!(c, "ibc/usdc");
        assert_eq!(f, vec![]);
        assert_eq!(
            msg,
            Cw20ExecuteMsg::Transfer {
                recipient: "anyone".to_string(),
                amount: Uint128::new(2)
            }
        )
    });

    let perform_dca_id3_1 = execute(
        deps.as_mut(),
        mock_env_at_timestamp(20),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 3,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-c"),
                ask_asset_info: native("uluna"),
            }],
        },
    )
    .unwrap();

    // Assert first perform DCA (Orderid = 2) --------------------------------------------------
    assert_eq!(perform_dca_id3_1.messages.len(), 3);
    assert_execute_eq::<Cw20ExecuteMsg>(perform_dca_id3_1.messages[0].clone(), |c, f, cw| {
        assert_eq!(c, "token-c");
        assert_eq!(f, vec![]);
        assert_eq!(
            cw,
            Cw20ExecuteMsg::TransferFrom {
                owner: "alice".to_string(),
                recipient: "router".to_string(),
                amount: Uint128::new(5)
            }
        )
    });
    assert_execute_eq::<RouterExecuteMsg>(perform_dca_id3_1.messages[1].clone(), |c, f, msg| {
        assert_eq!(c, "router");
        assert_eq!(f, vec![]);
        assert_eq!(
            msg,
            RouterExecuteMsg::ExecuteSwapOperations {
                to: Some("alice".to_string()),
                max_spread: Some(Decimal::from_str("0.1").unwrap()),
                minimum_receive: None,
                operations: vec![SwapOperation::AstroSwap {
                    offer_asset_info: token("token-c"),
                    ask_asset_info: native("uluna"),
                }]
            }
        )
    });
    // instead of luna now usdc is sent
    assert_execute_eq::<Cw20ExecuteMsg>(perform_dca_id3_1.messages[2].clone(), |c, f, msg| {
        assert_eq!(c, "ibc/usdc");
        assert_eq!(f, vec![]);
        assert_eq!(
            msg,
            Cw20ExecuteMsg::Transfer {
                recipient: "anyone".to_string(),
                amount: Uint128::new(2)
            }
        )
    });

    // jars are now empty -----------------------------------------------------
    let jars = state
        .get_tip_jars(deps.as_ref().storage, Addr::unchecked("alice"))
        .unwrap();
    assert_eq!(jars, vec![]);

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("ibc/usdc", &[]),
        astroport_dca::dca::ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "alice".to_string(),
            amount: Uint128::new(100),
            msg: to_binary(&ReceiveMsg::AddBotTip {}).unwrap(),
        }),
    )
    .unwrap();

    let perform_dca_id1_2 = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10 + (12 * 60 * 60)),
        mock_info("anyone", &[]),
        astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
            id: 1,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: token("token-a"),
                ask_asset_info: token("astro"),
            }],
        },
    )
    .unwrap();

    // Assert second perform DCA (Orderid = 1) --------------------------------------------------

    assert_eq!(perform_dca_id1_2.messages.len(), 3);
    assert_execute_eq::<Cw20ExecuteMsg>(perform_dca_id1_2.messages[0].clone(), |c, f, cw| {
        assert_eq!(c, "token-a");
        assert_eq!(f, vec![]);
        assert_eq!(
            cw,
            Cw20ExecuteMsg::TransferFrom {
                owner: "alice".to_string(),
                recipient: "router".to_string(),
                amount: Uint128::new(1)
            }
        )
    });
    assert_execute_eq::<RouterExecuteMsg>(perform_dca_id1_2.messages[1].clone(), |c, f, msg| {
        assert_eq!(c, "router");
        assert_eq!(f, vec![]);
        assert_eq!(
            msg,
            RouterExecuteMsg::ExecuteSwapOperations {
                to: Some("alice".to_string()),
                max_spread: Some(Decimal::from_str("0.1").unwrap()),
                minimum_receive: None,
                operations: vec![SwapOperation::AstroSwap {
                    offer_asset_info: token("token-a"),
                    ask_asset_info: token("astro"),
                }]
            }
        )
    });

    // instead of luna now usdc is sent
    assert_execute_eq::<Cw20ExecuteMsg>(perform_dca_id1_2.messages[2].clone(), |c, f, msg| {
        assert_eq!(c, "ibc/usdc");
        assert_eq!(f, vec![]);
        assert_eq!(
            msg,
            Cw20ExecuteMsg::Transfer {
                recipient: "anyone".to_string(),
                amount: Uint128::new(2)
            }
        )
    });

    let orders = get_orders(&state, deps.as_ref().storage);
    // after second execution, first order is done
    let ids: Vec<u64> = orders.iter().map(|o| o.id).collect();
    assert_eq!(orders.len(), 2);
    assert_eq!(ids, vec![2, 3]);

    let res: Vec<DcaQueryInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::UserAssetDcaOrders {
            user: "alice".to_string(),
            asset: token("token-b"),
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(res.len(), 1);
}

#[test]
fn test_queries() {
    let mut deps = setup_test();
    let _state = State::default();

    deps.querier.set_cw20_allowance("token-a", "alice", 50);
    deps.querier.set_cw20_allowance("token-b", "alice", 5);
    deps.querier.set_cw20_allowance("token-c", "alice", 10);
    deps.querier.set_cw20_allowance("token-a", "user2", 2);

    let max_spread = Some(Decimal::from_str("0.1").unwrap());
    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-a"),
                amount: Uint128::new(2),
            },
            target_asset: token("astro"),
            interval: 12 * 60 * 60,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-b"),
                amount: Uint128::new(5),
            },
            target_asset: token("astro"),
            interval: 24 * 60 * 60,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-c"),
                amount: Uint128::new(10),
            },
            target_asset: native("uluna"),
            interval: 7 * 24 * 60 * 60,
            dca_amount: Uint128::new(5),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("user2", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-a"),
                amount: Uint128::new(2),
            },
            target_asset: token("astro"),
            interval: 24 * 60 * 60,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    let _create_dca = execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[]),
        astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
            initial_asset: Asset {
                info: token("token-a"),
                amount: Uint128::new(40),
            },
            target_asset: token("astro"),
            interval: 12 * 60 * 60,
            dca_amount: Uint128::new(10),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("alice", &[coin(1, "uluna")]),
        astroport_dca::dca::ExecuteMsg::AddBotTip {
            asset: Asset {
                info: native("uluna"),
                amount: Uint128::new(1),
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env_at_timestamp(10),
        mock_info("ibc/usdc", &[]),
        astroport_dca::dca::ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "alice".to_string(),
            amount: Uint128::new(4),
            msg: to_binary(&ReceiveMsg::AddBotTip {}).unwrap(),
        }),
    )
    .unwrap();

    // GET CONFIG
    let res: ConfigResponse = query_helper(deps.as_ref(), QueryMsg::Config {});
    assert_eq!(
        res,
        ConfigResponse {
            max_hops: 2,
            whitelisted_tokens: vec![native("uluna"), token("allowed1"), token("allowed2")],
            whitelisted_tip_tokens: vec![
                TipAssetInfo {
                    info: native("uluna"),
                    per_hop_fee: Uint128::new(1),
                },
                TipAssetInfo {
                    info: token("ibc/usdc"),
                    per_hop_fee: Uint128::new(2),
                },
            ],
            max_spread: Decimal::from_str("0.01").unwrap(),
            factory_addr: Addr::unchecked("factory"),
            router_addr: Addr::unchecked("router"),
        }
    );

    // DCA ORDER
    let res: DcaInfo = query_helper(deps.as_ref(), QueryMsg::DcaOrder { id: 4 });
    assert_eq!(
        res,
        DcaInfo {
            id: 4,
            initial_asset: Asset {
                info: token("token-a"),
                amount: Uint128::new(2),
            },
            target_asset: token("astro"),
            interval: 24 * 60 * 60,
            dca_amount: Uint128::new(1),
            start_purchase: None,
            max_hops: None,
            max_spread: max_spread.clone(),
            user: Addr::unchecked("user2"),
            last_purchase: 0
        }
    );

    // DCA ORDERS
    let res: Vec<DcaInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::DcaOrders {
            start_after: None,
            limit: None,
        },
    );
    let ids: Vec<u64> = res.iter().map(|a| a.id).collect();
    assert_eq!(ids, vec![1, 2, 3, 4, 5]);

    let res2: Vec<DcaInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::DcaOrders {
            start_after: Some(2),
            limit: None,
        },
    );
    let ids2: Vec<u64> = res2.iter().map(|a| a.id).collect();
    assert_eq!(ids2, vec![3, 4, 5]);

    let res3: Vec<DcaInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::DcaOrders {
            start_after: None,
            limit: Some(1),
        },
    );
    let ids3: Vec<u64> = res3.iter().map(|a| a.id).collect();
    assert_eq!(ids3, vec![1]);

    // DCA USER ASSET ORDERS
    let res: Vec<DcaQueryInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::UserAssetDcaOrders {
            user: "alice".to_string(),
            asset: token("token-a"),
            start_after: None,
            limit: None,
        },
    );
    let ids: Vec<u64> = res.iter().map(|a| a.info.id).collect();
    assert_eq!(ids, vec![1, 5]);
    let all_alice = res.iter().all(|a| a.info.user == "alice");
    assert_eq!(all_alice, true);
    let all_token = res
        .iter()
        .all(|a| a.info.initial_asset.info == token("token-a"));
    assert_eq!(all_token, true);

    let res: Vec<DcaQueryInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::UserAssetDcaOrders {
            user: "alice".to_string(),
            asset: token("token-a"),
            start_after: Some(2),
            limit: None,
        },
    );
    let ids: Vec<u64> = res.iter().map(|a| a.info.id).collect();
    assert_eq!(ids, vec![5]);
    let all_alice = res.iter().all(|a| a.info.user == "alice");
    assert_eq!(all_alice, true);
    let all_token = res
        .iter()
        .all(|a| a.info.initial_asset.info == token("token-a"));
    assert_eq!(all_token, true);

    // DCA USER ORDERS
    let res: Vec<DcaQueryInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::UserDcaOrders {
            user: "alice".to_string(),
            start_after: None,
            limit: None,
        },
    );
    let ids: Vec<u64> = res.iter().map(|a| a.info.id).collect();
    assert_eq!(ids, vec![1, 2, 3, 5]);
    let all_alice = res.iter().all(|a| a.info.user == "alice");
    assert_eq!(all_alice, true);

    let res: Vec<DcaQueryInfo> = query_helper(
        deps.as_ref(),
        QueryMsg::UserDcaOrders {
            user: "alice".to_string(),
            start_after: Some(2),
            limit: None,
        },
    );
    let ids: Vec<u64> = res.iter().map(|a| a.info.id).collect();
    assert_eq!(ids, vec![3, 5]);
    let all_alice = res.iter().all(|a| a.info.user == "alice");
    assert_eq!(all_alice, true);

    // USER TIPS ORDER
    let user_tips: Vec<Asset> = query_helper(
        deps.as_ref(),
        QueryMsg::UserTips {
            user: "alice".to_string(),
        },
    );
    assert_eq!(
        user_tips,
        vec![native_amount("uluna", 1), token_amount("ibc/usdc", 4)]
    );
}

fn assert_execute_eq<T: DeserializeOwned>(
    message: SubMsg,
    f: fn(String, Vec<Coin>, T) -> (),
) -> () {
    match message.msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds,
            msg,
        }) => {
            let sub_msg: T = from_binary(&msg).unwrap();
            f(contract_addr, funds, sub_msg);
        }

        _ => panic!("DO NOT ENTER HERE"),
    }
}
