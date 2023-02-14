use crate::{
  error::ContractError,
  models::{Asset, ContractResult, RaffleStatus},
  state::{OWNER, RAFFLE, REFUND_STATUSES, WALLET_METADATA},
};
use cosmwasm_std::{attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_msg, build_send_msg},
};

pub fn claim_refund(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  let mut raffle = RAFFLE.load(deps.storage)?;
  let wallet_meta = WALLET_METADATA.load(deps.storage, info.sender.clone())?;

  // only canceled raffles can issue refunds
  if raffle.status != RaffleStatus::Canceled {
    return Err(ContractError::NotAuthorized {});
  }

  // disallow double refundsand indicate refund has occured
  REFUND_STATUSES.update(
    deps.storage,
    info.sender.clone(),
    |maybe_has_claimed| -> ContractResult<bool> {
      if maybe_has_claimed.unwrap_or(false) {
        return Err(ContractError::AlreadyClaimed {});
      }
      Ok(true)
    },
  )?;

  let mut cw20_transfer_msgs: Vec<SubMsg> = vec![];
  let mut native_transfer_msgs: Vec<CosmosMsg> = vec![];

  // send raffled token assets balance back to raffle owner
  let owner = OWNER.load(deps.storage)?;
  for asset in raffle.assets.iter() {
    if let Asset::Token { token, amount } = &asset {
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

  let refund_amount = Uint128::from(wallet_meta.ticket_count) * raffle.price.amount;

  // send total amount spend by buyer back
  match &raffle.price.token {
    Token::Native { denom } => {
      native_transfer_msgs.push(build_send_msg(&info.sender, denom, refund_amount)?)
    },
    Token::Cw20 { address: cw20_addr } => cw20_transfer_msgs.push(build_cw20_transfer_msg(
      &info.sender,
      cw20_addr,
      refund_amount,
    )?),
  }

  raffle.status = RaffleStatus::Canceled;

  RAFFLE.save(deps.storage, &raffle)?;

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "cancel")])
      .add_messages(native_transfer_msgs)
      .add_submessages(cw20_transfer_msgs),
  )
}
