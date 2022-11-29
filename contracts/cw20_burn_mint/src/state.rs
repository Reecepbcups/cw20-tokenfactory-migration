use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct State {
    pub cw20_address: Addr,
    pub tf_denom: String,
    pub prev_admin: Addr,
}

pub const STATE: Item<State> = Item::new("state");
