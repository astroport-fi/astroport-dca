use astroport::{
    asset::{Asset, AssetInfo},
    router::SwapOperation,
};
use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::DcaInfo;

/// Describes the parameters used for creating a contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if
    /// the user does not specify a custom max hop amount
    pub max_hops: u32,
    /// The fee a user must pay per hop performed in a DCA purchase
    pub per_hop_fee: Uint128,
    /// The whitelisted tokens that can be used in a DCA hop route
    pub whitelisted_tokens: Vec<AssetInfo>,
    /// The maximum amount of spread
    pub max_spread: String,
    /// The address of the Astroport factory contract
    pub factory_addr: String,
    /// The address of the Astroport router contract
    pub router_addr: String,
}

/// This structure describes the execute messages available in the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Add uusd top-up for bots to perform DCA requests
    AddBotTip {},
    /// Cancels a DCA order, returning any native asset back to the user
    CancelDcaOrder { initial_asset: AssetInfo },
    /// Creates a new DCA order where `dca_amount` of token `initial_asset` will purchase
    /// `target_asset` every `interval`
    ///
    /// If `initial_asset` is a Cw20 token, the user needs to have increased the allowance prior to
    /// calling this execution
    CreateDcaOrder {
        initial_asset: Asset,
        target_asset: AssetInfo,
        interval: u64,
        dca_amount: Uint128,
    },
    /// Modifies an existing DCA order, allowing the user to change certain parameters
    ModifyDcaOrder {
        old_initial_asset: AssetInfo,
        new_initial_asset: Asset,
        new_target_asset: AssetInfo,
        new_interval: u64,
        new_dca_amount: Uint128,
        should_reset_purchase_time: bool,
    },
    /// Performs a DCA purchase for a specified user given a hop route
    PerformDcaPurchase {
        user: String,
        hops: Vec<SwapOperation>,
    },
    /// Updates the configuration of the contract
    UpdateConfig {
        /// The new maximum amount of hops to perform from `initial_asset` to `target_asset` when
        /// performing DCA purchases if the user does not specify a custom max hop amount
        max_hops: Option<u32>,
        /// The new fee a user must pay per hop performed in a DCA purchase
        per_hop_fee: Option<Uint128>,
        /// The new whitelisted tokens that can be used in a DCA hop route
        whitelisted_tokens: Option<Vec<AssetInfo>>,
        /// The new maximum spread for DCA purchases
        max_spread: Option<Decimal>,
    },
    /// Update the configuration for a user
    UpdateUserConfig {
        /// The maximum amount of hops per swap
        max_hops: Option<u32>,
        /// The maximum spread per token when performing DCA purchases
        max_spread: Option<Decimal>,
    },
    /// Withdraws a users bot tip from the contract.
    Withdraw { tip: Uint128 },
}

/// This structure describes the query messages available in the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns information about the users current active DCA orders in a [`Vec<DcaInfo>`] object.
    UserDcaOrders { user: String },
    /// Returns information about the contract configuration in a [`Config`] object.
    Config {},
    /// Returns the users current configuration as a [`UserConfig`] object.
    UserConfig { user: String },
}

/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

/// Describes information for a UserDcaOrders query
///
/// Contains both the user DCA order and the cw20 token allowance, or, if the initial asset is a
/// native token, the balance.
///
/// This is useful for bots and front-end to distinguish between a users token allowance (which may
/// have changed) for the DCA contract, and the created DCA order size.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DcaQueryInfo {
    pub token_allowance: Uint128,
    pub info: DcaInfo,
}
