use astroport::asset::AssetInfo;
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::{error::ContractError, get_token_allowance::get_token_allowance, state::DCA};

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
    id: u64,
    initial_amount: Option<Uint128>,
    interval: Option<u64>,
    dca_amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut attrs = vec![attr("action", "modify_dca_order")];
    let mut order = DCA.load(deps.storage, id)?;

    (order.owner == info.sender)
        .then(|| ())
        .ok_or(ContractError::Unauthorized {})?;

    if let Some(initial_amount) = initial_amount {
        // check if new amount is greater than old amount
        (initial_amount > order.initial_asset.amount)
            .then(|| ())
            .ok_or(ContractError::InvalidNewInitialAmount {})?;

        match &order.initial_asset.info {
            AssetInfo::NativeToken { denom } => {
                match info.funds.iter().find(|e| &e.denom == denom) {
                    Some(amt) => (amt.amount >= (initial_amount - order.initial_asset.amount))
                        .then(|| ())
                        .ok_or(ContractError::InvalidNativeTokenDeposit {}),
                    None => Err(ContractError::InvalidNativeTokenDeposit {}),
                }?;
            }
            AssetInfo::Token { contract_addr } => {
                let allowance =
                    get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
                if allowance < initial_amount {
                    return Err(ContractError::InvalidTokenDeposit {});
                }
            }
        }

        order.initial_asset.amount = initial_amount;

        // check that initial_asset.amount is divisible by dca_amount
        if !order
            .initial_asset
            .amount
            .checked_rem(order.dca_amount)
            .map_err(StdError::divide_by_zero)?
            .is_zero()
        {
            return Err(ContractError::IndivisibleDeposit {});
        }

        attrs.push(attr("new_initial_asset_amount", initial_amount));
    }

    if let Some(interval) = interval {
        order.interval = interval;
        attrs.push(attr("new_interval", interval.to_string()));
    }

    if let Some(dca_amount) = dca_amount {
        if dca_amount > order.initial_asset.amount {
            return Err(ContractError::DepositTooSmall {});
        }

        // check that initial_asset.amount is divisible by dca_amount
        if !order
            .initial_asset
            .amount
            .checked_rem(dca_amount)
            .map_err(StdError::divide_by_zero)?
            .is_zero()
        {
            return Err(ContractError::IndivisibleDeposit {});
        }

        order.dca_amount = dca_amount;
        attrs.push(attr("new_dca_amount", dca_amount));
    }

    DCA.save(deps.storage, id, &order)?;

    Ok(Response::new().add_attributes(attrs))
}
