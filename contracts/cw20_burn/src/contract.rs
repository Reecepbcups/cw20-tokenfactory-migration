#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, BankMsg, Coin, from_binary, WasmMsg, CosmosMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetConfig, InstantiateMsg, QueryMsg, Cw20HookMsg};
use crate::state::{State, STATE};
use token_bindings::{TokenFactoryMsg, TokenFactoryQuery, TokenMsg, TokenQuerier};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-burn";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<TokenFactoryQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let tf_denom = msg.tf_denom;
    let cw20_addr = deps.api.addr_validate(&msg.cw20_address)?;
    
    if !tf_denom.clone().starts_with("factory/") {
        return Err(ContractError::InvalidDenom{
            denom: tf_denom.clone(),
            message: "Denom must start with 'factory/'".to_string(),            
        });
    }
    
    let state = State {        
        cw20_address: cw20_addr,
        tf_denom,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
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
            let tf_denom = STATE.load(deps.storage)?.tf_denom;
            // let sender = cw20_msg.sender.clone();
            let amt = cw20_msg.amount;                        
            
            let contract_balance = deps.querier.query_all_balances(env.contract.address)?;
            let contract_balance = contract_balance.iter().find(|c| c.denom == tf_denom).unwrap();
        
            if contract_balance.amount < amt {                        
                return Err(ContractError::InvalidDenom{
                    denom: tf_denom.clone(),
                    message: "The contract has run out of funds to redeem these CW20 tokens, talk to an admin.".to_string(),            
                });
            }

            // match from_binary(&cw20_msg.msg) {
            //     // DepositMint & DepositBalance? <- balance pulls from contract balance
            //     Ok(Cw20HookMsg::Deposit {}) => execute_redeem(deps, info, cw20_msg),
            //     _ => Err(ContractError::InvalidCW20Message {}),
            // }         
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
    let state_address = STATE.load(deps.storage)?.cw20_address;

    if cw20_contract != state_address.to_string() {
        return Err(ContractError::InvalidCW20Address {});
    }

    // Send our token-factory balance to the sender of the CW20 tokens
    let tf_denom = STATE.load(deps.storage)?.tf_denom;
    let sender = cw20_msg.sender.clone();    
    let bank_msg = BankMsg::Send { 
        to_address: sender,
        amount: vec![Coin {
            denom: tf_denom,
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
        .add_message(bank_msg)
        .add_message(cw20_burn_msg)
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
            })            
        },        
    }
}



// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{
//         mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
//     };
//     use cosmwasm_std::{
//         coins, from_binary, Attribute, ContractResult, CosmosMsg, OwnedDeps, Querier, StdError,
//         SystemError, SystemResult,
//     };
//     use std::marker::PhantomData;
//     use token_bindings::TokenQuery;
//     use token_bindings_test::TokenFactoryApp;

//     const DENOM_NAME: &str = "mydenom";
//     const DENOM_PREFIX: &str = "factory";

//     fn mock_dependencies_with_custom_quierier<Q: Querier>(
//         querier: Q,
//     ) -> OwnedDeps<MockStorage, MockApi, Q, TokenFactoryQuery> {
//         OwnedDeps {
//             storage: MockStorage::default(),
//             api: MockApi::default(),
//             querier,
//             custom_query_type: PhantomData,
//         }
//     }

//     fn mock_dependencies_with_query_error(
//     ) -> OwnedDeps<MockStorage, MockApi, MockQuerier<TokenFactoryQuery>, TokenFactoryQuery> {
//         let custom_querier: MockQuerier<TokenFactoryQuery> =
//             MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]).with_custom_handler(|a| match a {
//                 TokenFactoryQuery::Token(TokenQuery::FullDenom {
//                     creator_addr,
//                     subdenom,
//                 }) => {
//                     let binary_request = to_binary(a).unwrap();

//                     if creator_addr.eq("") {
//                         return SystemResult::Err(SystemError::InvalidRequest {
//                             error: String::from("invalid creator address"),
//                             request: binary_request,
//                         });
//                     }
//                     if subdenom.eq("") {
//                         return SystemResult::Err(SystemError::InvalidRequest {
//                             error: String::from("invalid subdenom"),
//                             request: binary_request,
//                         });
//                     }
//                     SystemResult::Ok(ContractResult::Ok(binary_request))
//                 }
//                 _ => todo!(),
//             });
//         mock_dependencies_with_custom_quierier(custom_querier)
//     }

//     pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, TokenFactoryApp, TokenFactoryQuery>
//     {
//         let custom_querier = TokenFactoryApp::new();
//         mock_dependencies_with_custom_quierier(custom_querier)
//     }

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg {};
//         let info = mock_info("creator", &coins(1000, "uosmo"));

//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());
//     }

//     #[test]
//     fn query_get_denom() {
//         let deps = mock_dependencies();
//         let get_denom_query = QueryMsg::GetDenom {
//             creator_address: String::from(MOCK_CONTRACT_ADDR),
//             subdenom: String::from(DENOM_NAME),
//         };
//         let response = query(deps.as_ref(), mock_env(), get_denom_query).unwrap();
//         let get_denom_response: GetDenomResponse = from_binary(&response).unwrap();
//         assert_eq!(
//             format!("{}/{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME),
//             get_denom_response.denom
//         );
//     }

//     #[test]
//     fn msg_create_denom_success() {
//         let mut deps = mock_dependencies();

//         let subdenom: String = String::from(DENOM_NAME);

//         let msg = ExecuteMsg::CreateDenom { subdenom };
//         let info = mock_info("creator", &coins(2, "token"));
//         let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         assert_eq!(1, res.messages.len());

//         let expected_message = CosmosMsg::from(TokenMsg::CreateDenom {
//             subdenom: String::from(DENOM_NAME),
//             metadata: None,
//         });
//         let actual_message = res.messages.get(0).unwrap();
//         assert_eq!(expected_message, actual_message.msg);

//         assert_eq!(1, res.attributes.len());

//         let expected_attribute = Attribute::new("method", "create_denom");
//         let actual_attribute = res.attributes.get(0).unwrap();
//         assert_eq!(expected_attribute, actual_attribute);

//         assert_eq!(res.data.ok_or(0), Err(0));
//     }

//     #[test]
//     fn msg_create_denom_invalid_subdenom() {
//         let mut deps = mock_dependencies();

//         let subdenom: String = String::from("");

//         let msg = ExecuteMsg::CreateDenom { subdenom };
//         let info = mock_info("creator", &coins(2, "token"));
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//         assert_eq!(
//             ContractError::InvalidSubdenom {
//                 subdenom: String::from("")
//             },
//             err
//         );
//     }

//     #[test]
//     fn msg_change_admin_success() {
//         let mut deps = mock_dependencies();

//         const NEW_ADMIN_ADDR: &str = "newadmin";

//         let info = mock_info("creator", &coins(2, "token"));

//         let full_denom_name: &str =
//             &format!("{}/{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME)[..];

//         let msg = ExecuteMsg::ChangeAdmin {
//             denom: String::from(full_denom_name),
//             new_admin_address: String::from(NEW_ADMIN_ADDR),
//         };
//         let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         assert_eq!(1, res.messages.len());

//         let expected_message = CosmosMsg::from(TokenMsg::ChangeAdmin {
//             denom: String::from(full_denom_name),
//             new_admin_address: String::from(NEW_ADMIN_ADDR),
//         });
//         let actual_message = res.messages.get(0).unwrap();
//         assert_eq!(expected_message, actual_message.msg);

//         assert_eq!(1, res.attributes.len());

//         let expected_attribute = Attribute::new("method", "change_admin");
//         let actual_attribute = res.attributes.get(0).unwrap();
//         assert_eq!(expected_attribute, actual_attribute);

//         assert_eq!(res.data.ok_or(0), Err(0));
//     }

//     #[test]
//     fn msg_change_admin_empty_address() {
//         let mut deps = mock_dependencies();

//         const EMPTY_ADDR: &str = "";

//         let info = mock_info("creator", &coins(2, "token"));

//         let msg = ExecuteMsg::ChangeAdmin {
//             denom: String::from(DENOM_NAME),
//             new_admin_address: String::from(EMPTY_ADDR),
//         };
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//         match err {
//             ContractError::Std(StdError::GenericErr { msg, .. }) => {
//                 assert!(msg.contains("human address too short"))
//             }
//             e => panic!("Unexpected error: {:?}", e),
//         }
//     }

//     #[test]
//     fn msg_validate_denom_too_many_parts_valid() {
//         let mut deps = mock_dependencies();

//         // too many parts in denom
//         let full_denom_name: &str =
//             &format!("{}/{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME)[..];

//         validate_denom(deps.as_mut(), String::from(full_denom_name)).unwrap()
//     }

//     #[test]
//     fn msg_change_admin_invalid_denom() {
//         let mut deps = mock_dependencies();

//         const NEW_ADMIN_ADDR: &str = "newadmin";

//         let info = mock_info("creator", &coins(2, "token"));

//         // too many parts in denom
//         let full_denom_name: &str = &format!(
//             "{}/{}/{}/invalid",
//             DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME
//         )[..];

//         let msg = ExecuteMsg::ChangeAdmin {
//             denom: String::from(full_denom_name),
//             new_admin_address: String::from(NEW_ADMIN_ADDR),
//         };
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

//         let expected_error = ContractError::InvalidDenom {
//             denom: String::from(full_denom_name),
//             message: String::from("denom must have 3 parts separated by /, had 4"),
//         };

//         assert_eq!(expected_error, err);
//     }

//     #[test]
//     fn msg_mint_tokens_success() {
//         let mut deps = mock_dependencies();

//         const NEW_ADMIN_ADDR: &str = "newadmin";

//         let mint_amount = Uint128::new(100_u128);

//         let full_denom_name: &str =
//             &format!("{}/{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME)[..];

//         let info = mock_info("creator", &coins(2, "token"));

//         let msg = ExecuteMsg::MintTokens {
//             denom: String::from(full_denom_name),
//             amount: mint_amount,
//             mint_to_address: String::from(NEW_ADMIN_ADDR),
//         };
//         let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         assert_eq!(1, res.messages.len());

//         let expected_message = CosmosMsg::from(TokenMsg::MintTokens {
//             denom: String::from(full_denom_name),
//             amount: mint_amount,
//             mint_to_address: String::from(NEW_ADMIN_ADDR),
//         });
//         let actual_message = res.messages.get(0).unwrap();
//         assert_eq!(expected_message, actual_message.msg);

//         assert_eq!(1, res.attributes.len());

//         let expected_attribute = Attribute::new("method", "mint_tokens");
//         let actual_attribute = res.attributes.get(0).unwrap();
//         assert_eq!(expected_attribute, actual_attribute);

//         assert_eq!(res.data.ok_or(0), Err(0));
//     }

//     #[test]
//     fn msg_mint_invalid_denom() {
//         let mut deps = mock_dependencies();

//         const NEW_ADMIN_ADDR: &str = "newadmin";

//         let mint_amount = Uint128::new(100_u128);

//         let info = mock_info("creator", &coins(2, "token"));

//         let full_denom_name: &str = &format!("{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR)[..];
//         let msg = ExecuteMsg::MintTokens {
//             denom: String::from(full_denom_name),
//             amount: mint_amount,
//             mint_to_address: String::from(NEW_ADMIN_ADDR),
//         };
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//         let expected_error = ContractError::InvalidDenom {
//             denom: String::from(full_denom_name),
//             message: String::from("denom must have 3 parts separated by /, had 2"),
//         };

//         assert_eq!(expected_error, err);
//     }

//     #[test]
//     fn msg_burn_tokens_success() {
//         let mut deps = mock_dependencies();

//         let mint_amount = Uint128::new(100_u128);
//         let full_denom_name: &str =
//             &format!("{}/{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME)[..];

//         let info = mock_info("creator", &coins(2, "token"));

//         let msg = ExecuteMsg::BurnTokens {
//             denom: String::from(full_denom_name),
//             burn_from_address: String::from(""),
//             amount: mint_amount,
//         };
//         let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         assert_eq!(1, res.messages.len());
//         let expected_message = CosmosMsg::from(TokenMsg::BurnTokens {
//             denom: String::from(full_denom_name),
//             amount: mint_amount,
//             burn_from_address: String::from(""),
//         });
//         let actual_message = res.messages.get(0).unwrap();
//         assert_eq!(expected_message, actual_message.msg);

//         assert_eq!(1, res.attributes.len());

//         let expected_attribute = Attribute::new("method", "burn_tokens");
//         let actual_attribute = res.attributes.get(0).unwrap();
//         assert_eq!(expected_attribute, actual_attribute);

//         assert_eq!(res.data.ok_or(0), Err(0))
//     }

//     #[test]
//     fn msg_burn_tokens_input_address() {
//         let mut deps = mock_dependencies();

//         const BURN_FROM_ADDR: &str = "burnfrom";
//         let burn_amount = Uint128::new(100_u128);
//         let full_denom_name: &str =
//             &format!("{}/{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME)[..];

//         let info = mock_info("creator", &coins(2, "token"));

//         let msg = ExecuteMsg::BurnTokens {
//             denom: String::from(full_denom_name),
//             burn_from_address: String::from(BURN_FROM_ADDR),
//             amount: burn_amount,
//         };
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

//         let expected_error = ContractError::BurnFromAddressNotSupported {
//             address: String::from(BURN_FROM_ADDR),
//         };

//         assert_eq!(expected_error, err)
//     }

//     #[test]
//     fn msg_validate_denom_too_many_parts_invalid() {
//         let mut deps = mock_dependencies();

//         // too many parts in denom
//         let full_denom_name: &str = &format!(
//             "{}/{}/{}/invalid",
//             DENOM_PREFIX, MOCK_CONTRACT_ADDR, DENOM_NAME
//         )[..];

//         let err = validate_denom(deps.as_mut(), String::from(full_denom_name)).unwrap_err();

//         let expected_error = ContractError::InvalidDenom {
//             denom: String::from(full_denom_name),
//             message: String::from("denom must have 3 parts separated by /, had 4"),
//         };

//         assert_eq!(expected_error, err);
//     }

//     #[test]
//     fn msg_validate_denom_not_enough_parts_invalid() {
//         let mut deps = mock_dependencies();

//         // too little parts in denom
//         let full_denom_name: &str = &format!("{}/{}", DENOM_PREFIX, MOCK_CONTRACT_ADDR)[..];

//         let err = validate_denom(deps.as_mut(), String::from(full_denom_name)).unwrap_err();

//         let expected_error = ContractError::InvalidDenom {
//             denom: String::from(full_denom_name),
//             message: String::from("denom must have 3 parts separated by /, had 2"),
//         };

//         assert_eq!(expected_error, err);
//     }

//     #[test]
//     fn msg_validate_denom_denom_prefix_invalid() {
//         let mut deps = mock_dependencies();

//         // invalid denom prefix
//         let full_denom_name: &str =
//             &format!("{}/{}/{}", "invalid", MOCK_CONTRACT_ADDR, DENOM_NAME)[..];

//         let err = validate_denom(deps.as_mut(), String::from(full_denom_name)).unwrap_err();

//         let expected_error = ContractError::InvalidDenom {
//             denom: String::from(full_denom_name),
//             message: String::from("prefix must be 'factory', was invalid"),
//         };

//         assert_eq!(expected_error, err);
//     }

//     #[test]
//     fn msg_validate_denom_creator_address_invalid() {
//         let mut deps = mock_dependencies_with_query_error();

//         let full_denom_name: &str = &format!("{}/{}/{}", DENOM_PREFIX, "", DENOM_NAME)[..]; // empty contract address

//         let err = validate_denom(deps.as_mut(), String::from(full_denom_name)).unwrap_err();

//         match err {
//             ContractError::InvalidDenom { denom, message } => {
//                 assert_eq!(String::from(full_denom_name), denom);
//                 assert!(message.contains("invalid creator address"))
//             }
//             err => panic!("Unexpected error: {:?}", err),
//         }
//     }
// }
