use crate::models::{Raffle, RaffleMetadata, RaffleStatus, WalletMetadata};
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::TicketOrder};
use cosmwasm_std::{Addr, Binary, DepsMut, Env, MessageInfo, StdResult, Storage};
use cw_lib::random::{Pcg64, RngComponent};
use cw_storage_plus::{Deque, Item, Map};

pub const OWNER: Item<Addr> = Item::new("owner");
pub const RAFFLE: Item<Raffle> = Item::new("raffle");
pub const METADATA: Item<RaffleMetadata> = Item::new("raffle_metadata");
pub const TICKET_ORDERS: Deque<TicketOrder> = Deque::new("ticket_orders");
pub const WALLET_METADATA: Map<Addr, WalletMetadata> = Map::new("wallet_metadata");
pub const REFUND_STATUSES: Map<Addr, bool> = Map::new("refund_statuses");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  OWNER.save(deps.storage, &info.sender)?;
  METADATA.save(
    deps.storage,
    &RaffleMetadata {
      terms: msg.terms.clone(),
      style: msg.style.clone(),
      name: msg.name.clone(),
      website: msg.website.clone(),
      description: msg.description.clone(),
      socials: msg.socials.clone(),
    },
  )?;
  RAFFLE.save(
    deps.storage,
    &Raffle {
      assets: msg.assets.clone(),
      price: msg.price.clone(),
      status: RaffleStatus::Active,
      ticket_supply: msg.ticket_supply,
      ticket_sales_end_at: msg.ticket_sales_end_at,
      tickets_sold: 0,
      wallet_count: 0,
      seed: Binary::from(Pcg64::build_seed(&vec![
        RngComponent::Str(info.sender.to_string()),
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
      .to_base64(),
    },
  )?;
  Ok(())
}

pub fn is_owner(
  storage: &dyn Storage,
  addr: &Addr,
) -> StdResult<bool> {
  return Ok(OWNER.load(storage)? == *addr);
}
