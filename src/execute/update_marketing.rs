use crate::{
  error::ContractError,
  models::RaffleMarketingInfo,
  state::{is_owner, MARKETING_INFO},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

pub fn update_marketing(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  updated_marketing_info: &RaffleMarketingInfo,
) -> Result<Response, ContractError> {
  if !is_owner(deps.storage, &info.sender)? {
    return Err(ContractError::NotAuthorized {});
  }
  MARKETING_INFO.save(deps.storage, updated_marketing_info)?;
  Ok(Response::new().add_attributes(vec![attr("action", "update_marketing")]))
}
