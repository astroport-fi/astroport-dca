use std::error::Error;

use astroport::{asset::PairInfo, router::SwapOperation};
use astroport_dca::{DcaInfo, ExecuteMsg, QueryMsg, UserConfig, UserDcaInfo};
use cosmwasm_std::{Addr, Coin, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_multi_test::{App, Executor};

use crate::state::Config;

use super::common::*;

fn instantiate_cw20() -> (App, Addr, Addr) {
    let (mut app, core) = instantiate();
    let token = new_cw20(&mut app, USER_ONE);

    let Config { factory_addr, .. } = app
        .wrap()
        .query_wasm_smart(&core, &QueryMsg::Config {})
        .unwrap();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        factory_addr.clone(),
        &astroport::factory::ExecuteMsg::CreatePair {
            pair_type: astroport::factory::PairType::Xyk {},
            asset_infos: [token_info(&token), native_info(USDT)],
            init_params: None,
        },
        &[],
    )
    .unwrap();

    let PairInfo { contract_addr, .. } = app
        .wrap()
        .query_wasm_smart(
            &factory_addr,
            &astroport::factory::QueryMsg::Pair {
                asset_infos: [token_info(&token), native_info(USDT)],
            },
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        token.clone(),
        &Cw20ExecuteMsg::IncreaseAllowance {
            spender: contract_addr.to_string(),
            amount: Uint128::new(1_000_000_000),
            expires: None,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        contract_addr,
        &astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: [
                token_asset(&token, 1_000_000_000),
                native_asset(USDT, 1_000_000_000),
            ],
            slippage_tolerance: None,
            auto_stake: None,
            receiver: None,
        },
        &[Coin::new(1_000_000_000, USDT)],
    )
    .unwrap();

    (app, core, token)
}

#[test]
fn purchase_cw20_works() -> Result<(), Box<dyn Error>> {
    let (mut app, dca, token) = instantiate_cw20();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::AddTips {},
        &[Coin::new(2_000_000, USDC)],
    )?;

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 1_000_000),
            target_asset: token_info(&token),
            interval: 600,
            dca_amount: Uint128::new(1_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(1_000_000, USDC)],
    )?;

    proceed(&mut app, 100);

    let BalanceResponse {
        balance: balance_before,
    } = app.wrap().query_wasm_smart(
        &token,
        &Cw20QueryMsg::Balance {
            address: USER_ONE.to_string(),
        },
    )?;

    app.execute_contract(
        Addr::unchecked(USER_TWO),
        dca.clone(),
        &ExecuteMsg::PerformDcaPurchase {
            id: 0,
            hops: vec![
                SwapOperation::AstroSwap {
                    offer_asset_info: native_info(USDC),
                    ask_asset_info: native_info(USDT),
                },
                SwapOperation::AstroSwap {
                    offer_asset_info: native_info(USDT),
                    ask_asset_info: token_info(&token),
                },
            ],
        },
        &[],
    )?;

    let BalanceResponse {
        balance: balance_after,
    } = app.wrap().query_wasm_smart(
        &token,
        &Cw20QueryMsg::Balance {
            address: USER_ONE.to_string(),
        },
    )?;
    assert_eq!((balance_after - balance_before).u128(), 995418);

    let UserConfig { tips_balance, .. } = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserConfig {
            user: USER_ONE.to_owned(),
        },
    )?;
    assert_eq!(tips_balance, vec![]);

    let orders: Vec<UserDcaInfo> = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserDcaOrders {
            user: USER_ONE.to_owned(),
        },
    )?;
    let UserDcaInfo {
        info:
            DcaInfo {
                ref initial_asset,
                last_purchase,
                ..
            },
        ..
    } = orders[0];
    assert_eq!(initial_asset, &native_asset(USDC, 0));
    assert_eq!(last_purchase, 601);

    Ok(())
}
