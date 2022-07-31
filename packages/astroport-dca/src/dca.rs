use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport::{
    asset::{Asset, AssetInfo},
    router::SwapOperation,
};

use cosmwasm_std::{Addr, Decimal, Uint128};

/// Describes information about a DCA order
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DcaInfo {
    /// DCA order Id
    pub id: u64,
    /// The starting asset deposited by the user, with the amount representing the users deposited
    /// amount of the token
    pub initial_asset: Asset,
    /// The asset being purchased in DCA purchases
    pub target_asset: AssetInfo,
    /// The interval in seconds between DCA purchases
    pub interval: u64,
    /// The last time the `target_asset` was purchased
    pub last_purchase: u64,
    /// The start time when `target_asset` can first be purchased
    pub start_purchase: Option<u64>,
    /// The amount of `initial_asset` to spend each DCA purchase
    pub dca_amount: Uint128,
    /// An override for the maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing
    pub max_hops: Option<u32>,
    /// An override for the maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing
    pub max_spread: Option<Decimal>,
    /// User of the DCA order
    pub user: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TipAssetInfo {
    /// The asset that can be used for the tip
    pub info: AssetInfo,
    /// The fee a user must pay per hop performed in a DCA purchase, using the asset info
    pub per_hop_fee: Uint128,
}
/// Describes the parameters used for creating a contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if
    /// the user does not specify a custom max hop amount
    pub max_hops: u32,
    /// The whitelisted tokens that can be used in a DCA hop route
    pub whitelisted_tokens: Vec<AssetInfo>,
    /// The whitelisted tokens that can be used for the bot tips
    pub whitelisted_tip_tokens: Vec<TipAssetInfo>,
    /// The maximum amount of spread
    pub max_spread: Decimal,
    /// The address of the Astroport factory contract
    pub factory_addr: String,
    /// The address of the Astroport router contract
    pub router_addr: String,
}

/// Stores a modified dca order new parameters
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ModifyDcaOrderParameters {
    /// DCA order Id
    pub id: u64,
    /// The new [`Asset`] that is being spent to create DCA orders.
    pub new_initial_asset: Asset,
    /// The [`AssetInfo`] that is being purchased with `new_initial_asset`.
    pub new_target_asset: AssetInfo,
    /// The time in seconds between DCA purchases.
    pub new_interval: u64,
    /// a [`Uint128`] amount of `new_initial_asset` to spend each DCA purchase.
    pub new_dca_amount: Uint128,
    /// A bool flag that determines if the order's last purchase time should be reset.
    pub should_reset_purchase_time: bool,
    /// The start time when `target_asset` can first be purchased
    pub start_purchase: Option<u64>,
    /// An override for the maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing
    pub max_hops: Option<u32>,
    /// An override for the maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing
    pub max_spread: Option<Decimal>,
}

/// This structure describes the execute messages available in the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Implements the Cw20 receiver interface
    Receive(Cw20ReceiveMsg),

    /// Add uusd top-up for bots to perform DCA requests
    AddBotTip { asset: Asset },
    /// Cancels a DCA order, returning any native asset back to the user
    CancelDcaOrder { id: u64 },

    /// Creates a new DCA order where `dca_amount` of token `initial_asset` will purchase
    /// `target_asset` every `interval`
    ///
    /// If `initial_asset` is a Cw20 token, the user needs to have increased the allowance prior to
    /// calling this execution
    CreateDcaOrder {
        /// asset that should be used for investment
        initial_asset: Asset,
        /// asset in which the initial_asset should be converted into
        target_asset: AssetInfo,
        /// interval in s to allow execution of the DCA
        interval: u64,
        /// amount of initial_asset that should be converted per execution. Initial asset needs to be devisable by dca_amount
        dca_amount: Uint128,
        /// if set, specifies, when the first execution is allowed to happen, otherwise immediately
        start_purchase: Option<u64>,
        /// specifies, how many hops the conversion is allowed to execute. If not set, the contract default is used.
        max_hops: Option<u32>,
        /// specifies, how high the spread is allowed to execute. If not set, the contract default is used.
        max_spread: Option<Decimal>,
    },
    /// Modifies an existing DCA order, allowing the user to change certain parameters
    ModifyDcaOrder {
        parameters: ModifyDcaOrderParameters,
    },
    /// Performs a DCA purchase for a specified id given a hop route
    PerformDcaPurchase {
        // Id of the Dca order
        id: u64,
        /// Hops that are being executed
        hops: Vec<SwapOperation>,
    },

    /// Updates the configuration of the contract
    UpdateConfig {
        /// The new maximum amount of hops to perform from `initial_asset` to `target_asset` when
        /// performing DCA purchases if the user does not specify a custom max hop amount
        max_hops: Option<u32>,
        /// The new whitelisted tokens that can be used in a DCA hop route
        whitelisted_tokens: Option<Vec<AssetInfo>>,
        /// The whitelisted tokens that can be used for the bot tips
        whitelisted_tip_tokens: Option<Vec<TipAssetInfo>>,
        /// The new maximum spread for DCA purchases
        max_spread: Option<Decimal>,
    },
    /// Withdraws a users bot tip from the contract.
    Withdraw { assets: Option<Vec<Asset>> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    /// used to add cw20 bot tips
    AddBotTip {},
}

/// This structure describes the query messages available in the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns information about the users current active DCA orders in a [`Vec<DcaInfo>`] object.
    UserDcaOrders {
        /// orders for the user
        user: String,
        /// pagination of orders
        start_after: Option<u64>,
        /// pagination of orders
        limit: Option<u32>,
    },
    /// Returns [`Vec<Asset>`]
    UserTips {
        // tips of the user
        user: String,
    },
    /// Load all orders for a user and specific asset Returns [`Vec<DcaQueryInfo>`]
    UserAssetDcaOrders {
        /// orders for the user
        user: String,
        /// Asset that should be returned
        asset: AssetInfo,
        /// pagination of orders
        start_after: Option<u64>,
        /// pagination of orders
        limit: Option<u32>,
    },
    /// Load a single order by id. Returns [`DcaInfo`]
    DcaOrder {
        /// Id of the DCA order to be returned
        id: u64,
    },
    /// Load all orders [`Vec<DcaInfo>`]
    DcaOrders {
        // pagination of orders
        start_after: Option<u64>,
        /// pagination of orders
        limit: Option<u32>,
    },
    /// Returns information about the contract configuration in a [`ConfigResponse`] object.
    Config {},
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
#[serde(rename_all = "snake_case")]
pub struct DcaQueryInfo {
    pub token_allowance: Uint128,
    pub info: DcaInfo,
}

/// Stores the main dca module parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_hops: u32,
    /// The maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_spread: Decimal,
    /// The whitelisted tokens that can be used in a DCA purchase route
    pub whitelisted_tokens: Vec<AssetInfo>,
    /// The whitelisted tokens that can be used in a DCA purchase route
    pub whitelisted_tip_tokens: Vec<TipAssetInfo>,
    /// The address of the Astroport factory contract
    pub factory_addr: Addr,
    /// The address of the Astroport router contract
    pub router_addr: Addr,
}
