use astroport::asset::Asset;
use astroport::asset::AssetInfo;

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport_dca::dca::DcaInfo;

/// Stores the main dca module parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /*
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_hops: u32,
    /// The maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_spread: Decimal,
    /// The fee a user must pay per hop performed in a DCA purchase
    pub per_hop_fee: Uint128,
    /// The whitelisted tokens that can be used in a DCA purchase route

    */
    // the list of tokens which are allowed in the DCA contracts.
    pub whitelist_tokens: WhitelistTokens,

    /// The address of the Astroport factory contract
    pub factory_addr: Addr,
    /// The address of the Astroport router contract
    pub router_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistTokens {
    // Token which can be by the user to deposit in the DCA contract
    pub deposit: Vec<AssetInfo>,

    // Token which can be used by the user to reward a bot for
    // executing DCA orders.
    pub tip: Vec<AssetInfo>,
}

impl WhitelistTokens {
    pub fn is_deposit_asset(&self, asset: &AssetInfo) -> bool {
        self.deposit.contains(asset)
    }

    pub fn is_tip_asset(&self, asset: &AssetInfo) -> bool {
        self.tip.contains(asset)
    }
}

/// Stores the users custom configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserConfig {
    /// An override for the maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing
    pub max_hops: Option<u32>,
    /// An override for the maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing
    pub max_spread: Option<Decimal>,
    /// The amount of uusd the user has deposited for their tips when performing DCA purchases
    pub tip_balance: Uint128,
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            max_hops: None,
            max_spread: None,
            tip_balance: Uint128::zero(),
        }
    }
}

/// The contract configuration
pub const CONFIG: Item<Config> = Item::new("config");
/// The DCA orders for a user
pub const USER_DCA: Map<&Addr, Vec<DcaInfo>> = Map::new("user_dca");
