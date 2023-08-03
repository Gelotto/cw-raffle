use crate::{
  error::ContractError,
  models::{ContractResult, RaffleStatus},
  state::{RAFFLE, REFUND_STATUSES, WALLET_METADATA},
};
use cosmwasm_std::{attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_submsg, build_send_msg},
};

pub fn claim_refund(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  let raffle = RAFFLE.load(deps.storage)?;

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

  // refund ticket proceeds
  if let Some(mut wallet_meta) = WALLET_METADATA.may_load(deps.storage, info.sender.clone())? {
    if wallet_meta.has_claimed_refund {
      return Err(ContractError::AlreadyClaimed {});
    }

    wallet_meta.has_claimed_refund = true;
    WALLET_METADATA.save(deps.storage, info.sender.clone(), &wallet_meta)?;

    // get total cost for all buyer's tickets
    let refund_amount = Uint128::from(wallet_meta.ticket_count) * raffle.price.amount;
    match &raffle.price.token {
      Token::Native { denom } => {
        native_transfer_msgs.push(build_send_msg(&info.sender, denom, refund_amount)?)
      },
      Token::Cw20 { address: cw20_addr } => cw20_transfer_msgs.push(build_cw20_transfer_submsg(
        &info.sender,
        cw20_addr,
        refund_amount,
      )?),
    }
  }

  RAFFLE.save(deps.storage, &raffle)?;

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "cancel")])
      .add_messages(native_transfer_msgs)
      .add_submessages(cw20_transfer_msgs),
  )
}
