#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, BankMsg, Coin, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetConfig, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-burn";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let tf_denom = msg.tf_denom;
    let cw20_addr = deps.api.addr_validate(&msg.cw20_address)?;
    
    if !tf_denom.starts_with("factory/") {
        return Err(ContractError::InvalidDenom{
            denom: tf_denom,
            message: "Denom must start with 'factory/'".to_string(),            
        });
    }
    
    let state = State {        
        cw20_address: cw20_addr,
        tf_denom,
        contact: msg.contact,
    };    
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))        
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => {                
            let state = STATE.load(deps.storage)?;            
            let amt = cw20_msg.amount;                        
            
            let contract_balance = deps.querier.query_all_balances(env.contract.address)?;
            let contract_balance = contract_balance.iter().find(|c| c.denom == state.tf_denom).unwrap();
        
            if contract_balance.amount < amt {
                return Err(ContractError::OutOfFunds{
                    request: amt,
                    amount: contract_balance.amount,                
                });
            }
 
            execute_redeem(deps, info, cw20_msg)             
        },
    }
}

pub fn execute_redeem(    
    deps: DepsMut,    
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError>  {

    let cw20_contract = info.sender.to_string();    
    let state = STATE.load(deps.storage)?;

    if cw20_contract != state.cw20_address {
        return Err(ContractError::InvalidCW20Address {});
    }

    // Send our token-factory balance to the sender of the CW20 tokens for the exchange    
    let bank_msg = BankMsg::Send { 
        to_address: cw20_msg.sender.clone(),
        amount: vec![Coin {
            denom: state.tf_denom,
            amount: cw20_msg.amount,
        }] 
    };
    
    // Burn the CW20 since it is in our possession now
    let cw20_burn = cw20::Cw20ExecuteMsg::Burn { 
        amount: cw20_msg.amount 
    };
    let cw20_burn_msg: WasmMsg = WasmMsg::Execute {
        contract_addr: cw20_contract,
        msg: to_binary(&cw20_burn)?,
        funds: vec![],
    };    

    Ok(Response::new()
        .add_attribute("method", "redeem")
        .add_message(cw20_burn_msg)
        .add_message(bank_msg)
    )
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&GetConfig {
                cw20_address: state.cw20_address.into_string(),
                tf_denom: state.tf_denom,
                contact: state.contact.unwrap_or_default(),
            })            
        },        
    }
}

// TODO: test cw20 -> native denom