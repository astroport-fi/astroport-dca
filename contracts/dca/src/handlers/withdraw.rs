use astroport::asset::Asset;
use cosmwasm_std::{attr, CosmosMsg, DepsMut, MessageInfo, Response};

use crate::{error::ContractError, helpers::asset_transfer, state::USER_CONFIG};

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
    tips: Vec<Asset>,
) -> Result<Response, ContractError> {
    let mut config = USER_CONFIG
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();
    let mut msgs: Vec<CosmosMsg> = vec![];

    for asset in tips {
        match config
            .tips_balance
            .iter_mut()
            .enumerate()
            .find(|e| e.1.info == asset.info)
        {
            Some((idx, bal)) => {
                match bal.amount == asset.amount {
                    // withdraw all
                    true => {
                        config.tips_balance.remove(idx);
                    }
                    false => {
                        bal.amount = bal
                            .amount
                            .checked_sub(asset.amount)
                            .map_err(|_| ContractError::InsufficientTipWithdrawBalance {})?;
                    }
                };
            }
            None => {
                Err(ContractError::InsufficientTipWithdrawBalance {})?;
            }
        };

        msgs.push(asset_transfer(&asset.info, asset.amount, &info.sender)?);
    }

    USER_CONFIG.save(deps.storage, &info.sender, &config)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "withdraw")]))
}
