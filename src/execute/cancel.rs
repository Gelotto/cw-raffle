use crate::{
  error::ContractError,
  models::{ContractResult, RaffleAsset, RaffleStatus, RAFFLE_STAGE_CANCELED},
  state::{is_owner, repository, IX_U64_STATUS, RAFFLE, RAFFLE_OWNER},
};
use cosmwasm_std::{attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_msg, build_send_msg},
};

pub fn cancel(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  if !is_owner(deps.storage, &info.sender)? {
    return Err(ContractError::NotAuthorized {});
  }
  let mut raffle = RAFFLE.load(deps.storage)?;

  // prevent raffle from being double-ended
  if raffle.status != RaffleStatus::Active {
    return Err(ContractError::NotAuthorized {});
  }

  let mut cw20_transfer_msgs: Vec<SubMsg> = vec![];
  let mut native_transfer_msgs: Vec<CosmosMsg> = vec![];

  // send contract balance back to raffle owner
  // build msgs to transfer auto-transferable assets
  let owner = RAFFLE_OWNER.load(deps.storage)?;
  for asset in raffle.assets.iter() {
    if let RaffleAsset::Token { token, amount, .. } = &asset {
      match token {
        Token::Native { denom } => {
          native_transfer_msgs.push(build_send_msg(&owner, denom, *amount)?)
        },
        Token::Cw20 { address: cw20_addr } => {
          cw20_transfer_msgs.push(build_cw20_transfer_msg(&owner, cw20_addr, *amount)?)
        },
      }
    }
  }

  raffle.status = RaffleStatus::Canceled;

  RAFFLE.save(deps.storage, &raffle)?;

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "cancel")])
      .add_message(
        repository(deps.storage)?
          .update()
          .set_u64(IX_U64_STATUS, RAFFLE_STAGE_CANCELED as u64)
          .build_msg()?,
      )
      .add_messages(native_transfer_msgs)
      .add_submessages(cw20_transfer_msgs),
  )
}
