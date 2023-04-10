use crate::{
  error::ContractError,
  models::RaffleMarketingInfo,
  state::{is_allowed, MARKETING_INFO},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

pub fn update(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  maybe_marketing: &Option<RaffleMarketingInfo>,
) -> Result<Response, ContractError> {
  if !is_allowed(&deps.as_ref(), &info.sender, "update")? {
    return Err(ContractError::NotAuthorized {});
  }
  if let Some(marketing) = maybe_marketing {
    MARKETING_INFO.save(deps.storage, marketing)?;
  }
  Ok(Response::new().add_attributes(vec![attr("action", "update")]))
}
