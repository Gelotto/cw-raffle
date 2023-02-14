use crate::{
  error::ContractError,
  models::{ContractResult, TicketOrder, WalletMetadata},
  state::{RAFFLE, TICKET_ORDERS, WALLET_METADATA},
};
use cosmwasm_std::{attr, Binary, DepsMut, Empty, Env, MessageInfo, Response, Uint128};
use cw_lib::{
  random::{Pcg64, RngComponent},
  utils::funds::{
    build_cw20_transfer_msg, build_send_msg, has_funds, require_balance, require_cw20_token_balance,
  },
};

pub fn buy_tickets(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  count: u32,
  message: Option<String>,
  is_visible: bool,
) -> Result<Response, ContractError> {
  let buyer = &info.sender;
  let mut raffle = RAFFLE.load(deps.storage)?;

  // abort if there aren't enough tickets left
  if let Some(ticket_supply) = raffle.ticket_supply {
    let tickets_remaining = ticket_supply - raffle.tickets_sold;
    if count == 0 || tickets_remaining < count {
      return Err(ContractError::SoldOut {});
    }
  }

  let mut resp: Response<Empty> = Response::new().add_attributes(vec![
    attr("action", "buy_tickets"),
    attr("count", count.to_string()),
  ]);

  let balance_required = Uint128::from(count) * raffle.price.amount;

  // verify buyer can make payment
  match &raffle.price.token {
    cw_lib::models::Token::Native { denom } => {
      require_balance(deps.querier, buyer, balance_required, denom, false)?;
      if !has_funds(&info.funds, balance_required, denom) {
        resp = resp.add_message(build_send_msg(
          &env.contract.address,
          denom,
          balance_required,
        )?);
      } else {
        return Err(ContractError::MissingFunds {});
      }
    },
    cw_lib::models::Token::Cw20 { address: cw20_addr } => {
      require_cw20_token_balance(deps.querier, buyer, balance_required, cw20_addr, false)?;
      resp = resp.add_submessage(build_cw20_transfer_msg(
        &env.contract.address,
        cw20_addr,
        balance_required,
      )?);
    },
  }

  // update tickets sold and RNG seed
  raffle.tickets_sold += count;
  raffle.seed = Binary::from(Pcg64::build_seed(&vec![
    RngComponent::Str(raffle.seed.clone()),
    RngComponent::Str(buyer.to_string()),
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
  ]))
  .to_base64();

  // update wallet-level metadata
  WALLET_METADATA.update(
    deps.storage,
    buyer.clone(),
    |maybe_meta| -> ContractResult<WalletMetadata> {
      if let Some(mut meta) = maybe_meta {
        meta.ticket_count += count;
        meta.ticket_order_count += 1;
        Ok(meta)
      } else {
        Ok(WalletMetadata {
          has_agreed_to_terms: true,
          ticket_order_count: 1,
          ticket_count: count,
        })
      }
    },
  )?;

  RAFFLE.save(deps.storage, &raffle)?;

  TICKET_ORDERS.push_back(
    deps.storage,
    &TicketOrder {
      address: buyer.clone(),
      cum_count: raffle.tickets_sold + count,
      message: message.clone(),
      is_visible,
      count,
    },
  )?;

  Ok(resp)
}
