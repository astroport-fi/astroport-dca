use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stores the users custom configuration
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, JsonSchema, Default)]
pub struct UserConfig {
    /// An override for the maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing
    pub max_hops: Option<u32>,
    /// An override for the maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing
    pub max_spread: Option<Decimal>,
    /// The amount of uusd the user has deposited for their tips when performing DCA purchases
    pub tip_balance: Uint128,
}
