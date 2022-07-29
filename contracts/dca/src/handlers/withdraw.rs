use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, MessageInfo, Response, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{error::ContractError, state::USER_CONFIG};

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

        msgs.push(match asset.info {
            AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: asset.amount,
                })?,
                funds: vec![],
            }),
            AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom,
                    amount: asset.amount,
                }],
            }),
        });
    }

    USER_CONFIG.save(deps.storage, &info.sender, &config)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "withdraw")]))
}
