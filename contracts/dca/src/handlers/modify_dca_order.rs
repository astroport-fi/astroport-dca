use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::ModifyDcaOrderParameters;
use cosmwasm_std::{attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError};

use crate::{error::ContractError, get_token_allowance::get_token_allowance, state::State};

/// ## Description
/// Modifies an existing DCA order for a user such that the new parameters will apply to the
/// existing order.
///
/// If the user increases the size of their order, they must allocate the correct amount of new
/// assets to the contract.
///
/// If the user decreases the size of their order, they will be refunded with the difference.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to modify their order, containing the
/// [`AssetInfo::NativeToken`] if the DCA order is being increased in size.
///
/// * `order_details` - The [`ModifyDcaOrderParameters`] details about the old and new DCA order
/// parameters.
pub fn modify_dca_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_details: ModifyDcaOrderParameters,
) -> Result<Response, ContractError> {
    let ModifyDcaOrderParameters {
        id,
        new_initial_asset,
        new_target_asset,
        new_interval,
        new_dca_amount,
        should_reset_purchase_time,
        max_hops,
        max_spread,
        start_purchase,
    } = order_details;

    let state = State::default();
    let mut order = state.dca_requests.load(deps.storage, id)?;

    if order.user != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let orig_asset = order.initial_asset.clone();

    let should_refund = order.initial_asset.amount > new_initial_asset.amount;

    let asset_difference = Asset {
        info: new_initial_asset.info.clone(),
        amount: match should_refund {
            true => order
                .initial_asset
                .amount
                .checked_sub(new_initial_asset.amount)?,
            false => new_initial_asset
                .amount
                .checked_sub(order.initial_asset.amount)?,
        },
    };

    let mut messages: Vec<CosmosMsg> = Vec::new();

    if order.initial_asset.info == new_initial_asset.info {
        if !should_refund {
            // if the user needs to have deposited more, check that we have the correct funds/allowance sent
            // this is the case only when the old_initial_asset and new_initial_asset are the same

            // if native token, they should have included it in the message
            // otherwise, if cw20 token, they should have provided the correct allowance
            match &order.initial_asset.info {
                AssetInfo::NativeToken { .. } => {
                    asset_difference.assert_sent_native_token_balance(&info)?
                }
                AssetInfo::Token { contract_addr } => {
                    let allowance =
                        get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
                    if allowance != new_initial_asset.amount {
                        return Err(ContractError::InvalidTokenDeposit {});
                    }
                }
            }
        } else {
            // we need to refund the user with the difference if it is a native token
            if new_initial_asset.info.is_native_token() {
                messages.push(asset_difference.into_msg(&deps.querier, info.sender)?)
            }
        }
    } else {
        // they are different assets, so we will return the old_initial_asset if it is a native token
        if new_initial_asset.info.is_native_token() {
            messages.push(
                order
                    .initial_asset
                    .into_msg(&deps.querier, info.sender.clone())?,
            )
        }

        // validate that user sent either native tokens or has set allowance for the new token
        match &new_initial_asset.info {
            AssetInfo::NativeToken { .. } => {
                new_initial_asset.assert_sent_native_token_balance(&info)?
            }
            AssetInfo::Token { contract_addr } => {
                let allowance =
                    get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
                if allowance != new_initial_asset.amount {
                    return Err(ContractError::InvalidTokenDeposit {});
                }
            }
        }
    }

    // update order
    order.initial_asset = new_initial_asset.clone();
    order.target_asset = new_target_asset.clone();
    order.interval = new_interval;
    order.dca_amount = new_dca_amount;
    order.max_hops = max_hops;
    order.max_spread = max_spread;
    order.start_purchase = start_purchase;

    if should_reset_purchase_time {
        order.last_purchase = 0;
    }

    if let Some(start_purchase) = order.start_purchase {
        if start_purchase < env.block.time.seconds() {
            return Err(ContractError::StartTimeInPast {});
        }
    }

    // check that assets are not duplicate
    if order.initial_asset.info == order.target_asset {
        return Err(ContractError::DuplicateAsset {});
    }

    // check that dca_amount is less than initial_asset.amount
    if order.dca_amount > order.initial_asset.amount {
        return Err(ContractError::DepositTooSmall {});
    }

    // check that initial_asset.amount is divisible by dca_amount
    if !order
        .initial_asset
        .amount
        .checked_rem(order.dca_amount)
        .map_err(|e| StdError::DivideByZero { source: e })?
        .is_zero()
    {
        return Err(ContractError::IndivisibleDeposit {});
    }

    state.dca_requests.save(deps.storage, id, &order)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "modify_dca_order"),
        attr("old_initial_asset", orig_asset.to_string()),
        attr("new_initial_asset", new_initial_asset.to_string()),
        attr("new_target_asset", new_target_asset.to_string()),
        attr("new_interval", new_interval.to_string()),
        attr("new_dca_amount", new_dca_amount),
    ]))
}
