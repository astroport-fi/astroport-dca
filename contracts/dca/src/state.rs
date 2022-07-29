use astroport::asset::AssetInfo;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport_dca::{DcaInfo, UserConfig};

/// Stores the main dca module parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_hops: u32,
    /// The maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_spread: Decimal,
    /// The fee a user must pay per hop performed in a DCA purchase
    pub per_hop_fee: Uint128,
    /// The whitelisted tokens that can be used in a DCA purchase route
    pub whitelisted_tokens: Vec<AssetInfo>,
    /// The address of the Astroport factory contract
    pub factory_addr: Addr,
    /// The address of the Astroport router contract
    pub router_addr: Addr,
}

impl Config {
    pub fn is_whitelisted_asset(&self, asset: &AssetInfo) -> bool {
        self.whitelisted_tokens.contains(asset)
    }
}

/// The contract configuration
pub const CONFIG: Item<Config> = Item::new("config");
/// The configuration set by each user
pub const USER_CONFIG: Map<&Addr, UserConfig> = Map::new("user_config");
/// The DCA orders for a user
pub const USER_DCA: Map<&Addr, Vec<DcaInfo>> = Map::new("user_dca");
