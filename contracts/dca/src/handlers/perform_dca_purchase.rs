use astroport::{
    asset::{AssetInfo, UUSD_DENOM},
    router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
};
use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{CONFIG, DCA, USER_CONFIG},
};

/// ## Description
/// Performs a DCA purchase on behalf of another user using the hop route specified.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Params
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] from the bot who is performing a DCA purchase on behalf of another
/// user, who will be rewarded with a uusd tip.
///
/// * `user` - The address of the user as a [`String`] who is having a DCA purchase fulfilled.
///
/// * `hops` - A [`Vec<SwapOperation>`] of the hop operations to complete in the swap to purchase
/// the target asset.
pub fn perform_dca_purchase(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    hops: Vec<SwapOperation>,
) -> Result<Response, ContractError> {
    // validate user address
    let mut order = DCA.load(deps.storage, id)?;

    // retrieve configs
    let mut user_config = USER_CONFIG
        .may_load(deps.storage, &order.owner)?
        .unwrap_or_default();
    let contract_config = CONFIG.load(deps.storage)?;

    // validate hops is at least one
    if hops.is_empty() {
        return Err(ContractError::EmptyHopRoute {});
    }

    // validate hops does not exceed max_hops
    let hops_len = hops.len() as u32;
    if hops_len > user_config.max_hops.unwrap_or(contract_config.max_hops) {
        return Err(ContractError::MaxHopsAssertion { hops: hops_len });
    }

    // retrieve max_spread from user config, or default to contract set max_spread
    let max_spread = user_config.max_spread.unwrap_or(contract_config.max_spread);

    // store messages to send in response
    let mut messages: Vec<CosmosMsg> = Vec::new();

    // validate all swap operation
    for (idx, hop) in hops.iter().enumerate() {
        match hop {
            SwapOperation::NativeSwap { .. } => Err(ContractError::InvalidNativeSwap {})?,
            SwapOperation::AstroSwap {
                offer_asset_info,
                ask_asset_info,
            } => {
                // validate the first offer asset info
                (idx == 0 && offer_asset_info == &order.initial_asset.info)
                    .then(|| ())
                    .ok_or(ContractError::InitialAssetAssertion {})?;

                // validate the last ask asset info
                (idx == (hops.len() - 1) && ask_asset_info == &order.target_asset)
                    .then(|| ())
                    .ok_or(ContractError::TargetAssetAssertion {})?;

                // validate that all middle hops (last hop excluded) are whitelisted tokens for the ask_denom or ask_asset
                (idx != 0
                    && idx != (hops.len() - 1)
                    && contract_config.is_whitelisted_asset(ask_asset_info))
                .then(|| ())
                .ok_or(ContractError::InvalidHopRoute {
                    token: ask_asset_info.to_string(),
                })?;
            }
        };
    }

    // check that it has been long enough between dca purchases
    if order.last_purchase + order.interval >= env.block.time.seconds() {
        return Err(ContractError::PurchaseTooEarly {});
    }

    // subtract dca_amount from order and update last_purchase time
    order.initial_asset.amount = order
        .initial_asset
        .amount
        .checked_sub(order.dca_amount)
        .map_err(|_| ContractError::InsufficientBalance {})?;
    order.last_purchase = env.block.time.seconds();

    let funds = match &order.initial_asset.info {
        // if its a native token, we need to send the funds
        AssetInfo::NativeToken { denom } => vec![Coin {
            amount: order.dca_amount,
            denom: denom.clone(),
        }],
        //if its a token, send a TransferFrom request to the token to the router
        AssetInfo::Token { contract_addr } => {
            messages.push(
                WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    funds: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: order.owner.to_string(),
                        recipient: contract_config.router_addr.to_string(),
                        amount: order.dca_amount,
                    })?,
                }
                .into(),
            );

            vec![]
        }
    };

    // tell the router to perform swap operations
    messages.push(
        WasmMsg::Execute {
            contract_addr: contract_config.router_addr.to_string(),
            funds,
            msg: to_binary(&RouterExecuteMsg::ExecuteSwapOperations {
                operations: hops,
                minimum_receive: None,
                to: Some(order.owner.to_string()),
                max_spread: Some(max_spread),
            })?,
        }
        .into(),
    );

    // validate purchaser has enough funds to pay the sender
    let tip_cost = contract_config
        .per_hop_fee
        .checked_mul(Uint128::from(hops_len))?;
    if tip_cost >= user_config.tip_balance {
        return Err(ContractError::InsufficientTipBalance {});
    }

    // update user tip balance
    user_config.tip_balance -= tip_cost;
    USER_CONFIG.save(deps.storage, &order.owner, &user_config)?;

    // add tip payment to messages
    messages.push(
        BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                amount: tip_cost,
                denom: UUSD_DENOM.to_string(),
            }],
        }
        .into(),
    );

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "perform_dca_purchase"),
        attr("id", id.to_string()),
        attr("tip_cost", tip_cost),
    ]))
}
