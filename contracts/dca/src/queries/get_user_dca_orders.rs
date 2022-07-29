use astroport::asset::{addr_validate_to_lower, AssetInfo};
use astroport_dca::UserDcaInfo;
use cosmwasm_std::{Deps, Env, Order, StdResult};

use crate::{
    get_token_allowance::get_token_allowance,
    state::{DCA, DCA_OWNER},
};

/// ## Description
/// Returns a users DCA orders currently set.
///
/// The result is returned in a [`Vec<UserDcaInfo>`] object of the users current DCA orders with the
/// `amount` of each order set to the native token amount that can be spent, or the token allowance.
///
/// ## Arguments
/// * `deps` - A [`Deps`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `user` - The users lowercase address as a [`String`].
pub fn get_user_dca_orders(deps: Deps, env: Env, user: String) -> StdResult<Vec<UserDcaInfo>> {
    let user_address = addr_validate_to_lower(deps.api, &user)?;

    DCA_OWNER
        .prefix(&user_address)
        .keys(deps.storage, None, None, Order::Descending)
        .map(|e| -> StdResult<_> {
            let order = DCA.load(deps.storage, e?)?;
            Ok(UserDcaInfo {
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
