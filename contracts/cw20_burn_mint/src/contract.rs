#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use token_bindings::{TokenFactoryMsg, TokenMsg};

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
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let tf_denom = msg.tf_denom;
    let cw20_address = deps.api.addr_validate(&msg.cw20_address)?;

    if !tf_denom.starts_with("factory/") {
        return Err(ContractError::InvalidDenom {
            denom: tf_denom,
            message: "Denom must start with 'factory/'".to_string(),
        });
    }

    let state = State {
        cw20_address,
        tf_denom,
        prev_admin: deps
            .api
            .addr_validate(&msg.admin.unwrap_or_else(|| info.sender.to_string()))?,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => execute_redeem_mint(deps, info, cw20_msg),
        ExecuteMsg::TransferBackAdmin {} => execute_transfer_back_admin(deps, info),
    }
}

pub fn execute_transfer_back_admin(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.prev_admin {
        return Err(ContractError::Unauthorized {});
    }

    let msg = TokenMsg::ChangeAdmin {
        denom: state.tf_denom,
        new_admin_address: state.prev_admin.to_string(),
    };

    Ok(Response::new()
        .add_attribute("method", "transfer_back_admin")
        .add_attribute("new_admin", info.sender)
        .add_message(msg))
}

pub fn execute_redeem_mint(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let cw20_contract = info.sender.to_string();
    let state = STATE.load(deps.storage)?;

    if cw20_contract != state.cw20_address {
        return Err(ContractError::InvalidCW20Address {});
    }

    // Mint the tokens to their account
    let mint_tokens_msg =
        TokenMsg::mint_contract_tokens(state.tf_denom, cw20_msg.amount, cw20_msg.sender.clone());

    // Burn the CW20 since it is in our possession now
    let cw20_burn = cw20::Cw20ExecuteMsg::Burn {
        amount: cw20_msg.amount,
    };
    let cw20_burn_msg: WasmMsg = WasmMsg::Execute {
        contract_addr: cw20_contract,
        msg: to_binary(&cw20_burn)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("method", "redeem_mint")
        .add_message(cw20_burn_msg)
        .add_message(mint_tokens_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&GetConfig {
                cw20_address: state.cw20_address.into_string(),
                tf_denom: state.tf_denom,
                mode: "mint".to_string(),
            })
        }
    }
}

// TODO: test cw20 -> native denom
