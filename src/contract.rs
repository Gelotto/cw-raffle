#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::execute;
use crate::models::ContractResult;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query;
use crate::state;
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-contract-template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
  state::initialize(deps, &env, &info, &msg)?;
  Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::TransferOwnership { new_owner } => {
      execute::transfer_ownership(deps, env, info, &new_owner)
    },
    ExecuteMsg::BuyTickets {
      count,
      message,
      is_visible,
    } => execute::buy_tickets(deps, env, info, count, message, is_visible),
    ExecuteMsg::ChooseWinner {} => execute::choose_winner(deps, env, info),
    ExecuteMsg::Cancel {} => execute::cancel(deps, env, info),
    ExecuteMsg::ClaimRefund {} => execute::claim_refund(deps, env, info),
  }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
  deps: Deps,
  _env: Env,
  msg: QueryMsg,
) -> ContractResult<Binary> {
  let result = match msg {
    QueryMsg::Select { fields } => to_binary(&query::select(deps, fields)?),
    QueryMsg::RefundStatus { claimant } => to_binary(&query::refund_status(deps, &claimant)?),
  }?;
  Ok(result)
}
