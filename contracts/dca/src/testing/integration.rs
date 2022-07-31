use std::str::FromStr;
use std::vec;

use astroport::asset::{Asset, AssetInfo};
use astroport::factory::{
    ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg, PairConfig, PairType,
};

use astroport::router::{InstantiateMsg as RouterInstantiateMsg, SwapOperation};

use astroport_dca::dca::{DcaQueryInfo, InstantiateMsg, QueryMsg, ReceiveMsg, TipAssetInfo};

use astroport::token::InstantiateMsg as TokenInstantiateMsg;

use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{coin, to_binary, Addr, Coin, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw_multi_test::{App, AppBuilder, BankKeeper, ContractWrapper, Executor};

use crate::testing::helpers::{native, token};

use super::helpers::{native_amount, token_amount};

const OWNER: &str = "owner";

fn mock_app(bank: BankKeeper) -> App {
    let env = mock_env();
    let api = MockApi::default();
    let mut storage = MockStorage::new();

    bank.init_balance(
        &mut storage,
        &Addr::unchecked("alice"),
        vec![coin(100_000_000_000, "uluna")],
    )
    .unwrap();

    bank.init_balance(
        &mut storage,
        &Addr::unchecked("bot"),
        vec![coin(100, "uluna")],
    )
    .unwrap();

    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .build(|_, _, _| {})
}

fn store_token_code(app: &mut App) -> u64 {
    let astro_token_contract = Box::new(ContractWrapper::new_with_empty(
        astroport_token::contract::execute,
        astroport_token::contract::instantiate,
        astroport_token::contract::query,
    ));

    app.store_code(astro_token_contract)
}

fn store_pair_code(app: &mut App) -> u64 {
    let pair_contract = Box::new(
        ContractWrapper::new_with_empty(
            astroport_pair::contract::execute,
            astroport_pair::contract::instantiate,
            astroport_pair::contract::query,
        )
        .with_reply_empty(astroport_pair::contract::reply),
    );

    app.store_code(pair_contract)
}

fn store_router_code(app: &mut App) -> u64 {
    let router_contract = Box::new(ContractWrapper::new(
        astroport_router::contract::execute,
        astroport_router::contract::instantiate,
        astroport_factory::contract::query,
    ));

    app.store_code(router_contract)
}

fn store_factory_code(app: &mut App) -> u64 {
    let factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            astroport_factory::contract::execute,
            astroport_factory::contract::instantiate,
            astroport_factory::contract::query,
        )
        .with_reply_empty(astroport_factory::contract::reply),
    );

    app.store_code(factory_contract)
}

fn store_dca(app: &mut App) -> u64 {
    let dca_contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));

    app.store_code(dca_contract)
}

#[test]
fn test_example_bounty_happy_path() {
    let bank = BankKeeper::new();
    let mut app = mock_app(bank);

    let owner = Addr::unchecked(OWNER);
    let alice_address = Addr::unchecked("alice");
    let bot_address = Addr::unchecked("bot");

    let token_code_id = store_token_code(&mut app);
    let factory_code_id = store_factory_code(&mut app);
    let router_code_id = store_router_code(&mut app);
    let pair_code_id = store_pair_code(&mut app);
    let dca_code_id = store_dca(&mut app);

    let astro_token = create_token(&mut app, token_code_id, "astro");
    let usdc_token = create_token(&mut app, token_code_id, "ibcusdc");
    let usdt_token = create_token(&mut app, token_code_id, "ibcusdt");
    let any_token = create_token(&mut app, token_code_id, "any");
    let a_token = create_token(&mut app, token_code_id, "tokena");
    let b_token = create_token(&mut app, token_code_id, "tokenb");
    let c_token = create_token(&mut app, token_code_id, "tokenc");

    let uluna_asset = native("uluna");
    let astro_asset = token(astro_token.to_string().as_str());
    let usdc_asset = token(usdc_token.to_string().as_str());
    let usdt_asset = token(usdt_token.to_string().as_str());
    let _any_asset = token(any_token.to_string().as_str());
    let a_asset = token(a_token.to_string().as_str());
    let b_asset = token(b_token.to_string().as_str());
    let c_asset = token(c_token.to_string().as_str());

    let init_factory = FactoryInstantiateMsg {
        fee_address: None,
        pair_configs: vec![PairConfig {
            code_id: pair_code_id,
            maker_fee_bps: 0,
            pair_type: PairType::Xyk {},
            total_fee_bps: 0,
            is_disabled: false,
            is_generator_disabled: false,
        }],
        token_code_id,
        generator_address: Some(String::from("generator")),
        owner: owner.to_string(),
        whitelist_code_id: 234u64,
    };

    let factory_contract = app
        .instantiate_contract(
            factory_code_id,
            owner.clone(),
            &init_factory,
            &[],
            "FACTORY",
            None,
        )
        .unwrap();

    let init_router = RouterInstantiateMsg {
        astroport_factory: factory_contract.to_string(),
    };

    let router_contract = app
        .instantiate_contract(
            router_code_id,
            owner.clone(),
            &init_router,
            &[],
            "ROUTER",
            None,
        )
        .unwrap();

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        native_amount("uluna", 100000),
        token_amount(astro_token.to_string().as_str(), 4000000),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        native_amount("uluna", 100000),
        token_amount(usdc_token.to_string().as_str(), 202000),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        native_amount("uluna", 100000),
        token_amount(usdt_token.to_string().as_str(), 200000),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        native_amount("uluna", 100),
        token_amount(any_token.to_string().as_str(), 100),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        token_amount(a_token.to_string().as_str(), 10000000),
        token_amount(astro_token.to_string().as_str(), 400000000),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        native_amount("uluna", 100000),
        token_amount(a_token.to_string().as_str(), 100000),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        native_amount("uluna", 100),
        token_amount(b_token.to_string().as_str(), 100),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        native_amount("uluna", 100),
        token_amount(c_token.to_string().as_str(), 100),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        token_amount(astro_token.to_string().as_str(), 1000),
        token_amount(usdc_token.to_string().as_str(), 20000),
    );

    add_pair_and_liquidity(
        &mut app,
        &factory_contract,
        token_amount(usdt_token.to_string().as_str(), 100000),
        token_amount(usdc_token.to_string().as_str(), 100000),
    );

    let init_dca = InstantiateMsg {
        max_hops: 2,
        whitelisted_tokens: vec![uluna_asset.clone(), astro_asset.clone()],
        whitelisted_tip_tokens: vec![
            TipAssetInfo {
                info: usdc_asset.clone(),
                per_hop_fee: Uint128::new(1),
            },
            TipAssetInfo {
                info: usdt_asset.clone(),
                per_hop_fee: Uint128::new(2),
            },
            TipAssetInfo {
                info: uluna_asset.clone(),
                per_hop_fee: Uint128::new(10),
            },
        ],
        max_spread: Decimal::from_str("0.01").unwrap(),
        factory_addr: factory_contract.to_string(),
        router_addr: router_contract.to_string(),
    };

    let dca_contract = app
        .instantiate_contract(dca_code_id, owner.clone(), &init_dca, &[], "DCA", None)
        .unwrap();

    increase_allowance(
        &mut app,
        &dca_contract.to_string(),
        &Asset {
            info: a_asset.clone(),
            amount: Uint128::new(2),
        },
    );
    increase_allowance(
        &mut app,
        &dca_contract.to_string(),
        &Asset {
            info: b_asset.clone(),
            amount: Uint128::new(5),
        },
    );
    increase_allowance(
        &mut app,
        &dca_contract.to_string(),
        &Asset {
            info: c_asset.clone(),
            amount: Uint128::new(10),
        },
    );

    let max_spread = Some(Decimal::from_str("0.05").unwrap());
    let create_dca = astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
        initial_asset: Asset {
            info: a_asset.clone(),
            amount: Uint128::new(2),
        },
        target_asset: astro_asset.clone(),
        interval: 12 * 60 * 60,
        dca_amount: Uint128::new(1),
        start_purchase: None,
        max_hops: None,
        max_spread: max_spread.clone(),
    };

    let _res = app
        .execute_contract(
            alice_address.clone(),
            dca_contract.clone(),
            &create_dca,
            &[],
        )
        .unwrap();

    let create_dca = astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
        initial_asset: Asset {
            info: b_asset.clone(),
            amount: Uint128::new(5),
        },
        target_asset: astro_asset.clone(),
        interval: 24 * 60 * 60,
        dca_amount: Uint128::new(1),
        start_purchase: None,
        max_hops: None,
        max_spread: max_spread.clone(),
    };

    let _res = app
        .execute_contract(
            alice_address.clone(),
            dca_contract.clone(),
            &create_dca,
            &[],
        )
        .unwrap();

    let create_dca = astroport_dca::dca::ExecuteMsg::CreateDcaOrder {
        initial_asset: Asset {
            info: c_asset.clone(),
            amount: Uint128::new(10),
        },
        target_asset: uluna_asset.clone(),
        interval: 7 * 24 * 60 * 60,
        dca_amount: Uint128::new(5),
        start_purchase: None,
        max_hops: None,
        max_spread: max_spread.clone(),
    };

    let _res = app
        .execute_contract(
            alice_address.clone(),
            dca_contract.clone(),
            &create_dca,
            &[],
        )
        .unwrap();

    let res: Vec<DcaQueryInfo> = app
        .wrap()
        .query_wasm_smart(
            &dca_contract,
            &QueryMsg::UserDcaOrders {
                user: alice_address.to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    let ids: Vec<u64> = res.iter().map(|item| item.info.id).collect();
    assert_eq!(ids, vec![1, 2, 3]);

    let add_bot_tip = astroport_dca::dca::ExecuteMsg::AddBotTip {
        asset: native_amount("uluna", 100),
    };

    let _res = app
        .execute_contract(
            alice_address.clone(),
            dca_contract.clone(),
            &add_bot_tip,
            &[coin(100, "uluna")],
        )
        .unwrap();

    let add_bot_tip = Cw20ExecuteMsg::Send {
        contract: dca_contract.to_string(),
        amount: Uint128::new(100),
        msg: to_binary(&ReceiveMsg::AddBotTip {}).unwrap(),
    };

    let _res = app
        .execute_contract(alice_address.clone(), usdc_token.clone(), &add_bot_tip, &[])
        .unwrap();

    let bot_balance = get_luna_balance(&app, &bot_address);
    let alice_a_balance = get_cw20_balance(&app, &a_token, &alice_address);
    let alice_astro_balance = get_cw20_balance(&app, &astro_token, &alice_address);

    let perform_purchase = astroport_dca::dca::ExecuteMsg::PerformDcaPurchase {
        id: 1,
        hops: vec![SwapOperation::AstroSwap {
            offer_asset_info: a_asset.clone(),
            ask_asset_info: astro_asset.clone(),
        }],
    };

    let _res = app
        .execute_contract(
            bot_address.clone(),
            dca_contract.clone(),
            &perform_purchase,
            &[],
        )
        .unwrap();

    let bot_balance2 = get_luna_balance(&app, &bot_address);
    let alice_a_balance2 = get_cw20_balance(&app, &a_token, &alice_address);
    let alice_astro_balance2 = get_cw20_balance(&app, &astro_token, &alice_address);

    assert_eq!(bot_balance2.u128(), bot_balance.u128() + 10);
    assert_eq!(alice_a_balance2.u128(), alice_a_balance.u128() - 1);
    assert_eq!(alice_astro_balance2.u128(), alice_astro_balance.u128() + 39);
}

fn get_cw20_balance(app: &App, token: &Addr, address: &Addr) -> Uint128 {
    let result: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            token,
            &Cw20QueryMsg::Balance {
                address: address.to_string(),
            },
        )
        .unwrap();
    result.balance
}

fn get_luna_balance(app: &App, address: &Addr) -> Uint128 {
    app.wrap()
        .query_balance(address.clone(), "uluna")
        .unwrap()
        .amount
}

fn create_token(app: &mut App, token_code_id: u64, token_name: &str) -> Addr {
    let owner = Addr::unchecked(OWNER);
    let init_token_msg = TokenInstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: "alice".to_string(),
            amount: Uint128::from(10_000_000_000u128),
        }],
        mint: Some(MinterResponse {
            minter: token_name.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    app.instantiate_contract(token_code_id, owner, &init_token_msg, &[], token_name, None)
        .unwrap()
}

fn add_pair_and_liquidity(app: &mut App, factory_contract: &Addr, asset1: Asset, asset2: Asset) {
    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::Xyk {},
        asset_infos: [asset1.info.clone(), asset2.info.clone()],
        init_params: None,
    };
    let result = app
        .execute_contract(Addr::unchecked(OWNER), factory_contract.clone(), &msg, &[])
        .unwrap();

    let pair_contract_addr = result
        .events
        .iter()
        .rev()
        .find(|ev| ev.ty == "wasm")
        .map(|ev| {
            ev.attributes
                .iter()
                .find(|a| a.key == "pair_contract_addr")
                .map(|a| a.value.clone())
                .unwrap()
        })
        .unwrap();

    increase_allowance(app, &pair_contract_addr, &asset1);
    increase_allowance(app, &pair_contract_addr, &asset2);

    let msg = astroport::pair::ExecuteMsg::ProvideLiquidity {
        assets: [asset1.clone(), asset2],
        slippage_tolerance: None,
        auto_stake: None,
        receiver: None,
    };

    app.execute_contract(
        Addr::unchecked("alice"),
        Addr::unchecked(pair_contract_addr),
        &msg,
        &get_funds(&asset1),
    )
    .unwrap();
}

fn increase_allowance(app: &mut App, spender: &String, asset: &Asset) -> () {
    match asset.info.clone() {
        AssetInfo::Token { contract_addr } => {
            let msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.clone(),
                expires: None,
                amount: asset.amount,
            };

            app.execute_contract(Addr::unchecked("alice"), contract_addr, &msg, &[])
                .unwrap();
        }
        _ => {}
    }
}

fn get_funds(asset1: &Asset) -> Vec<Coin> {
    match asset1.info.clone() {
        AssetInfo::NativeToken { denom } => {
            vec![coin(asset1.amount.u128(), denom)]
        }
        AssetInfo::Token { .. } => {
            vec![]
        }
    }
}
