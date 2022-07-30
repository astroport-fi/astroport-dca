use std::error::Error;

use astroport::router::SwapOperation;
use astroport_dca::ExecuteMsg;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::Executor;

use crate::{
    error::ContractError,
    tests::common::{OSMO, USDT},
};

use super::common::{
    instantiate, native_asset, native_info, proceed, LUNA, USDC, USER_ONE, USER_TWO,
};

#[test]
fn purchase_not_too_early() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca,
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![SwapOperation::AstroSwap {
                    offer_asset_info: native_info(USDC),
                    ask_asset_info: native_info(LUNA),
                }],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::PurchaseTooEarly {}
    );

    Ok(())
}

#[test]
fn purchase_hops_not_empty() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca,
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::EmptyHopRoute {}
    );

    Ok(())
}

#[test]
fn purchase_not_exceed_max_hops() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca,
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(USDC),
                        ask_asset_info: native_info(LUNA),
                    },
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(LUNA),
                        ask_asset_info: native_info(USDT),
                    },
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(USDT),
                        ask_asset_info: native_info(USDC),
                    },
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(USDC),
                        ask_asset_info: native_info(LUNA),
                    },
                ],
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast::<ContractError>()?,
        ContractError::MaxHopsAssertion { .. }
    ));

    Ok(())
}

#[test]
fn purchase_correct_target_info() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca,
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![SwapOperation::AstroSwap {
                    offer_asset_info: native_info(OSMO),
                    ask_asset_info: native_info(LUNA),
                }],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::InitialAssetAssertion {}
    );

    Ok(())
}

#[test]
fn purchase_correct_initial_info() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca.clone(),
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![SwapOperation::AstroSwap {
                    offer_asset_info: native_info(USDC),
                    ask_asset_info: native_info(OSMO),
                }],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::TargetAssetAssertion {}
    );

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca,
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(USDC),
                        ask_asset_info: native_info(USDT),
                    },
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(USDT),
                        ask_asset_info: native_info(OSMO),
                    },
                ],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::TargetAssetAssertion {}
    );

    Ok(())
}

#[test]
fn purchase_whitelisted_hop_route() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca,
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(USDC),
                        ask_asset_info: native_info(OSMO),
                    },
                    SwapOperation::AstroSwap {
                        offer_asset_info: native_info(OSMO),
                        ask_asset_info: native_info(LUNA),
                    },
                ],
            },
            &[],
        )
        .unwrap_err();

    assert!(matches!(
        err.downcast::<ContractError>()?,
        ContractError::InvalidHopRoute { .. }
    ));

    Ok(())
}

#[test]
fn purchase_insuf_bal() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::AddTips {},
        &[Coin::new(10_000_000, USDC)],
    )?;

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 1_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(1_000_000),
            start_at: None,
        },
        &[Coin::new(1_000_000, USDC)],
    )?;

    proceed(&mut app, 100);

    app.execute_contract(
        Addr::unchecked(USER_TWO),
        dca.clone(),
        &ExecuteMsg::PerformDcaPurchase {
            id: 0,
            hops: vec![SwapOperation::AstroSwap {
                offer_asset_info: native_info(USDC),
                ask_asset_info: native_info(LUNA),
            }],
        },
        &[],
    )?;

    proceed(&mut app, 100);

    let err = app
        .execute_contract(
            Addr::unchecked(USER_TWO),
            dca,
            &ExecuteMsg::PerformDcaPurchase {
                id: 0,
                hops: vec![SwapOperation::AstroSwap {
                    offer_asset_info: native_info(USDC),
                    ask_asset_info: native_info(LUNA),
                }],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::InsufficientBalance {}
    );

    Ok(())
}
