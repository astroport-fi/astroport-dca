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

    #[error("Provided spread amount exceeds allowed limit")]
    AllowedSpreadAssertion {},

    #[error("Operation exceeds max spread limit")]
    MaxSpreadAssertion {},

    #[error("Token has already been used to DCA")]
    AlreadyDeposited {},

    #[error("DCA amount is not equal to fund sent")]
    InvalidNativeTokenDeposit {},

    #[error("DCA amount is not equal to allowance set by token")]
    InvalidTokenDeposit {},

    #[error("Invalid hop route through {token} due to token whitelist")]
    InvalidHopRoute { token: String },

    #[error("The user does not have the specified initial_asset to DCA")]
    NonexistentDca {},

    #[error("Swap exceeds maximum of {hops} hops")]
    MaxHopsAssertion { hops: u32 },

    #[error("Tip balance is insufficient to pay performer")]
    InsufficientTipBalance {},

    #[error("The hop route specified was empty")]
    EmptyHopRoute {},

    #[error("DCA purchase occurred too early")]
    PurchaseTooEarly {},

    #[error("Hop route does not end up at target_asset")]
    TargetAssetAssertion {},

    #[error("Hop route does not start at initial_asset")]
    InitialAssetAssertion {},

    #[error("Asset balance is less than DCA purchase amount")]
    InsufficientBalance {},

    #[error("Initial asset and target asset are the same")]
    DuplicateAsset {},

    #[error("DCA amount is greater than deposited amount")]
    DepositTooSmall {},

    #[error("Initial asset deposited is not divisible by the DCA amount")]
    IndivisibleDeposit {},

    #[error("Native swap is not allowed")]
    InvalidNativeSwap {},

    #[error("New initial amount must be greater than old initial amount")]
    InvalidNewInitialAmount {},

    #[error("Tip amount must be greater than zero")]
    InvalidTipAmount {},

    #[error("Tip asset info must be whitelisted")]
    InvalidTipAssetInfo {},

    #[error("Tip balance is insufficient to withdraw")]
    InsufficientTipWithdrawBalance {},
}
