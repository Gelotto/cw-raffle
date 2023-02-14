use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_lib::models::{Token, TokenAmount};

use crate::error::ContractError;

pub type ContractResult<T> = Result<T, ContractError>;

#[cw_serde]
pub enum RaffleStatus {
  Active,
  Complete,
  Canceled,
}

#[cw_serde]
pub struct RoyaltyRecipient {
  pub name: Option<String>,
  pub address: Addr,
  pub pct: u8,
}

#[cw_serde]
pub enum Network {
  Stargaze,
}

#[cw_serde]
pub struct Style {
  background: String,
  color: String,
  font_family: Option<String>,
}

#[cw_serde]
pub struct SocialMediaUrls {
  instagram: Option<String>,
  facebook: Option<String>,
  tiktok: Option<String>,
  linkedin: Option<String>,
  github: Option<String>,
  reddit: Option<String>,
  youtube: Option<String>,
  twitter: Option<String>,
  discord: Option<String>,
}

#[cw_serde]
pub enum Asset {
  Token {
    token: Token,
    amount: Uint128,
  },
  Nft {
    network: Network,
    url: Option<String>,
    collection_address: Addr,
    token_id: String,
  },
  Other {
    name: String,
    description: Option<String>,
    url: Option<String>,
    image: Option<String>,
    address: Option<Addr>,
  },
}

#[cw_serde]
pub struct TicketOrder {
  pub address: Addr,
  pub count: u32,
  pub cum_count: u32,
  pub message: Option<String>,
  pub is_visible: bool,
}

#[cw_serde]
pub struct Raffle {
  pub price: TokenAmount,
  pub assets: Vec<Asset>,
  pub status: RaffleStatus,
  pub ticket_supply: Option<u32>,
  pub ticket_sales_end_at: Option<Timestamp>,
  pub tickets_sold: u32,
  pub wallet_count: u32,
  pub seed: String,
}

#[cw_serde]
pub struct RaffleMetadata {
  pub terms: Option<String>,
  pub style: Style,
  pub name: String,
  pub website: Option<String>,
  pub description: Option<String>,
  pub socials: Option<Vec<SocialMediaUrls>>,
}

#[cw_serde]
pub struct WalletMetadata {
  pub has_agreed_to_terms: bool,
  pub ticket_order_count: u16,
  pub ticket_count: u32,
}
