use astroport::asset::Asset;
use cosmwasm_std::{attr, CosmosMsg, DepsMut, MessageInfo, Response};

use crate::{error::ContractError, state::State};

/// ## Description
/// Withdraws a users bot tip from the contract.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to withdraw their bot tip.
///
/// * `amount`` - A [`Uint128`] representing the amount of uusd to send back to the user.
pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    assets: Option<Vec<Asset>>,
) -> Result<Response, ContractError> {
    let state = State::default();
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    let mut attrs = vec![attr("action", "withdraw")];
    let mut tip_jars = state
        .get_tip_jars(deps.storage, info.sender.clone())
        .map_err(|_| ContractError::NonExistentTipJar {
            token: "*".to_string(),
        })?;

    if let Some(assets) = assets {
        // if asssets provided, check if enough balance for withdraw and token exists
        for asset in assets {
            let tip_jar = tip_jars.iter_mut().find(|jar| jar.info == asset.info);

            if let Some(tip_jar) = tip_jar {
                tip_jar.amount = tip_jar
                    .amount
                    .checked_sub(asset.amount.clone())
                    .map_err(|_| ContractError::InsufficientTipBalance {})?;
            } else {
                return Err(ContractError::NonExistentTipJar {
                    token: asset.info.to_string(),
                });
            }

            msgs.push(asset.clone().into_msg(&deps.querier, info.sender.clone())?);
            attrs.push(attr("tip_token", asset.info.to_string()));
            attrs.push(attr("tip_removed", asset.amount.clone()));
        }

        tip_jars = tip_jars
            .into_iter()
            .filter(|jar| !jar.amount.is_zero())
            .collect();

        state.tip_jars.save(deps.storage, info.sender, &tip_jars)?;
    } else {
        // if no assets provided, return all tip jars to the user and reset jars

        for jar in tip_jars {
            msgs.push(jar.clone().into_msg(&deps.querier, info.sender.clone())?);
            attrs.push(attr("tip_token", jar.info.to_string()));
            attrs.push(attr("tip_removed", jar.amount.clone()));
        }

        state.tip_jars.save(deps.storage, info.sender, &vec![])?;
    }

    Ok(Response::new().add_attributes(attrs).add_messages(msgs))
}
