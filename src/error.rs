use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("NotActive")]
  NotActive {},

  #[error("NotSoldOut")]
  NotSoldOut {},

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("MissingFunds")]
  MissingFunds {},

  #[error("BelowTicketSalesThreshold")]
  BelowTicketSalesThreshold {},

  #[error("InsufficientTicketSupply")]
  InsufficientTicketSupply {},

  #[error("AlreadyClaimed")]
  AlreadyClaimed {},

  #[error("SalesPeriodOver")]
  SalesPeriodOver {},

  #[error("SoldOut")]
  SoldOut {},

  #[error("ValidationError")]
  ValidationError { reason: Option<String> },
}
