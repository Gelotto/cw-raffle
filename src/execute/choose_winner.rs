use crate::{
  error::ContractError,
  models::{Asset, ContractResult, Raffle, RaffleStatus, TicketOrder},
  state::{is_owner, RAFFLE, ROYALTIES, TICKET_ORDERS},
};
use cosmwasm_std::{
  attr, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, Storage, SubMsg, Uint128,
};
use cw_lib::{
  models::Token,
  random::{Pcg64, RngComponent},
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

  // prevent raffle from being double-ended
  if raffle.status != RaffleStatus::Active {
    return Err(ContractError::NotAuthorized {});
  }

  // ensure the game status can transition from active
  if let Some(sales_end_at) = raffle.ticket_sales_end_at {
    if env.block.time < sales_end_at {
      // ticket sales still haven't ended
      return Err(ContractError::NotAuthorized {});
    }
  }

  let mut cw20_transfer_msgs: Vec<SubMsg> = vec![];
  let mut send_msgs: Vec<CosmosMsg> = vec![];

  if raffle.tickets_sold == 0 {
    cancel_and_refund_owner(&mut raffle)?;
  } else {
    // randomly select the winner wallet address
    let winner = resolve_winner(deps.storage, &mut raffle, &env, &info.sender)?;

    // build msgs to transfer auto-transferable assets from contract to winner
    for asset in raffle.assets.iter() {
      if let Asset::Token { token, amount } = &asset {
        match token {
          Token::Native { denom } => send_msgs.push(build_send_msg(&winner, denom, *amount)?),
          Token::Cw20 { address: cw20_addr } => {
            cw20_transfer_msgs.push(build_cw20_transfer_msg(&winner, cw20_addr, *amount)?)
          },
        }
      }
    }

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
  }

  raffle.status = RaffleStatus::Complete;

  RAFFLE.save(deps.storage, &raffle)?;

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "choose_winner")])
      .add_messages(send_msgs)
      .add_submessages(cw20_transfer_msgs),
  )
}

fn resolve_winner(
  storage: &dyn Storage,
  raffle: &mut Raffle,
  env: &Env,
  sender: &Addr,
) -> ContractResult<Addr> {
  let mut rng = Pcg64::from_components(&vec![
    RngComponent::Str(raffle.seed.clone()),
    RngComponent::Str(sender.to_string()),
    RngComponent::Int(env.block.time.nanos()),
    RngComponent::Int(env.block.height),
    RngComponent::Int(
      env
        .transaction
        .as_ref()
        .and_then(|x| Some(x.index as u64))
        .or(Some(0))
        .unwrap(),
    ),
  ]);

  let orders: Vec<TicketOrder> = TICKET_ORDERS
    .iter(storage)?
    .map(|result| result.unwrap())
    .collect();

  let x = rng.next_u64() % (raffle.tickets_sold as u64);
  let winning_ticket_order_idx = bisect(&orders[..], orders.len(), x as u32);

  Ok(orders[winning_ticket_order_idx].address.clone())
}

/// Perform binary search using bisection
/// to determine which interval contains `x`.
fn bisect(
  orders: &[TicketOrder], // slice to search
  n: usize,               // length of slice
  x: u32,                 // random value
) -> usize {
  let i = n / 2;
  let order = &orders[i];
  let lower = order.cum_count - order.count;
  let upper = order.cum_count;
  if x < lower {
    // go left
    return bisect(&orders[..i], i, x);
  } else if x >= upper {
    // go right
    return bisect(&orders[i..], n - i, x);
  }
  i // return the index of the TicketOrder
}

fn cancel_and_refund_owner(raffle: &mut Raffle) -> ContractResult<()> {
  raffle.status = RaffleStatus::Canceled;
  Ok(())
}
