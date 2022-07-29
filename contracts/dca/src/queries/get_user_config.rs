use astroport::asset::addr_validate_to_lower;
use astroport_dca::UserConfig;
use cosmwasm_std::{Deps, StdResult};

use crate::state::USER_CONFIG;

/// ## Description
/// Returns the configuration set for a user to override the default contract configuration.
///
/// The result is returned in a [`UserConfig`] object.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `user` - The users lowercase address as a [`String`].
pub fn get_user_config(deps: Deps, user: String) -> StdResult<UserConfig> {
    let user_address = addr_validate_to_lower(deps.api, &user)?;

    USER_CONFIG.load(deps.storage, &user_address)
}
