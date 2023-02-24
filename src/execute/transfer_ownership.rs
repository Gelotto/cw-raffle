use crate::{
  error::ContractError,
  state::{is_owner, repository, RAFFLE_OWNER},
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response};

pub fn transfer_ownership(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  new_owner: &Addr,
) -> Result<Response, ContractError> {
  if !is_owner(deps.storage, &info.sender)? {
    return Err(ContractError::NotAuthorized {});
  }
  RAFFLE_OWNER.save(deps.storage, new_owner)?;
  Ok(
    Response::new()
      .add_attributes(vec![
        attr("action", "transfer_ownership"),
        attr("new_owner", new_owner.to_string()),
      ])
      .add_message(
        repository(deps.storage)?
          .update()
          .untag_address(&info.sender, vec!["owner"])
          .tag_address(&new_owner, vec!["owner"])
          .build_msg()?,
      ),
  )
}
