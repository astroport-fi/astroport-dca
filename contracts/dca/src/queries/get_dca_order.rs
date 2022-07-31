use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{Deps, Env, StdResult};

use crate::state::State;

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
pub fn get_dca_order(deps: Deps, _env: Env, id: u64) -> StdResult<DcaInfo> {
    let state = State::default();
    Ok(state.dca_requests.load(deps.storage, id)?)
}
