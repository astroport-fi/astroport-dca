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

    #[error("Aggreagate assets (source, tip, gas) amount ('{aggr_amount}') is greater than the allowance '{allowance}' set by token '{token_addr}' ")]
    AllowanceCheckFail {
        token_addr: String,
        aggr_amount: String,
        allowance: String,
    },

    #[error("Unable to extraget the target amount from the reply object")]
    ParseReplySubMsgForPerformDcaPurhcase {},

    #[error("InvalidSwapOperations: '{msg}'")]
    InvalidSwapOperations { msg: String },

    #[error("Max_spread='{max_spread}' check fail. Got one swap_spread='{swap_spread}'")]
    MaxSpreadCheckFail {
        max_spread: String,
        swap_spread: String,
    },

    #[error("Invalid hop route through {token} due to token whitelist")]
    InvalidHopRoute { token: String },

    #[error("Invalid input. msg: '{msg}'")]
    InvalidInput { msg: String },

    #[error("The user does not have the specified DCA. msg: '{msg}'")]
    NonexistentDca { msg: String },

    #[error("Swap exceeds maximum of {hops} hops")]
    MaxHopsAssertion { hops: u32 },

    #[error("Tip balance is insufficient to pay performer")]
    InsufficientTipBalance {},

    #[error("The dca_order_id = '{id}' is already used!")]
    DCAUniqueContraintViolation { id: String },

    #[error("The hop route specified was empty")]
    EmptyHopRoute {},

    #[error("DCA purchase occurred too early")]
    PurchaseTooEarly {},

    #[error("There are too many DCA purchases in queue. Try later!")]
    TooManyPurchasesInQueue {},

    #[error("TMP_CONTRACT_TARGET_BALANCE is None")]
    TmpContractTargetBalance {},

    #[error("Hop route does not end up at target_asset")]
    TargetAssetAssertion {},

    #[error("Hop route does not start with the deposit asset")]
    StartAssetAssertion {},

    #[error("Asset balance is less than DCA purchase amount")]
    InsufficientBalance {},

    #[error("Initial asset and target asset are the same")]
    DuplicateAsset {},

    #[error("DCA amount is greater than deposited amount")]
    DepositTooSmall {},

    #[error("Initial asset deposited is not divisible by the DCA amount")]
    IndivisibleDeposit {},

    #[error("Unable to update the DCA balance. msg: '{msg}'")]
    BalanceUpdateError { msg: String },
}
