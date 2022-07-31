use astroport::{
    asset::{Asset, AssetInfo},
    router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
};
use cosmwasm_std::{
    attr, to_binary, Addr, Coin, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response, Storage,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{error::ContractError, state::State};

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
    let state = State::default();
    let mut order = state
        .dca_requests
        .load(deps.storage, id)
        .or_else(|_| Err(ContractError::NonExistentDca {}))?;

    let config = state.config.load(deps.storage)?;

    // validate hops is at least one
    if hops.is_empty() {
        return Err(ContractError::EmptyHopRoute {});
    }

    // validate hops does not exceed max_hops
    let hops_len = hops.len() as u32;
    if hops_len > order.max_hops.unwrap_or(config.max_hops) {
        return Err(ContractError::MaxHopsAssertion { hops: hops_len });
    }

    // validate no native swap
    for swap in hops.iter() {
        match swap {
            SwapOperation::NativeSwap { .. } => {
                return Err(ContractError::NativeSwapNotSupported {})
            }
            SwapOperation::AstroSwap { .. } => {}
        }
    }

    // validate that all middle hops (last hop excluded) are whitelisted tokens for the ask_denom or ask_asset
    let middle_hops = &hops[..hops.len() - 1];
    for swap in middle_hops {
        match swap {
            SwapOperation::NativeSwap { .. } => {}
            SwapOperation::AstroSwap { ask_asset_info, .. } => {
                if !config.is_whitelisted_asset(ask_asset_info) {
                    return Err(ContractError::InvalidHopRoute {
                        token: ask_asset_info.to_string(),
                    });
                }
            }
        }
    }

    // retrieve max_spread from user config, or default to contract set max_spread
    let max_spread = order.max_spread.unwrap_or(config.max_spread);

    // store messages to send in response
    let mut messages: Vec<CosmosMsg> = Vec::new();

    if let Some(start_purchase) = order.start_purchase {
        if start_purchase > env.block.time.seconds() {
            return Err(ContractError::PurchaseTooEarly {});
        }
    }

    // check that it has been long enough between dca purchases
    if order.last_purchase > 0 && order.last_purchase + order.interval > env.block.time.seconds() {
        return Err(ContractError::PurchaseTooEarly {});
    }

    // check that last hop is target asset
    let last_hop = &hops
        .last()
        .ok_or(ContractError::EmptyHopRoute {})?
        .get_target_asset_info();

    if last_hop != &order.target_asset {
        return Err(ContractError::TargetAssetAssertion {});
    }

    // subtract dca_amount from order and update last_purchase time
    order.initial_asset.amount = order
        .initial_asset
        .amount
        .checked_sub(order.dca_amount)
        .map_err(|_| ContractError::InsufficientBalance {})?;

    // validate purchaser has enough funds to pay the sender
    let tip_payment = take_payment_from_tip_jar(deps.storage, order.user.clone(), hops_len)?;

    order.last_purchase = env.block.time.seconds();

    // add funds and router message to response
    if let AssetInfo::Token { contract_addr } = &order.initial_asset.info {
        // send a TransferFrom request to the token to the router
        messages.push(
            WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: order.user.to_string(),
                    recipient: config.router_addr.to_string(),
                    amount: order.dca_amount,
                })?,
            }
            .into(),
        );
    }

    // if it is a native token, we need to send the funds
    let funds = match &order.initial_asset.info {
        AssetInfo::NativeToken { denom } => vec![Coin {
            amount: order.dca_amount,
            denom: denom.clone(),
        }],
        AssetInfo::Token { .. } => vec![],
    };

    // tell the router to perform swap operations
    messages.push(
        WasmMsg::Execute {
            contract_addr: config.router_addr.to_string(),
            funds,
            msg: to_binary(&RouterExecuteMsg::ExecuteSwapOperations {
                operations: hops,
                minimum_receive: None,
                to: Some(order.user.to_string()),
                max_spread: Some(max_spread),
            })?,
        }
        .into(),
    );

    let event: Event;

    if order.initial_asset.amount.is_zero() {
        state.dca_requests.remove(deps.storage, id)?;
        event = Event::new("astroport-dca/perform-finished")
            .add_attribute("id", order.id.to_string())
            .add_attribute("user", order.user.to_string());
    } else {
        state.dca_requests.save(deps.storage, id, &order)?;
        event = Event::new("astroport-dca/perform-executed")
            .add_attribute("id", order.id.to_string())
            .add_attribute("user", order.user.to_string());
    }

    // add tip payment to messages
    messages.push(
        tip_payment
            .clone()
            .into_msg(&deps.querier, info.sender.to_string())?,
    );

    Ok(Response::new()
        .add_messages(messages)
        .add_event(event)
        .add_attributes(vec![
            attr("action", "perform_dca_purchase"),
            attr("tip_cost", tip_payment.amount),
            attr("tip_asset", tip_payment.info.to_string()),
        ]))
}

/// This function takes a tip payment from the tip jars of the user.
///
/// # Errors
///
/// This function will return an error if no tip jar with enough funds is found.
fn take_payment_from_tip_jar(
    storage: &mut dyn Storage,
    user: Addr,
    hops_len: u32,
) -> Result<Asset, ContractError> {
    // iterates the available tip jars of the user and if it finds a whitelisted token it will take it.
    let state = State::default();

    let whitelisted_tip_tokens = state.whitelisted_tip_tokens.load(storage)?;
    let mut user_tip_jars = state
        .get_tip_jars(storage, user.clone())
        .map_err(|_| ContractError::NoTipBalance {})?;

    // use for to not move elements out of the vector
    for index in 0..user_tip_jars.len() {
        let tip_jar = &user_tip_jars[index];

        let whitelisted_tip_token = whitelisted_tip_tokens
            .iter()
            .find(|token| token.info == tip_jar.info);

        if let Some(whitelisted_tip_token) = whitelisted_tip_token {
            // token per_hop_fee * hops_len
            let tip_cost = whitelisted_tip_token
                .per_hop_fee
                .checked_mul(Uint128::from(hops_len))?;

            if tip_cost <= tip_jar.amount {
                user_tip_jars[index].amount = tip_jar.amount.checked_sub(tip_cost)?;
                let info = user_tip_jars[index].info.clone();

                if user_tip_jars[index].amount.is_zero() {
                    // remove jar when emptied
                    user_tip_jars.remove(index);
                }

                state.tip_jars.save(storage, user, &user_tip_jars)?;

                return Ok(Asset {
                    info,
                    amount: tip_cost,
                });
            }
        }
    }

    Err(ContractError::InsufficientTipBalance {})
}
