use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

use crate::{
    constants::{DEFAULT_LIMIT, MAX_LIMIT},
    state::State,
};

/// ## Description
/// Returns a users DCA orders currently set.
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
pub fn get_dca_orders(
    deps: Deps,
    _env: Env,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<DcaInfo>> {
    let state = State::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|id| Bound::exclusive(id));

    state
        .dca_requests
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;

            Ok(v.into())
        })
        .collect()
}
