use astroport::asset::{Asset, AssetInfo};
use astroport_dca::{ConfigOverride, DcaInfo};
use cosmwasm_std::{attr, DepsMut, Empty, Env, MessageInfo, Response, StdError, Uint128};

use crate::{
    error::ContractError,
    get_token_allowance::get_token_allowance,
    state::{DCA, DCA_ID, DCA_OWNER},
};

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
#[allow(clippy::too_many_arguments)]
pub fn create_dca_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    initial_asset: Asset,
    target_asset: AssetInfo,
    interval: u64,
    dca_amount: Uint128,
    start_at: Option<u64>,
    config_override: Option<ConfigOverride>,
) -> Result<Response, ContractError> {
    let id = DCA_ID.load(deps.storage)?;

    initial_asset.info.check(deps.api)?;
    target_asset.check(deps.api)?;

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
            if allowance < initial_asset.amount {
                return Err(ContractError::InvalidTokenDeposit {});
            }
        }
    }

    let now = env.block.time.seconds();
    let dca_info = DcaInfo {
        id,
        owner: info.sender,
        initial_asset,
        target_asset,
        interval,
        last_purchase: match start_at {
            // if start_at is in future -> calculate last_purchase to match start_at time
            Some(start_at) if start_at > now => start_at - interval,
            // else will default to start from now + interval
            _ => now,
        },
        dca_amount,
        config_override: config_override.unwrap_or_default(),
    };

    DCA_ID.save(deps.storage, &(id + 1))?;
    DCA.save(deps.storage, id, &dca_info)?;
    DCA_OWNER.save(deps.storage, (&dca_info.owner, id), &Empty {})?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "create_dca_order"),
        attr("id", id.to_string()),
        attr("initial_asset", dca_info.initial_asset.to_string()),
        attr("target_asset", dca_info.target_asset.to_string()),
        attr("interval", interval.to_string()),
        attr("dca_amount", dca_amount),
        attr("start_at", dca_info.last_purchase.to_string()),
        attr("config_override", dca_info.config_override.to_string()),
    ]))
}
