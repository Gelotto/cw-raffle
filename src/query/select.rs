use crate::{
  models::{ContractResult, WalletMetadata},
  msg::SelectResponse,
  state::{
    ACL_ADDRESS, MARKETING_INFO, RAFFLE, RAFFLE_OWNER, ROYALTIES, TICKET_ORDERS, WALLET_METADATA,
  },
};
use cosmwasm_std::{Addr, Deps, Order};
use cw_repository::client::Repository;

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
  _wallet: Option<Addr>,
) -> ContractResult<SelectResponse> {
  let loader = Repository::loader(deps.storage, &fields);
  Ok(SelectResponse {
    owner: loader.get("owner", &RAFFLE_OWNER)?,

    acl_address: loader.get("acl_address", &ACL_ADDRESS)?,

    raffle: loader.get("raffle", &RAFFLE)?,

    marketing: loader.get("marketing", &MARKETING_INFO)?,

    wallets: loader.view("wallets", || {
      let mut wallet_metas: Vec<WalletMetadata> = WALLET_METADATA
        .range(deps.storage, None, None, Order::Descending)
        .map(|result| {
          let (addr, mut meta) = result.unwrap();
          meta.address = Some(addr);
          meta
        })
        .collect();
      // sort wallet metas by largest ticket count first
      wallet_metas.sort_by(|a, b| b.ticket_count.cmp(&a.ticket_count));
      Ok(Some(wallet_metas))
    })?,

    royalties: loader.view("royalties", || {
      Ok(Some(
        ROYALTIES.iter(deps.storage)?.map(|x| x.unwrap()).collect(),
      ))
    })?,

    orders: loader.view("orders", || {
      Ok(Some(
        TICKET_ORDERS
          .iter(deps.storage)?
          .map(|x| x.unwrap())
          .collect(),
      ))
    })?,
  })
}
