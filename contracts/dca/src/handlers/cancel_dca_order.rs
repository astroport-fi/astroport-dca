use astroport::asset::AssetInfo;
use cosmwasm_std::{attr, BankMsg, Coin, DepsMut, MessageInfo, Response, Uint128};

use crate::{
    error::ContractError,
    state::{DCA, DCA_OWNER},
};

/// ## Description
/// Cancels a users DCA purchase so that it will no longer be fulfilled.
///
/// Returns the `initial_asset` back to the user if it was a native token.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to cancel their order.
///
/// * `initial_asset` The [`AssetInfo`] which the user wants to cancel the DCA order for.
pub fn cancel_dca_order(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut funds = Vec::new();
    let order = DCA.load(deps.storage, id)?;

    (order.owner == info.sender)
        .then(|| ())
        .ok_or(ContractError::Unauthorized {})?;

    // remove order from user dca's, and add any native token funds for `initial_asset` into the `funds`.
    if let AssetInfo::NativeToken { denom } = order.initial_asset.info {
        if order.initial_asset.amount > Uint128::zero() {
            funds.push(BankMsg::Send {
                to_address: order.owner.to_string(),
                amount: vec![Coin {
                    denom,
                    amount: order.initial_asset.amount,
                }],
            })
        }
    }

    DCA.remove(deps.storage, id);
    DCA_OWNER.remove(deps.storage, (&order.owner, id));

    Ok(Response::new().add_messages(funds).add_attributes(vec![
        attr("action", "cancel_dca_order"),
        attr("id", id.to_string()),
    ]))
}
