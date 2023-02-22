use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub cw20_token_address: String,
    pub contract_minter_address: String, // core middleware contract
    pub tf_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetConfig)]
    GetConfig {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetConfig {
    pub cw20_token_address: String,
    pub contract_minter_address: String,
    pub tf_denom: String,
    // pub mode: String, // I could allow this now?
}
