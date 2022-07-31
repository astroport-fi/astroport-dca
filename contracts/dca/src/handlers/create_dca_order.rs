use astroport::asset::{Asset, AssetInfo};
use astroport_dca::dca::DcaInfo;
use cosmwasm_std::{attr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::{error::ContractError, get_token_allowance::get_token_allowance, state::State};

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
    start_purchase: Option<u64>,
    max_hops: Option<u32>,
    max_spread: Option<Decimal>,
) -> Result<Response, ContractError> {
    if let Some(start_purchase) = start_purchase {
        if start_purchase < env.block.time.seconds() {
            return Err(ContractError::StartTimeInPast {});
        }
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

    let state = State::default();

    // check that user has sent the valid tokens to the contract
    // if native token, they should have included it in the message
    // otherwise, if cw20 token, they should have provided the correct allowance
    match &initial_asset.info {
        AssetInfo::NativeToken { .. } => initial_asset.assert_sent_native_token_balance(&info)?,
        AssetInfo::Token { contract_addr } => {
            // should aggregate all orders based on the initial_asset
            // the allowance should be bigger than the total sum of existing_dcas + initial_asset.amount

            // let existing_dcas: Vec<DcaInfo> = state
            //     .dca_requests
            //     .idx
            //     .user_asset
            //     .prefix((info.sender.to_string(), initial_asset.info.to_string()))
            //     .range(deps.storage, None, None, Order::Ascending)
            //     .map(|item| {
            //         let (_, res) = item.unwrap();
            //         res.into()
            //     })
            //     .collect();

            let allowance = get_token_allowance(&deps.as_ref(), &env, &info.sender, contract_addr)?;
            if allowance < initial_asset.amount {
                return Err(ContractError::InvalidTokenDeposit {});
            }
        }
    }

    let order_id = state.get_next_order_id(deps.storage)?;

    state.dca_requests.save(
        deps.storage,
        order_id,
        &DcaInfo {
            id: order_id,
            initial_asset: initial_asset.clone(),
            target_asset: target_asset.clone(),
            interval,
            last_purchase: 0,
            dca_amount,
            max_hops,
            start_purchase,
            max_spread,
            user: info.sender.clone(),
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "create_dca_order"),
        attr("id", order_id.to_string()),
        attr("initial_asset", initial_asset.to_string()),
        attr("target_asset", target_asset.to_string()),
        attr("interval", interval.to_string()),
        attr("dca_amount", dca_amount),
        attr("max_hops", max_hops.unwrap_or_default().to_string()),
        attr(
            "start_purchase",
            start_purchase.unwrap_or_default().to_string(),
        ),
        attr("max_spread", max_spread.unwrap_or_default().to_string()),
        attr("user", info.sender.to_string()),
    ]))
}
