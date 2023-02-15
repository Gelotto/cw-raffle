use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_lib::models::TokenAmount;

use crate::models::{
  Asset, Raffle, RaffleMetadata, RoyaltyRecipient, SocialMediaUrls, Style, TicketOrder,
  WalletMetadata,
};

#[cw_serde]
pub struct InstantiateMsg {
  pub ticket_supply: Option<u32>,
  pub ticket_sales_end_at: Option<Timestamp>,
  pub royalties: Vec<RoyaltyRecipient>,
  pub name: String,
  pub website: Option<String>,
  pub description: Option<String>,
  pub socials: Option<Vec<SocialMediaUrls>>,
  pub terms: Option<String>,
  pub assets: Vec<Asset>,
  pub price: TokenAmount,
  pub style: Style,
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
}

#[cw_serde]
pub struct SelectResponse {
  pub owner: Option<Addr>,
  pub raffle: Option<Raffle>,
  pub metadata: Option<RaffleMetadata>,
  pub orders: Option<Vec<TicketOrder>>,
  pub wallets: Option<Vec<WalletMetadata>>,
  pub royalties: Option<Vec<RoyaltyRecipient>>,
}

#[cw_serde]
pub struct RefundStatusResponse {
  pub has_claimed: bool,
}
