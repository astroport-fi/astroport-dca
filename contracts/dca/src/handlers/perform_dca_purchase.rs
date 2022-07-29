use astroport::{
    asset::{addr_validate_to_lower, AssetInfo, UUSD_DENOM},
    router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
};
use astroport_dca::{DcaInfo, UserConfig};
use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{CONFIG, USER_CONFIG, USER_DCA},
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
    user: String,
    hops: Vec<SwapOperation>,
) -> Result<Response, ContractError> {
    // validate user address
    let user_address = addr_validate_to_lower(deps.api, &user)?;

    // retrieve configs
    let user_config = USER_CONFIG
        .may_load(deps.storage, &user_address)?
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

    // validate that all middle hops (last hop excluded) are whitelisted tokens for the ask_denom or ask_asset
    let middle_hops = &hops[..hops.len() - 1];
    for swap in middle_hops {
        match swap {
            SwapOperation::NativeSwap { ask_denom, .. } => {
                if !contract_config
                    .whitelisted_tokens
                    .iter()
                    .any(|token| match token {
                        AssetInfo::NativeToken { denom } => ask_denom == denom,
                        AssetInfo::Token { .. } => false,
                    })
                {
                    // not a whitelisted native token
                    return Err(ContractError::InvalidHopRoute {
                        token: ask_denom.to_string(),
                    });
                }
            }
            SwapOperation::AstroSwap { ask_asset_info, .. } => {
                if !contract_config.is_whitelisted_asset(ask_asset_info) {
                    return Err(ContractError::InvalidHopRoute {
                        token: ask_asset_info.to_string(),
                    });
                }
            }
        }
    }

    // validate purchaser has enough funds to pay the sender
    let tip_cost = contract_config
        .per_hop_fee
        .checked_mul(Uint128::from(hops_len))?;
    if tip_cost > user_config.tip_balance {
        return Err(ContractError::InsufficientTipBalance {});
    }

    // retrieve max_spread from user config, or default to contract set max_spread
    let max_spread = user_config.max_spread.unwrap_or(contract_config.max_spread);

    // store messages to send in response
    let mut messages: Vec<CosmosMsg> = Vec::new();

    // load user dca orders and update the relevant one
    USER_DCA.update(
        deps.storage,
        &user_address,
        |orders| -> Result<Vec<DcaInfo>, ContractError> {
            let mut orders = orders.ok_or(ContractError::NonexistentDca {})?;

            let order = orders
                .iter_mut()
                .find(|order| match &hops[0] {
                    SwapOperation::NativeSwap { ask_denom, .. } => {
                        match &order.initial_asset.info {
                            AssetInfo::NativeToken { denom } => ask_denom == denom,
                            _ => false,
                        }
                    }
                    SwapOperation::AstroSwap {
                        offer_asset_info, ..
                    } => offer_asset_info == &order.initial_asset.info,
                })
                .ok_or(ContractError::NonexistentDca {})?;

            // check that it has been long enough between dca purchases
            if order.last_purchase + order.interval > env.block.time.seconds() {
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
            order.last_purchase = env.block.time.seconds();

            // add funds and router message to response
            if let AssetInfo::Token { contract_addr } = &order.initial_asset.info {
                // send a TransferFrom request to the token to the router
                messages.push(
                    WasmMsg::Execute {
                        contract_addr: contract_addr.to_string(),
                        funds: vec![],
                        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                            owner: user_address.to_string(),
                            recipient: contract_config.router_addr.to_string(),
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
                    contract_addr: contract_config.router_addr.to_string(),
                    funds,
                    msg: to_binary(&RouterExecuteMsg::ExecuteSwapOperations {
                        operations: hops,
                        minimum_receive: None,
                        to: Some(user_address.to_string()),
                        max_spread: Some(max_spread),
                    })?,
                }
                .into(),
            );

            Ok(orders)
        },
    )?;

    // remove tip from purchaser
    USER_CONFIG.update(
        deps.storage,
        &user_address,
        |user_config| -> Result<UserConfig, ContractError> {
            let mut user_config = user_config.unwrap_or_default();

            user_config.tip_balance = user_config
                .tip_balance
                .checked_sub(tip_cost)
                .map_err(|_| ContractError::InsufficientTipBalance {})?;

            Ok(user_config)
        },
    )?;

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
        attr("tip_cost", tip_cost),
    ]))
}
