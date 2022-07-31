use astroport::asset::{addr_validate_to_lower, Asset};
use cosmwasm_std::{Deps, Env, StdResult};

use crate::state::State;

/// ## Description
/// Returns the tips stored in the contract for a user.
///
/// The result is returned in a [`Vec<Asset>`] object.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `user` - The users lowercase address as a [`String`].
pub fn get_user_tips(deps: Deps, _env: Env, user: String) -> StdResult<Vec<Asset>> {
    let addr = addr_validate_to_lower(deps.api, &user)?;
    let state = State::default();
    state.get_tip_jars(deps.storage, addr)
}
