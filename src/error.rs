use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("MissingFunds")]
  MissingFunds {},

  #[error("AlreadyClaimed")]
  AlreadyClaimed {},

  #[error("SoldOut")]
  SoldOut {},

  #[error("ValidationError")]
  ValidationError {},
}
