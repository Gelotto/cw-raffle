use crate::{
  error::ContractError,
  models::{ContractResult, RaffleAsset, RaffleStatus, RAFFLE_STAGE_COMPLETED},
  selection::draw_winner,
  state::{is_allowed, repository, IX_U64_STATUS, RAFFLE, RAFFLE_OWNER, ROYALTIES},
};
use cosmwasm_std::{attr, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_submsg, build_send_msg, get_token_balance},
};

// addresses for gelotto taxes:
pub const GELOTTO_ADDR: &str = "juno1jume25ttjlcaqqjzjjqx9humvze3vcc8z87szj";
pub const GELOTTO_NFT_1_REWARDS_ADDR: &str = "juno18fd2xax0uh9dxusg8uae5rkeu8a4sv3gk6zm7h";
pub const GELOTTO_NFT_2_REWARDS_ADDR: &str = "juno13c97054tjktvzvgqe2xfxj28j6wmhhlz03ut32";
pub const GELOTTO_OWNERS_ADDR: &str = "juno1dunhw3y4m6lu642lk20hfq9q3scr70l2vuyrwj";

// percentages for tax allocations:
pub const GELOTTO_PCT: u128 = 200_000;
pub const GELOTTO_NFT_1_REWARDS_PCT: u128 = 200_000;
pub const GELOTTO_NFT_2_REWARDS_PCT: u128 = 200_000;
pub const GELOTTO_OWNERS_PCT: u128 = 200_000;
pub const RAFFLE_CREATOR_PCT: u128 = 200_000;

pub fn choose_winner(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  if !is_allowed(&deps.as_ref(), &info.sender, "choose_winner")? {
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
    if env.block.time < sales_end_at {
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
        Token::Cw20 { address: cw20_addr } => cw20_transfer_msgs.push(build_cw20_transfer_submsg(
          &winning_addr,
          cw20_addr,
          *amount,
        )?),
      }
    }
  }

  let total_pot: Uint128 = Uint128::from(raffle.tickets_sold) * raffle.price.amount;
  let total_royalties = total_pot.multiply_ratio(900_000u128, 1_000_000u128);
  let balance = get_token_balance(deps.querier, &env.contract.address, &raffle.price.token)?;
  let total_taxes = balance - total_royalties;

  // let total_amount = get_token_balance(deps.querier, &env.contract.address, &raffle.price.token)?;
  let owner = RAFFLE_OWNER.load(deps.storage)?;
  let mut total_tax_pct = 0u128;

  // prepare list of (addr, tax_amount) tuples for building send msgs
  let tax_payouts: Vec<(Addr, Uint128)> = [
    (GELOTTO_ADDR, GELOTTO_PCT),
    (GELOTTO_NFT_1_REWARDS_ADDR, GELOTTO_NFT_1_REWARDS_PCT),
    (GELOTTO_NFT_2_REWARDS_ADDR, GELOTTO_NFT_2_REWARDS_PCT),
    (GELOTTO_OWNERS_ADDR, GELOTTO_OWNERS_PCT),
    (owner.as_str(), RAFFLE_CREATOR_PCT),
  ]
  .iter()
  .map(|(s, n)| {
    total_tax_pct += *n;
    (Addr::unchecked(*s), Uint128::from(*n))
  })
  .collect();

  // build transfer msgs for sending proceeds to royalty recipients and gelotto
  match &raffle.price.token {
    Token::Native { denom } => {
      // send royalties
      for result in ROYALTIES.iter(deps.storage)? {
        if let Ok(recipient) = result {
          let amount = total_royalties.multiply_ratio(recipient.pct, 100u128);
          send_msgs.push(build_send_msg(&recipient.address, denom, amount)?);
        }
      }
      // send gelotto taxes
      for (addr, pct) in tax_payouts.iter() {
        let tax_amount = total_taxes.multiply_ratio(*pct, 1_000_000u128);
        send_msgs.push(build_send_msg(addr, denom, tax_amount)?);
      }
    },
    Token::Cw20 { address: cw20_addr } => {
      // send royalties
      for result in ROYALTIES.iter(deps.storage)? {
        if let Ok(recipient) = result {
          let amount = (Uint128::from(recipient.pct) * total_royalties) / Uint128::from(100u8);
          cw20_transfer_msgs.push(build_cw20_transfer_submsg(
            &recipient.address,
            cw20_addr,
            amount,
          )?);
        }
      }
      // send gelotto taxes
      for (addr, pct) in tax_payouts.iter() {
        let tax_amount = total_taxes.multiply_ratio(*pct, 1_000_000u128);
        cw20_transfer_msgs.push(build_cw20_transfer_submsg(addr, cw20_addr, tax_amount)?);
      }
    },
  }

  raffle.status = RaffleStatus::Complete;
  raffle.winner_address = Some(winning_addr.clone());

  RAFFLE.save(deps.storage, &raffle)?;

  let resp = Response::new()
    .add_attributes(vec![attr("action", "choose_winner")])
    .add_messages(send_msgs)
    .add_submessages(cw20_transfer_msgs);

  Ok(
    resp.add_message(
      repository(deps.storage)?
        .update()
        .set_u64(IX_U64_STATUS, RAFFLE_STAGE_COMPLETED as u64)
        .add_relationship(&winning_addr, "winner")
        .build_msg()?,
    ),
  )
}
