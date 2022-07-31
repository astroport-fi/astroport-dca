use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, Addr, BlockInfo, ContractInfo, Deps, Env, Order, OwnedDeps, QuerierResult,
    Storage, SystemError, SystemResult, Timestamp, Uint128,
};
use serde::de::DeserializeOwned;

use astroport_dca::dca::{DcaInfo, QueryMsg};

use crate::contract::query;
use crate::state::State;

use super::custom_querier::CustomQuerier;

pub(super) fn err_unsupported_query<T: std::fmt::Debug>(request: T) -> QuerierResult {
    SystemResult::Err(SystemError::InvalidRequest {
        error: format!("[mock] unsupported query: {:?}", request),
        request: Default::default(),
    })
}

pub(super) fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, CustomQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomQuerier::default(),
        custom_query_type: std::marker::PhantomData::default(),
    }
}

pub(super) fn mock_env_at_timestamp(timestamp: u64) -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time: Timestamp::from_seconds(timestamp),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        contract: ContractInfo {
            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        },
        transaction: None,
    }
}

pub(super) fn query_helper<T: DeserializeOwned>(deps: Deps, msg: QueryMsg) -> T {
    from_binary(&query(deps, mock_env(), msg).unwrap()).unwrap()
}

// pub(super) fn query_helper_env<T: DeserializeOwned>(
//     deps: Deps,
//     msg: QueryMsg,
//     timestamp: u64,
// ) -> T {
//     from_binary(&query(deps, mock_env_at_timestamp(timestamp), msg).unwrap()).unwrap()
// }

pub(super) fn get_orders(state: &State, storage: &dyn Storage) -> Vec<DcaInfo> {
    let orders: Vec<DcaInfo> = state
        .dca_requests
        .range(storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item.unwrap();
            v.into()
        })
        .collect();
    orders
}

pub(super) fn native(str: &str) -> AssetInfo {
    return AssetInfo::NativeToken {
        denom: str.to_string(),
    };
}

pub(super) fn token(str: &str) -> AssetInfo {
    return AssetInfo::Token {
        contract_addr: Addr::unchecked(str),
    };
}

pub(super) fn native_amount(str: &str, amount: u128) -> Asset {
    Asset {
        amount: Uint128::new(amount),
        info: native(str),
    }
}

pub(super) fn token_amount(str: &str, amount: u128) -> Asset {
    Asset {
        amount: Uint128::new(amount),
        info: token(str),
    }
}
