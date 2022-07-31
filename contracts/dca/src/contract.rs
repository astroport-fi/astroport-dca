use crate::error::ContractError;
use crate::handlers::{
    add_bot_tip, cancel_dca_order, create_dca_order, modify_dca_order, perform_dca_purchase,
    receive, update_config, withdraw,
};
use crate::queries::{
    get_config, get_dca_order, get_dca_orders, get_user_asset_dca_orders, get_user_dca_orders,
    get_user_tips,
};
use crate::state::{Config, State};

use astroport::asset::addr_validate_to_lower;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use astroport_dca::dca::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cw2::set_contract_version;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "astroport-dca";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in [`InstantiateMsg`].
///
/// Returns a [`Response`] with the specified attributes if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `_env` - The [`Env`] of the blockchain.
///
/// * `_info` - The [`MessageInfo`] from the contract instantiator.
///
/// * `msg` - A [`InstantiateMsg`] which contains the parameters for creating the contract.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // validate that factory_addr and router_addr is an address
    let factory_addr = addr_validate_to_lower(deps.api, &msg.factory_addr)?;
    let router_addr = addr_validate_to_lower(deps.api, &msg.router_addr)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State::default();

    let config = Config {
        max_hops: msg.max_hops,
        whitelisted_tokens: msg.whitelisted_tokens,
        max_spread: msg.max_spread,
        factory_addr,
        router_addr,
    };

    state.config.save(deps.storage, &config)?;
    state.dca_id.save(deps.storage, &0u64)?;
    state
        .whitelisted_tip_tokens
        .save(deps.storage, &msg.whitelisted_tip_tokens)?;

    Ok(Response::new())
}

/// ## Description
/// Used for contract migration. Returns a default object of type [`Response`].
/// ## Arguments
/// * `_deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `_env` - The [`Env`] of the blockchain.
///
/// * `_msg` - The [`MigrateMsg`] to migrate the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

/// ## Description
/// Exposes all the execute functions available in the contract.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `info` - A [`MessageInfo`] that contains the message information.
///
/// * `msg` - The [`ExecuteMsg`] to run.
///
/// ## Execution Messages
/// * **ExecuteMsg::AddBotTip { }** Adds a bot tip to fund DCA purchases.
///
/// * **ExecuteMsg::CancelDcaOrder { initial_asset }** Cancels an existing DCA order.
///
/// * **ExecuteMsg::CreateDcaOrder {
///         initial_asset,
///         target_asset,
///         interval,
///         dca_amount
///     }** Creates a new DCA order where `initial_asset` will purchase `target_asset`.
///
/// * **ExecuteMsg::ModifyDcaOrder {
///         old_initial_asset,
///         new_initial_asset,
///         new_target_asset,
///         new_interval,
///         new_dca_amount,
///         should_reset_purchase_time,
///     }** Modifies an existing DCA order, allowing the user to change certain parameters.
///
/// * **ExecuteMsg::PerformDcaPurchase { user, hops }** Performs a DCA purchase on behalf of a
/// specified user given a hop route.
///
/// * **ExecuteMsg::UpdateConfig {
///         max_hops,
///         per_hop_fee,
///         whitelisted_tokens,
///         max_spread
///     }** Updates the contract configuration with the specified input parameters.
///
/// * **ExecuteMsg::UpdateUserConfig {
///         max_hops,
///         max_spread,
///     }** Updates a users configuration with the new input parameters.
///
/// * **ExecuteMsg::Withdraw { tip }** Withdraws a bot tip from the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => receive(deps, env, info, cw20_msg),

        ExecuteMsg::UpdateConfig {
            max_hops,
            whitelisted_tokens,
            whitelisted_tip_tokens,
            max_spread,
        } => update_config(
            deps,
            info,
            max_hops,
            whitelisted_tokens,
            whitelisted_tip_tokens,
            max_spread,
        ),

        ExecuteMsg::CreateDcaOrder {
            initial_asset,
            target_asset,
            interval,
            dca_amount,
            start_purchase,
            max_hops,
            max_spread,
        } => create_dca_order(
            deps,
            env,
            info,
            initial_asset,
            target_asset,
            interval,
            dca_amount,
            start_purchase,
            max_hops,
            max_spread,
        ),
        ExecuteMsg::AddBotTip { asset } => {
            asset.assert_sent_native_token_balance(&info)?;
            add_bot_tip(deps, info.sender, asset)
        }
        ExecuteMsg::Withdraw { assets } => withdraw(deps, info, assets),
        ExecuteMsg::PerformDcaPurchase { id, hops } => {
            perform_dca_purchase(deps, env, info, id, hops)
        }
        ExecuteMsg::CancelDcaOrder { id } => cancel_dca_order(deps, info, id),
        ExecuteMsg::ModifyDcaOrder { parameters } => modify_dca_order(deps, env, info, parameters),
    }
}

/// ## Description
/// Exposes all the queries available in the contract.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `env` - The [`Env`] of the blockchain.
///
/// * `msg` - The [`QueryMsg`] to run.
///
/// ## Queries
/// * **QueryMsg::Config {}** Returns information about the configuration of the contract in a
/// [`Config`] object.
///
/// * **QueryMsg::UserConfig {}** Returns information about a specified users configuration set for
/// DCA purchases in a [`UserConfig`] object.
///
/// * **QueryMsg::UserDcaOrders {}** Returns information about a specified users current DCA orders
/// set in a [`Vec<DcaInfo>`] object.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&get_config(deps)?),

        QueryMsg::UserDcaOrders {
            user,
            start_after,
            limit,
        } => to_binary(&get_user_dca_orders(deps, env, user, start_after, limit)?),

        QueryMsg::UserAssetDcaOrders {
            user,
            start_after,
            limit,
            asset,
        } => to_binary(&get_user_asset_dca_orders(
            deps,
            env,
            user,
            asset,
            start_after,
            limit,
        )?),

        QueryMsg::UserTips { user } => to_binary(&get_user_tips(deps, env, user)?),

        QueryMsg::DcaOrder { id } => to_binary(&get_dca_order(deps, env, id)?),

        QueryMsg::DcaOrders { start_after, limit } => {
            to_binary(&get_dca_orders(deps, env, start_after, limit)?)
        }
    }
}
