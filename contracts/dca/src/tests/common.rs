#![allow(dead_code)]

use std::{error::Error, str::FromStr};

use astroport::{
    asset::{Asset, AssetInfo},
    factory::{PairConfig, PairType},
};
use astroport_dca::QueryMsg;
use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Uint128};
use cw20::Cw20Coin;
use cw_multi_test::{App, AppBuilder, ContractWrapper, Executor};

use crate::state::Config;

pub const ADMIN: &str = "admin";
pub const FEE: &str = "fee";
pub const USER_ONE: &str = "userone";
pub const LUNA: &str = "uluna";
pub const USDC: &str = "uusdc";
pub const USDT: &str = "uusdt";
pub const OSMO: &str = "uosmo";

pub const CW20_CODE: u64 = 1;

pub fn proceed(app: &mut App, block: u64) {
    app.update_block(|b| {
        b.height += block;
        b.time = b.time.plus_seconds(6 * block);
    })
}

pub fn new_cw20(app: &mut App, owner: &str) -> Addr {
    app.instantiate_contract(
        CW20_CODE,
        Addr::unchecked(owner),
        &cw20_base::msg::InstantiateMsg {
            name: "testcw".to_string(),
            symbol: "tcw".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount: Uint128::new(1_000_000_000),
            }],
            mint: None,
            marketing: None,
        },
        &[],
        "cw20",
        None,
    )
    .unwrap()
}

pub fn native_info(denom: impl Into<String>) -> AssetInfo {
    AssetInfo::NativeToken {
        denom: denom.into(),
    }
}

pub fn native_asset(denom: impl Into<String>, amount: u128) -> Asset {
    Asset {
        info: AssetInfo::NativeToken {
            denom: denom.into(),
        },
        amount: Uint128::new(amount),
    }
}

pub fn instantiate() -> (App, Addr) {
    let mut app = AppBuilder::new().build(|r, _, storage| {
        r.bank
            .init_balance(
                storage,
                &Addr::unchecked(ADMIN),
                vec![
                    Coin::new(1_500_000_001, LUNA),
                    Coin::new(3_000_000_001, USDC),
                    Coin::new(2_000_000_001, USDT),
                    Coin::new(1_000_000_001, OSMO),
                ],
            )
            .unwrap();

        r.bank
            .init_balance(
                storage,
                &Addr::unchecked(USER_ONE),
                vec![
                    Coin::new(1_000_000_000, LUNA),
                    Coin::new(1_000_000_000, USDC),
                    Coin::new(1_000_000_000, USDT),
                    Coin::new(1_000_000_000, OSMO),
                ],
            )
            .unwrap();
    });

    let cw20_code = app.store_code(Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    )));

    let cw1_code = app.store_code(Box::new(ContractWrapper::new(
        cw1_whitelist::contract::execute,
        cw1_whitelist::contract::instantiate,
        cw1_whitelist::contract::query,
    )));

    let xyk_pair_code = app.store_code(Box::new(
        ContractWrapper::new(
            astroport_pair::contract::execute,
            astroport_pair::contract::instantiate,
            astroport_pair::contract::query,
        )
        .with_reply(astroport_pair::contract::reply),
    ));

    let stable_pair_code = app.store_code(Box::new(
        ContractWrapper::new(
            astroport_pair_stable::contract::execute,
            astroport_pair_stable::contract::instantiate,
            astroport_pair_stable::contract::query,
        )
        .with_reply(astroport_pair_stable::contract::reply),
    ));

    let factory_code = app.store_code(Box::new(
        ContractWrapper::new(
            astroport_factory::contract::execute,
            astroport_factory::contract::instantiate,
            astroport_factory::contract::query,
        )
        .with_reply(astroport_factory::contract::reply),
    ));

    let router_code = app.store_code(Box::new(ContractWrapper::new(
        astroport_router::contract::execute,
        astroport_router::contract::instantiate,
        astroport_router::contract::query,
    )));

    let dca_code = app.store_code(Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )));

    let factory = app
        .instantiate_contract(
            factory_code,
            Addr::unchecked(ADMIN),
            &astroport::factory::InstantiateMsg {
                pair_configs: vec![
                    PairConfig {
                        code_id: xyk_pair_code,
                        pair_type: PairType::Xyk {},
                        total_fee_bps: 30,
                        maker_fee_bps: 3333,
                        is_disabled: false,
                        is_generator_disabled: true,
                    },
                    PairConfig {
                        code_id: stable_pair_code,
                        pair_type: PairType::Stable {},
                        total_fee_bps: 5,
                        maker_fee_bps: 5000,
                        is_disabled: false,
                        is_generator_disabled: true,
                    },
                ],
                token_code_id: cw20_code,
                fee_address: Some(FEE.to_string()),
                generator_address: None,
                owner: ADMIN.to_string(),
                whitelist_code_id: cw1_code,
            },
            &[
                Coin::new(1, LUNA),
                Coin::new(1, USDC),
                Coin::new(1, USDT),
                Coin::new(1, OSMO),
            ],
            "factory",
            None,
        )
        .unwrap();

    let luna_usdc_pair_addr = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            factory.clone(),
            &astroport::factory::ExecuteMsg::CreatePair {
                pair_type: PairType::Xyk {},
                asset_infos: [native_info(LUNA), native_info(USDC)],
                init_params: None,
            },
            &[],
        )
        .unwrap()
        .events
        .into_iter()
        .flat_map(|e| e.attributes)
        .find(|e| e.key == "pair_contract_addr")
        .unwrap()
        .value;

    app.execute_contract(
        Addr::unchecked(ADMIN),
        Addr::unchecked(&luna_usdc_pair_addr),
        &astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: [
                native_asset(LUNA, 500_000_000),
                native_asset(USDC, 1_000_000_000),
            ],
            slippage_tolerance: None,
            auto_stake: None,
            receiver: None,
        },
        &[Coin::new(500_000_000, LUNA), Coin::new(1_000_000_000, USDC)],
    )
    .unwrap();

    let usdt_usdc_pair_addr = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            factory.clone(),
            &astroport::factory::ExecuteMsg::CreatePair {
                pair_type: PairType::Stable {},
                asset_infos: [native_info(USDT), native_info(USDC)],
                init_params: Some(
                    to_binary(&astroport::pair::StablePoolParams { amp: 10 }).unwrap(),
                ),
            },
            &[],
        )
        .unwrap()
        .events
        .into_iter()
        .flat_map(|e| e.attributes)
        .find(|e| e.key == "pair_contract_addr")
        .unwrap()
        .value;

    app.execute_contract(
        Addr::unchecked(ADMIN),
        Addr::unchecked(&usdt_usdc_pair_addr),
        &astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: [
                native_asset(USDT, 1_000_000_000),
                native_asset(USDC, 1_000_000_000),
            ],
            slippage_tolerance: None,
            auto_stake: None,
            receiver: None,
        },
        &[
            Coin::new(1_000_000_000, USDT),
            Coin::new(1_000_000_000, USDC),
        ],
    )
    .unwrap();

    let router = app
        .instantiate_contract(
            router_code,
            Addr::unchecked(ADMIN),
            &astroport::router::InstantiateMsg {
                astroport_factory: factory.to_string(),
            },
            &[],
            "router",
            None,
        )
        .unwrap();

    let dca = app
        .instantiate_contract(
            dca_code,
            Addr::unchecked(ADMIN),
            &astroport_dca::InstantiateMsg {
                max_hops: 3,
                whitelisted_tokens: vec![native_info(LUNA), native_info(USDC), native_info(USDT)],
                max_spread: "0.005".to_string(), // 0.5%
                factory_addr: factory.to_string(),
                router_addr: router.to_string(),
                tips: vec![native_asset(USDC, 1_000_000), native_asset(USDT, 1_000_000)],
            },
            &[],
            "dca",
            None,
        )
        .unwrap();

    (app, dca)
}

#[test]
fn proper_instantiate() -> Result<(), Box<dyn Error>> {
    let (app, dca) = instantiate();

    let Config {
        max_hops,
        max_spread,
        whitelisted_tokens,
        ..
    } = app.wrap().query_wasm_smart(&dca, &QueryMsg::Config {})?;

    assert_eq!(max_hops, 3);
    assert_eq!(max_spread, Decimal::from_str("0.005")?);
    assert_eq!(
        whitelisted_tokens,
        vec![native_info(LUNA), native_info(USDC), native_info(USDT)]
    );

    let tips: Vec<Asset> = app.wrap().query_wasm_smart(&dca, &QueryMsg::Tips {})?;
    assert_eq!(
        tips,
        vec![native_asset(USDC, 1_000_000), native_asset(USDT, 1_000_000)]
    );

    Ok(())
}
