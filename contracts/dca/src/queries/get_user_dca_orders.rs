use astroport::asset::{addr_validate_to_lower, AssetInfo};
use astroport_dca::DcaQueryInfo;
use cosmwasm_std::{Deps, Env, StdResult};

use crate::{get_token_allowance::get_token_allowance, state::USER_DCA};

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
pub fn get_user_dca_orders(deps: Deps, env: Env, user: String) -> StdResult<Vec<DcaQueryInfo>> {
    let user_address = addr_validate_to_lower(deps.api, &user)?;

    USER_DCA
        .load(deps.storage, &user_address)?
        .into_iter()
        .map(|order| {
            Ok(DcaQueryInfo {
                token_allowance: match &order.initial_asset.info {
                    AssetInfo::NativeToken { .. } => order.initial_asset.amount,
                    AssetInfo::Token { contract_addr } => {
                        // since it is a cw20 token, we need to retrieve the current allowance for the dca contract
                        get_token_allowance(&deps, &env, &user_address, contract_addr)?
                    }
                },
                info: order,
            })
        })
        .collect::<StdResult<Vec<_>>>()
}
