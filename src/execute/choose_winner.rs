use crate::{
  error::ContractError,
  models::{ContractResult, RaffleAsset, RaffleStatus},
  selection::draw_winner,
  state::{is_owner, RAFFLE, ROYALTIES},
};
use cosmwasm_std::{attr, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_msg, build_send_msg},
};

// addresses for gelotto taxes:
pub const GELOTTO_ADDR: &str = "juno1jume25ttjlcaqqjzjjqx9humvze3vcc8z87szj";
pub const GELOTTO_ANNUAL_PRIZE_ADDR: &str = "juno1fxu5as8z5qxdulujzph3rm6c39r8427mjnx99r";
pub const GELOTTO_NFT_1_REWARDS_ADDR: &str = "juno1tlyqv2ss4p9zelllxm39hq5g6zw384mvvym6tp";

// percentages for tax allocations:
pub const GELOTTO_PCT: u8 = 2;
pub const GELOTTO_NFT_1_REWARDS_PCT: u8 = 3;
pub const GELOTTO_ANNUAL_PRIZE_PCT: u8 = 5;

pub fn choose_winner(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  if !is_owner(deps.storage, &info.sender)? {
    return Err(ContractError::NotAuthorized {});
  }

  let mut raffle = RAFFLE.load(deps.storage)?;

  if raffle.tickets_sold == 0 {
    return Err(ContractError::AlreadyClaimed {});
  }

  // prevent raffle from being double-ended
  if raffle.status != RaffleStatus::Active || raffle.winner_address.is_some() {
    return Err(ContractError::NotActive {});
  }

  // ensure the game status can transition from active if there's a timer on
  // this raffle but time hasn't expired, only continue if the raffle's tickets
  // are completely sold out. otherwise, make them wait.
  if let Some(sales_end_at) = raffle.ticket_sales_end_at {
    if env.block.time < sales_end_at && !raffle.is_sold_out() {
      return Err(ContractError::NotSoldOut {});
    }
  }

  let mut cw20_transfer_msgs: Vec<SubMsg> = vec![];
  let mut send_msgs: Vec<CosmosMsg> = vec![];

  // randomly select the winner wallet address
  let winning_addr = draw_winner(deps.storage, &mut raffle, &env)?;

  // build msgs to transfer auto-transferable assets from contract to winner
  for asset in raffle.assets.iter() {
    if let RaffleAsset::Token { token, amount, .. } = &asset {
      match token {
        Token::Native { denom } => send_msgs.push(build_send_msg(&winning_addr, denom, *amount)?),
        Token::Cw20 { address: cw20_addr } => {
          cw20_transfer_msgs.push(build_cw20_transfer_msg(&winning_addr, cw20_addr, *amount)?)
        },
      }
    }
  }

  raffle.winner_address = Some(winning_addr);

  let total_amount = raffle.price.amount * Uint128::from(raffle.tickets_sold);
  let mut total_tax_pct = 0u8;

  // prepare list of (addr, tax_amount) tuples for building send msgs
  let tax_addrs: Vec<(Addr, Uint128)> = [
    (GELOTTO_ADDR, GELOTTO_PCT),
    (GELOTTO_ANNUAL_PRIZE_ADDR, GELOTTO_ANNUAL_PRIZE_PCT),
    (GELOTTO_NFT_1_REWARDS_ADDR, GELOTTO_NFT_1_REWARDS_PCT),
  ]
  .iter()
  .map(|(s, n)| {
    total_tax_pct += *n;
    (Addr::unchecked(*s), Uint128::from(*n))
  })
  .collect();

  // compute total amount of proceeds remaining after tax, which is further
  // divided among royalty recipients, according to their assigned
  // percentages.
  let total_after_tax =
    (Uint128::from(100u8 - total_tax_pct) * total_amount) / Uint128::from(100u8);

  // build transfer msgs for sending proceeds to royalty recipients and gelotto
  match &raffle.price.token {
    Token::Native { denom } => {
      // send royalties
      for result in ROYALTIES.iter(deps.storage)? {
        if let Ok(recipient) = result {
          let amount = (Uint128::from(recipient.pct) * total_after_tax) / Uint128::from(100u8);
          send_msgs.push(build_send_msg(&recipient.address, denom, amount)?);
        }
      }
      // send gelotto taxes
      for (addr, pct) in tax_addrs.iter() {
        let tax_amount = ((*pct) * total_amount) / Uint128::from(100u8);
        send_msgs.push(build_send_msg(addr, denom, tax_amount)?);
      }
    },
    Token::Cw20 { address: cw20_addr } => {
      // send royalties
      for result in ROYALTIES.iter(deps.storage)? {
        if let Ok(recipient) = result {
          let amount = (Uint128::from(recipient.pct) * total_after_tax) / Uint128::from(100u8);
          cw20_transfer_msgs.push(build_cw20_transfer_msg(
            &recipient.address,
            cw20_addr,
            amount,
          )?);
        }
      }
      // send gelotto taxes
      for (addr, pct) in tax_addrs.iter() {
        let tax_amount = ((*pct) * total_amount) / Uint128::from(100u8);
        cw20_transfer_msgs.push(build_cw20_transfer_msg(addr, cw20_addr, tax_amount)?);
      }
    },
  }

  raffle.status = RaffleStatus::Complete;

  RAFFLE.save(deps.storage, &raffle)?;

  let resp = Response::new()
    .add_attributes(vec![attr("action", "choose_winner")])
    .add_messages(send_msgs)
    .add_submessages(cw20_transfer_msgs);

  Ok(resp)
}
