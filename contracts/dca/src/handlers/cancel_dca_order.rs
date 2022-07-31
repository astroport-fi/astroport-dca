use cosmwasm_std::{attr, DepsMut, MessageInfo, Response};

use crate::{error::ContractError, state::State};

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
    let state = State::default();
    let dca = state.dca_requests.load(deps.storage, id)?;

    if dca.user != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut msgs = Vec::new();

    if dca.initial_asset.is_native_token() {
        msgs.push(dca.initial_asset.into_msg(&deps.querier, dca.user)?);
    }

    state.dca_requests.remove(deps.storage, id)?;

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        attr("action", "cancel_dca_order"),
        attr("id", id.to_string()),
    ]))
}
