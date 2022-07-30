use std::error::Error;

use astroport_dca::{ExecuteMsg, QueryMsg, UserConfig};
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::Executor;

use crate::error::ContractError;

use super::common::*;

#[test]
fn empty_tips_balance() -> Result<(), Box<dyn Error>> {
    let (app, dca) = instantiate();

    let config: UserConfig = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserConfig {
            user: USER_ONE.to_string(),
        },
    )?;

    assert_eq!(config, UserConfig::default());
    assert_eq!(config.tips_balance, vec![]);

    Ok(())
}

#[test]
fn add_tips_not_empty() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    let err = app
        .execute_contract(Addr::unchecked(USER_ONE), dca, &ExecuteMsg::AddTips {}, &[])
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::InvalidZeroAmount {}
    );

    Ok(())
}

#[test]
fn only_add_tips_denom() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    let err = app
        .execute_contract(
            Addr::unchecked(USER_ONE),
            dca,
            &ExecuteMsg::AddTips {},
            &[Coin::new(1_000, OSMO)],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::InvalidTipAssetInfo {}
    );

    Ok(())
}

#[test]
fn add_tips_works() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::AddTips {},
        &[Coin::new(1_000, USDT)],
    )?;

    let config: UserConfig = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserConfig {
            user: USER_ONE.to_string(),
        },
    )?;

    assert_eq!(config.tips_balance, vec![native_asset(USDT, 1_000)]);

    Ok(())
}

#[test]
fn withdraw_tips_works() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::AddTips {},
        &[Coin::new(1_000, USDT), Coin::new(1_000, USDC)],
    )?;

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::WithdrawTips { tips: vec![] },
        &[],
    )?;

    let bal_before = app
        .wrap()
        .query_all_balances(USER_ONE)?
        .iter()
        .filter(|e| e.denom == USDT || e.denom == USDC)
        .map(|e| e.amount.u128())
        .collect::<Vec<_>>();

    assert_eq!(bal_before, vec![999_999_000, 999_999_000]);

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::WithdrawTips {
            tips: vec![native_asset(USDT, 500), native_asset(USDC, 1_000)],
        },
        &[],
    )?;

    let bal_after = app
        .wrap()
        .query_all_balances(USER_ONE)?
        .iter()
        .filter(|e| e.denom == USDT || e.denom == USDC)
        .map(|e| e.amount.u128())
        .collect::<Vec<_>>();

    assert_eq!(bal_after, vec![1_000_000_000, 999_999_500]);

    let config: UserConfig = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserConfig {
            user: USER_ONE.to_string(),
        },
    )?;

    assert_eq!(config.tips_balance, vec![native_asset(USDT, 500)]);

    Ok(())
}

#[test]
fn withdraw_tips_insuff_bal() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::AddTips {},
        &[Coin::new(1_000, USDT)],
    )?;

    let err = app
        .execute_contract(
            Addr::unchecked(USER_ONE),
            dca,
            &ExecuteMsg::WithdrawTips {
                tips: vec![native_asset(USDT, 2_000)],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::InsufficientTipWithdrawBalance {}
    );

    Ok(())
}
