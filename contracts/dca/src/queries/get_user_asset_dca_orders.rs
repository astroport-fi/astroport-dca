use astroport::asset::{addr_validate_to_lower, AssetInfo};
use astroport_dca::dca::{DcaInfo, DcaQueryInfo};
use cosmwasm_std::{Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

use crate::{get_token_allowance::get_token_allowance, state::State};

use crate::constants::{DEFAULT_LIMIT, MAX_LIMIT};

/// ## Description
/// Returns a users DCA orders currently set for a specific input asset.
///
/// The result is returned in a [`Vec<DcaQueryInfo`] object of the users current DCA orders with the
/// `amount` of each order set to the native token amount that can be spent, or the token allowance.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `user` - The users lowercase address as a [`String`].
///
/// * `asset` - Asset for which the DCA orders should be returned [`AssetInfo`].
///
/// * `start_after` - Start after the provided DCA id [`Option<u64>`].
///
/// * `limit` - Specifies how many items are returned - by default 10, max is 30 [`Option<u32>`].
pub fn get_user_asset_dca_orders(
    deps: Deps,
    env: Env,
    user: String,
    asset: AssetInfo,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<DcaQueryInfo>> {
    let addr = addr_validate_to_lower(deps.api, &user)?;

    let state = State::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|id| Bound::exclusive(id));

    let key = (user, asset.to_string());

    state
        .dca_requests
        .idx
        .user_asset
        .prefix(key)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;

            let order: DcaInfo = v.into();

            Ok(DcaQueryInfo {
                info: order.clone(),
                token_allowance: match &order.initial_asset.info {
                    AssetInfo::NativeToken { .. } => order.initial_asset.amount,
                    AssetInfo::Token { contract_addr } => {
                        // since it is a cw20 token, we need to retrieve the current allowance for the dca contract
                        get_token_allowance(&deps, &env, &addr, contract_addr)?
                    }
                },
            })
        })
        .collect()
}
