use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

/// ## Description
/// This enum describes DCA contract errors
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Event of zero transfer")]
    InvalidZeroAmount {},

    #[error("Bot tip token {token} is not whitelisted")]
    InvalidBotTipToken { token: String },

    #[error("DCA amount is not equal to allowance set by token")]
    InvalidTokenDeposit {},

    #[error("Invalid hop route through {token} due to token whitelist")]
    InvalidHopRoute { token: String },

    #[error("Provided DCA Id is not known")]
    NonExistentDca {},

    #[error("Provided token {token} does not exist in the tip jars")]
    NonExistentTipJar { token: String },

    #[error("Swap exceeds maximum of {hops} hops")]
    MaxHopsAssertion { hops: u32 },

    #[error("Tip balance not deposited to pay performer")]
    NoTipBalance {},

    #[error("Tip balance is insufficient to pay performer")]
    InsufficientTipBalance {},

    #[error("The hop route specified was empty")]
    EmptyHopRoute {},

    #[error("DCA purchase occurred too early")]
    PurchaseTooEarly {},

    #[error("Hop route does not end up at target_asset")]
    TargetAssetAssertion {},

    #[error("Asset balance is less than DCA purchase amount")]
    InsufficientBalance {},

    #[error("Initial asset and target asset are the same")]
    DuplicateAsset {},

    #[error("DCA amount is greater than deposited amount")]
    DepositTooSmall {},

    #[error("Initial asset deposited is not divisible by the DCA amount")]
    IndivisibleDeposit {},

    #[error("Start time must be unset or in the future")]
    StartTimeInPast {},

    #[error("NativeSwap is not supported")]
    NativeSwapNotSupported {},
}
