use astroport::asset::Asset;
use cosmwasm_std::{Deps, StdResult};

use crate::state::TIPS;

/// ## Description
/// Returns the current tips denom and amount configuration as a [`Vec<Asset>`] object.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
pub fn get_tips(deps: Deps) -> StdResult<Vec<Asset>> {
    TIPS.load(deps.storage)
}
