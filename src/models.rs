use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_lib::models::{Token, TokenAmount};

use crate::error::ContractError;

pub const RAFFLE_STAGE_HAS_BUYERS: u8 = 3;
pub const RAFFLE_STAGE_ACTIVE: u8 = 2;
pub const RAFFLE_STAGE_COMPLETED: u8 = 1;
pub const RAFFLE_STAGE_CANCELED: u8 = 0;

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
  Teritori,
  Juno,
}

#[cw_serde]
pub struct RaffleStyle {
  ui_base_color: String,
  ui_focus_color: Option<String>,
  font_color: Option<String>,
  bg_color: String,
  bg_src: Option<String>,
  font_family: Option<String>,
}

#[cw_serde]
pub enum SocialMediaUrl {
  Instagram(String),
  Facebook(String),
  Tiktok(String),
  Linkedin(String),
  Github(String),
  Reddit(String),
  Youtube(String),
  Twitter(String),
  Discord(String),
  Telegram(String),
}

#[cw_serde]
pub enum RaffleAsset {
  Token {
    token: Token,
    amount: Uint128,
    terms: Option<String>,
  },
  Nft {
    network: Network,
    url: Option<String>,
    collection_address: Addr,
    token_id: String,
    terms: Option<String>,
    image_url: Option<String>,
  },
  Asset {
    name: String,
    description: Option<String>,
    url: Option<String>,
    image: Option<String>,
    address: Option<Addr>,
    terms: Option<String>,
  },
}

#[cw_serde]
pub struct TicketOrder {
  pub address: Addr,
  pub count: u32,
  pub is_visible: bool,
}

#[cw_serde]
pub struct Raffle {
  pub price: TokenAmount,
  pub assets: Vec<RaffleAsset>,
  pub status: RaffleStatus,
  pub ticket_supply: Option<u32>,
  pub ticket_sales_end_at: Option<Timestamp>,
  pub ticket_sales_target: Option<u32>,
  pub winner_address: Option<Addr>,
  pub tickets_sold: u32,
  pub wallet_count: u32,
  pub seed: String,
}

impl Raffle {
  pub fn is_sold_out(&self) -> bool {
    if let Some(n) = self.ticket_supply {
      return n == self.tickets_sold;
    }
    return false;
  }
}

#[cw_serde]
pub struct RaffleMarketingInfo {
  pub style: RaffleStyle,
  pub raffle_name: String,
  pub org_name: Option<String>,
  pub org_logo_url: Option<String>,
  pub org_wallet: Option<Addr>,
  pub youtube_video_id: Option<String>,
  pub website: Option<String>,
  pub description: Option<String>,
  pub socials: Option<Vec<SocialMediaUrl>>,
}

#[cw_serde]
pub struct WalletMetadata {
  pub has_agreed_to_terms: bool,
  pub ticket_order_count: u16,
  pub ticket_count: u32,
  pub address: Option<Addr>,
  pub display_message: Option<String>,
  pub has_claimed_refund: bool,
}
