use astroport_dca::dca::ConfigResponse;
use cosmwasm_std::{Deps, StdResult};

use crate::state::State;

/// ## Description
/// Returns the contract configuration set by the factory address owner or contract instantiator.
///
/// The result is returned in a [`Config`] object.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
pub fn get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let whitelisted_tip_tokens = state.whitelisted_tip_tokens.load(deps.storage)?;

    Ok(ConfigResponse {
        factory_addr: config.factory_addr,
        max_hops: config.max_hops,
        max_spread: config.max_spread,
        router_addr: config.router_addr,
        whitelisted_tokens: config.whitelisted_tokens,
        whitelisted_tip_tokens,
    })
}
