use astroport::asset::{Asset, AssetInfo};
use astroport_dca::DcaInfo;
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::{error::ContractError, get_token_allowance::get_token_allowance, state::USER_DCA};

/// ## Description
/// Creates a new DCA order for a user where the `target_asset` will be purchased with `dca_amount`
/// of token `initial_asset` every `interval`.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to create their order, containing the
/// [`AssetInfo::NativeToken`] if the `initial_asset` is a native token.
///
/// * `initial_asset` - The [`Asset`] that is being spent to purchase DCA orders. If the asset is a
/// Token (non-native), the contact will need to have the allowance for the DCA contract set to the
/// `initial_asset.amount`.
///
/// * `target_asset` - The [`AssetInfo`] that is being purchased with `initial_asset`.
///
/// * `interval` - The time in seconds between DCA purchases.
///
/// * `dca_amount` - A [`Uint128`] representing the amount of `initial_asset` to spend each DCA
/// purchase.
pub fn create_dca_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    initial_asset: Asset,
    target_asset: AssetInfo,
    interval: u64,
    dca_amount: Uint128,
) -> Result<Response, ContractError> {
    // check that user has not previously created dca strategy with this initial_asset
    let mut orders = USER_DCA
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    if orders
        .iter()
        .any(|order| order.initial_asset.info == initial_asset.info)
    {
        return Err(ContractError::AlreadyDeposited {});
    }

    // check that assets are not duplicate
    if initial_asset.info == target_asset {
        return Err(ContractError::DuplicateAsset {});
    }

    // check that dca_amount is less than initial_asset.amount
    if dca_amount > initial_asset.amount {
        return Err(ContractError::DepositTooSmall {});
    }

    // check that initial_asset.amount is divisible by dca_amount
    if !initial_asset
        .amount
        .checked_rem(dca_amount)
        .map_err(|e| StdError::DivideByZero { source: e })?
        .is_zero()
    {
        return Err(ContractError::IndivisibleDeposit {});
    }

    // check that user has sent the valid tokens to the contract
    // if native token, they should have included it in the message
    // otherwise, if cw20 token, they should have provided the correct allowance
    match &initial_asset.info {
        AssetInfo::NativeToken { .. } => initial_asset.assert_sent_native_token_balance(&info)?,
        AssetInfo::Token { contract_addr } => {
            let allowance = get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
            if allowance != initial_asset.amount {
                return Err(ContractError::InvalidTokenDeposit {});
            }
        }
    }

    // store dca order
    orders.push(DcaInfo {
        initial_asset: initial_asset.clone(),
        target_asset: target_asset.clone(),
        interval,
        last_purchase: 0,
        dca_amount,
    });

    USER_DCA.save(deps.storage, &info.sender, &orders)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "create_dca_order"),
        attr("initial_asset", initial_asset.to_string()),
        attr("target_asset", target_asset.to_string()),
        attr("interval", interval.to_string()),
        attr("dca_amount", dca_amount),
    ]))
}
