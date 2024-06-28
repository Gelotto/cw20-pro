use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// pub const RAFFLE_MANAGER: Item<Addr> = Item::new("raffle_manager");
// pub const RAFFLE_STATUS: Item<RaffleStatus> = Item::new("raffle_status");
// pub const RAFFLE_NOIS_PROXY_ADDR: Item<Addr> = Item::new("raffle_nois_proxy_addr");
// pub const RAFFLE_NFT_OWNER_ADDR: Item<Addr> = Item::new("raffle_nft_owner_addrk");
// pub const RAFFLE_CW721_ADDR: Item<Addr> = Item::new("raffle_cw721_addr");
// pub const RAFFLE_CW721_TOKEN_ID: Item<String> = Item::new("raffle_cw721_token_id");
// pub const RAFFLE_CREATED_AT: Item<Timestamp> = Item::new("raffle_created_at");
// pub const RAFFLE_STARTED_AT: Item<Timestamp> = Item::new("raffle_started_at");
// pub const RAFFLE_ENDS_AT: Item<Timestamp> = Item::new("raffle_ends_at");
// pub const RAFFLE_LOCKUP_SECONDS: Item<Uint64> = Item::new("raffle_lockup_seconds");

pub const RANKED_BALANCES: Map<(u128, &Addr), u8> = Map::new("pro_ranked_balances");
