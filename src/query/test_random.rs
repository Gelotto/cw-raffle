use std::collections::HashMap;

use crate::{
  models::ContractResult, msg::RandomResponse, selection::resolve_multiple_winners, state::RAFFLE,
};
use cosmwasm_std::{Addr, Deps, Env};

pub fn test_random(
  deps: Deps,
  env: &Env,
) -> ContractResult<RandomResponse> {
  let raffle = RAFFLE.load(deps.storage)?;
  let mut results_map: HashMap<Addr, u32> = HashMap::with_capacity(4);

  if raffle.tickets_sold == 0 {
    return Ok(RandomResponse { results: vec![] });
  }

  for addr in resolve_multiple_winners(deps.storage, &raffle, env, 100)? {
    results_map.insert(addr.clone(), *(results_map.get(&addr).unwrap_or(&0)) + 1);
  }
  Ok(RandomResponse {
    results: results_map.iter().map(|(k, v)| (k.clone(), *v)).collect(),
  })
}
