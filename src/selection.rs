use std::collections::HashMap;

use cosmwasm_std::{Addr, Env, Order, Storage};
use cw_lib::random::{Pcg64, RngComponent};

use crate::{
  models::{ContractResult, Raffle},
  state::WALLET_METADATA,
};

pub fn draw_winner(
  storage: &dyn Storage,
  raffle: &Raffle,
  env: &Env,
) -> ContractResult<Addr> {
  let addrs = resolve_multiple_winners(storage, raffle, env, 1)?;
  Ok(addrs[0].clone())
}

pub fn resolve_multiple_winners(
  storage: &dyn Storage,
  raffle: &Raffle,
  env: &Env,
  count: u32,
) -> ContractResult<Vec<Addr>> {
  let mut rng = Pcg64::from_components(&vec![
    RngComponent::Str(raffle.seed.clone()),
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

  let mut bag: Vec<usize> = Vec::with_capacity(raffle.tickets_sold as usize);
  let mut idx: usize = 0;
  let mut idx_2_addr: HashMap<usize, Addr> = HashMap::new();
  let mut addr_2_idx: HashMap<Addr, usize> = HashMap::new();

  WALLET_METADATA
    .range(storage, None, None, Order::Ascending)
    .for_each(|result| {
      if let Ok((addr, meta)) = result {
        let idx = match addr_2_idx.get(&addr) {
          Some(idx) => *idx,
          None => {
            idx_2_addr.insert(idx, addr.clone());
            addr_2_idx.insert(addr, idx);
            idx += 1;
            idx - 1
          },
        };
        for _ in 0..meta.ticket_count {
          bag.push(idx);
        }
      }
    });

  let mut addrs: Vec<Addr> = Vec::with_capacity(count as usize);

  for _ in 0..(count as usize) {
    let bag_index = rng.next_u64() % (bag.len() as u64);
    let addr_index = bag[bag_index as usize].clone();
    addrs.push(idx_2_addr.get(&addr_index).unwrap().clone())
  }

  Ok(addrs)
}
