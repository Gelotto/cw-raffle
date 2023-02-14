use crate::{models::ContractResult, msg::RefundStatusResponse, state::REFUND_STATUSES};
use cosmwasm_std::{Addr, Deps};

pub fn refund_status(
  deps: Deps,
  claimant: &Addr,
) -> ContractResult<RefundStatusResponse> {
  Ok(RefundStatusResponse {
    has_claimed: REFUND_STATUSES
      .load(deps.storage, claimant.clone())
      .unwrap_or(false),
  })
}
