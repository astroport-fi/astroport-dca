use core::fmt;

use astroport::asset::{Asset, AssetInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};

/// Describes information about a DCA order
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DcaInfo {
    /// Unique id of this DCA purchases
    pub id: u64,
    /// Owner of this DCA purchases
    pub owner: Addr,
    /// The starting asset deposited by the user, with the amount representing the users deposited
    /// amount of the token
    pub initial_asset: Asset,
    /// The asset being purchased in DCA purchases
    pub target_asset: AssetInfo,
    /// The interval in seconds between DCA purchases
    pub interval: u64,
    /// The last time the `target_asset` was purchased
    pub last_purchase: u64,
    /// The amount of `initial_asset` to spend each DCA purchase
    pub dca_amount: Uint128,
    /// Config to override user's `max_hops` and `max_spread`, if this is [None], will use global user config instead
    pub config_override: ConfigOverride,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, JsonSchema, Default)]
pub struct ConfigOverride {
    /// Maximum hops to perform, if this is [None], will use global user config instead
    pub max_hops: Option<u32>,
    /// Maximum spread to perform, if this is [None], will use global user config instead
    pub max_spread: Option<Decimal>,
}

impl fmt::Display for ConfigOverride {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}/{:?}", self.max_hops, self.max_spread)
    }
}

//#[test]
//fn test() {
//let g = ConfigOverride {
//max_spread: None,
//max_hops: Some(3),
//};

//println!("{}", g);
//}
