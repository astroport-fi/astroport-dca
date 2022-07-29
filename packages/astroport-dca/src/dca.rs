use astroport::asset::{Asset, AssetInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;

/// Describes information about a DCA order
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DcaInfo {
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
}
