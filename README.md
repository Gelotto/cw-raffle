# CosmWasm General Asset Raffle Contract

This is a smart contact for raffling any kind of asset, some which may be
transferable by the contract itself, like tokens, or others which must be
transferred off-chain.

## API

### Execute Functions

```rust
pub enum ExecuteMsg {
  // buys a specified number of tickets, along with a "lucky message" that is
  // displayed publicly in the front end when `is_visible` is set.
  BuyTickets {
    count: u32,
    message: Option<String>,
    is_visible: bool,
  },

  // As the raffle owner, this triggers the random drawing of the winner
  // address, transfering royalties as well as any auto-transferrable asset
  // being raffled. This puts the raffle into the Completed state.
  ChooseWinner {},

  // As the raffle owner, you can cancel the raffle so long as it is still in
  // the Active state. Upon cancelation, the auto-transferable assets in the pot
  // are transferred back to the raffle owner. At the same time, ticket holders
  // can now claim refunds.
  Cancel {},

  // If the raffle has been canceled and is in the Canceled state, ticket
  // holders can claim a full refund by calling this function.
  ClaimRefund {},

  // transfers ownership of the raffle contract to a given address.
  TransferOwnership {
    new_owner: Addr,
  },
}
```

### Query Functions

```rust
pub enum QueryMsg {
  // Selectively return named fields, including: owner, raffle, profile,
  // wallets, orders.
  Select { fields: Option<Vec<String>> },

  // Return true if the given claimant address has successfully claimed a refund.
  RefundStatus { claimant: Addr },
}
```
