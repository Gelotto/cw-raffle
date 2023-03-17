use crate::models::{
  ContractResult, Raffle, RaffleMarketingInfo, RaffleStatus, RoyaltyRecipient, WalletMetadata,
};
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::TicketOrder};
use cosmwasm_std::{Addr, Binary, DepsMut, Env, MessageInfo, StdResult, Storage};
use cw_lib::random::{Pcg64, RngComponent};
use cw_repository::client::Repository;
use cw_storage_plus::{Deque, Item, Map};

pub const IX_U64_TICKETS_SOLD: u8 = 0;
pub const IX_U64_WALLET_COUNT: u8 = 1;
pub const IX_U64_STATUS: u8 = 2;

pub const IX_STR_OWNER: u8 = 0;
pub const IX_STR_ASSET: u8 = 1;

pub const REPO_CONTRACT_ADDR: Item<Addr> = Item::new("repo_contract_addr");
pub const RAFFLE_OWNER: Item<Addr> = Item::new("raffle_owner");
pub const RAFFLE: Item<Raffle> = Item::new("raffle");
pub const MARKETING_INFO: Item<RaffleMarketingInfo> = Item::new("raffle_metadata");
pub const TICKET_ORDERS: Deque<TicketOrder> = Deque::new("ticket_orders");
pub const ROYALTIES: Deque<RoyaltyRecipient> = Deque::new("royalties");
pub const WALLET_METADATA: Map<Addr, WalletMetadata> = Map::new("wallet_metadata");
pub const REFUND_STATUSES: Map<Addr, bool> = Map::new("refund_statuses");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  // Ensure at least 1 ticket is available. If both ticket sales target and a
  // finite ticket supply are defined, ensure that the supply is at least as
  // much as the target.
  if let Some(tick_supply) = msg.ticket_supply {
    if tick_supply == 0 {
      return Err(ContractError::InsufficientTicketSupply {});
    }
    if let Some(target) = msg.ticket_sales_target {
      if tick_supply < target {
        return Err(ContractError::InsufficientTicketSupply {});
      }
    }
  }
  // require at least 1 asset being raffled
  if msg.assets.is_empty() {
    return Err(ContractError::ValidationError {
      reason: Some("at least one asset is required".into()),
    });
  }

  REPO_CONTRACT_ADDR.save(deps.storage, &info.sender)?;

  RAFFLE_OWNER.save(deps.storage, &msg.owner.clone())?;

  for recipient in msg.royalties.iter() {
    ROYALTIES.push_back(deps.storage, &recipient)?;
  }

  MARKETING_INFO.save(
    deps.storage,
    &RaffleMarketingInfo {
      style: msg.style.clone(),
      raffle_name: msg.raffle_name.clone(),
      website: msg.website.clone(),
      description: msg.description.clone(),
      socials: msg.socials.clone(),
      youtube_video_id: msg.youtube_video_id.clone(),
      org_name: msg.org_name.clone(),
      org_wallet: msg.org_wallet.clone(),
      org_logo_url: msg.org_logo_url.clone(),
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
      ticket_sales_target: msg.ticket_sales_target,
      tickets_sold: 0,
      wallet_count: 0,
      winner_address: None,
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
  return Ok(RAFFLE_OWNER.load(storage)? == *addr);
}

pub fn repository(store: &dyn Storage) -> ContractResult<Repository> {
  Ok(Repository::new(&REPO_CONTRACT_ADDR.load(store)?))
}
