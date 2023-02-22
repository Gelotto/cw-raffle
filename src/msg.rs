use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_lib::models::TokenAmount;

use crate::models::{
  Raffle, RaffleAsset, RaffleMarketingInfo, RaffleStyle, RoyaltyRecipient, SocialMediaUrl,
  TicketOrder, WalletMetadata,
};

#[cw_serde]
pub struct InstantiateMsg {
  pub ticket_supply: Option<u32>,
  pub ticket_sales_end_at: Option<Timestamp>,
  pub royalties: Vec<RoyaltyRecipient>,
  pub raffle_name: String,
  pub org_name: Option<String>,
  pub youtube_video_id: Option<String>,
  pub website: Option<String>,
  pub description: Option<String>,
  pub socials: Option<Vec<SocialMediaUrl>>,
  pub terms: Option<String>,
  pub assets: Vec<RaffleAsset>,
  pub price: TokenAmount,
  pub style: RaffleStyle,
}

#[cw_serde]
pub enum ExecuteMsg {
  TransferOwnership {
    new_owner: Addr,
  },
  BuyTickets {
    count: u32,
    message: Option<String>,
    is_visible: bool,
  },
  ChooseWinner {},
  Cancel {},
  ClaimRefund {},
}

#[cw_serde]
pub enum QueryMsg {
  Select { fields: Option<Vec<String>> },
  RefundStatus { claimant: Addr },
  Random {},
}

#[cw_serde]
pub struct SelectResponse {
  pub owner: Option<Addr>,
  pub raffle: Option<Raffle>,
  pub marketing: Option<RaffleMarketingInfo>,
  pub orders: Option<Vec<TicketOrder>>,
  pub wallets: Option<Vec<WalletMetadata>>,
  pub royalties: Option<Vec<RoyaltyRecipient>>,
}

#[cw_serde]
pub struct RefundStatusResponse {
  pub has_claimed: bool,
}

#[cw_serde]
pub struct RandomResponse {
  pub results: Vec<(Addr, u32)>,
}
