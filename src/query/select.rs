use crate::{
  models::ContractResult,
  msg::SelectResponse,
  state::{METADATA, OWNER, RAFFLE, ROYALTIES, TICKET_ORDERS, WALLET_METADATA},
};
use cosmwasm_std::{Deps, Order};
use cw_repository::client::Repository;

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
) -> ContractResult<SelectResponse> {
  let loader = Repository::loader(deps.storage, &fields);
  Ok(SelectResponse {
    owner: loader.get("owner", &OWNER)?,
    raffle: loader.get("raffle", &RAFFLE)?,
    metadata: loader.get("profile", &METADATA)?,
    wallets: loader.view("wallets", || {
      Ok(Some(
        WALLET_METADATA
          .range(deps.storage, None, None, Order::Descending)
          .map(|result| {
            let (_addr, meta) = result.unwrap();
            meta
          })
          .collect(),
      ))
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
