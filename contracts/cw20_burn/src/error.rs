use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid subdenom: {subdenom:?}")]
    InvalidSubdenom { subdenom: String },

    #[error("Invalid denom: {denom:?} {message:?}")]
    InvalidDenom { denom: String, message: String },

    #[error("denom does not exist: {denom:?}")]
    DenomDoesNotExist { denom: String },

    #[error("address is not supported yet, was: {address:?}")]
    BurnFromAddressNotSupported { address: String },

    #[error("amount was zero, must be positive")]
    ZeroAmount {},

    #[error("this is not an invalid cw20 message")]
    InvalidCW20Message {},

    #[error("invalid cw20 address, does not match with state.")]
    InvalidCW20Address {},
}
