# Test script for Juno Smart Contracts (By @Reecepbcups)
# ./github/workflows/e2e.yml
#
# sh ./e2e/test_e2e.sh
#
# NOTES: anytime you use jq, use `jq -rc` for ASSERT_* functions (-c removes format, -r is raw to remove \" quotes)

# get functions from helpers file 
# -> query_contract, wasm_cmd, mint_cw721, send_nft_to_listing, send_cw20_to_listing
source ./e2e/helpers.sh

CONTAINER_NAME="tokenfactory_migratecw20_test"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"

MAIN_REPO_RAW_ARTIFACTS="https://github.com/Reecepbcups/tokenfactory-core-contract/raw/main/artifacts"

function create_denom {
    RANDOM_STRING=$(cat /dev/urandom | tr -dc 'a-zA-Z' | fold -w 6 | head -n 1)

    $BINARY tx tokenfactory create-denom $RANDOM_STRING $JUNOD_COMMAND_ARGS    
    export FULL_DENOM="factory/$KEY_ADDR/$RANDOM_STRING" && echo $FULL_DENOM
}

# ========================
# === Contract Uploads ===
# ========================
# function upload_testing_contract {    

#     echo "Storing contract..."
#     UPLOAD=$($BINARY tx wasm store /tf_example.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
#     BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

#     # == INSTANTIATE ==
#     # PAYLOAD=$(printf '{"core_factory_address":"%s"}' $TF_CONTRACT) && echo $PAYLOAD
#     PAYLOAD=$(printf '{}' $TF_CONTRACT) && echo $PAYLOAD
#     TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "tf_test" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH


#     export TEST_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TEST_CONTRACT: $TEST_CONTRACT"
# }

function upload_cw20_base {
    UPLOAD=$($BINARY tx wasm store /cw20_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    CW20_TX_INIT=$($BINARY tx wasm instantiate "$BASE_CODE_ID" '{"name":"test","symbol":"aaaa","decimals":6,"initial_balances":[{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","amount":"100"}]}' --label "juno-cw20" $JUNOD_COMMAND_ARGS -y --admin $KEY_ADDR | jq -r '.txhash') && echo $CW20_TX_INIT
    export CW20_ADDR=$($BINARY query tx $CW20_TX_INIT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "$CW20_ADDR"
}



function transfer_denom_to_contract {
    # transfer admin to the contract from the user
    $BINARY tx tokenfactory change-admin $FULL_DENOM $TF_CONTRACT $JUNOD_COMMAND_ARGS
    $BINARY q tokenfactory denom-authority-metadata $FULL_DENOM # admin is the TF_CONTRACT
}

function upload_tokenfactory_core {
    # download latest core contract from public repo
    wget -O e2e/tokenfactory_core.wasm "$MAIN_REPO_RAW_ARTIFACTS/tokenfactory_core.wasm"

    echo "Storing contract..."
    create_denom
    UPLOAD=$($BINARY tx wasm store /tokenfactory_core.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    # no allowed_mint_addresses initially until we make the cw20burnmint, then we will add it as the admin of this contract via an execute
    PAYLOAD=$(printf '{"allowed_mint_addresses":[],"denoms":["%s"]}' $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "tf-middlware" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export TF_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TF_CONTRACT: $TF_CONTRACT"

    transfer_denom_to_contract
}

function upload_cw20burnmint { # must run after uploading the tokenfactory core
    echo "Storing contract..."
    # its from the root of the docker container
    UPLOAD=$($BINARY tx wasm store /cw20_burn_mint.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"
    PAYLOAD=$(printf '{"cw20_token_address":"%s","contract_minter_address":"%s","tf_denom":"%s"}' $CW20_ADDR $TF_CONTRACT $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "cw20burnmint" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export CW20_BURN=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW20_BURN: $CW20_BURN"

    # execute on the tokenfactory core as the admin to set this CW20_BURN contract to be allowed to mint on its behalf
    PAYLOAD=$(printf '{"add_whitelist":{"addresses":["%s"]}}' $CW20_BURN) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm execute "$TF_CONTRACT" "$PAYLOAD" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX_HASH

    # query the contract to see if it was added    
    $BINARY query wasm contract-state smart $TF_CONTRACT '{"get_config":{}}' --output json | jq .data.allowed_mint_addresses
    # the cw20burnmint address can now mint tokens from the TF_CONTRACT
}

# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====

# =============
# === LOGIC ===
# =============

start_docker
add_accounts
compile_and_copy # the compile takes time for the docker container to start up

sleep 5
health_status


# upload base contracts
upload_cw20_base
upload_tokenfactory_core

# this programs conrtacts
upload_cw20burnmint


# we are going to send some balance from the CW20 to the cw20burnmint address and ensure they get the tokens in return
function sendCw20Msg() {
    BASE64_MSG=$(echo -n "{"receive":{}}" | base64)
    export EXECUTED_MINT_JSON=`printf '{"send":{"contract":"%s","amount":"%s","msg":"%s"}}' $CW20_BURN "5" $BASE64_MSG` && echo $EXECUTED_MINT_JSON

    # Base cw20 contract
    TX=$($BINARY tx wasm execute "$CW20_ADDR" "$EXECUTED_MINT_JSON" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX
    # junod tx wasm execute "$CW20_ADDR" `printf '{"send":{"contract":"%s","amount":"5","msg":"e3JlZGVlbTp7fX0="}}' $BURN_ADDR` $JUNOD_COMMAND_ARGS
}

# get balance of the $KEY_ADDR
# 0 initially
$BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount
sendCw20Msg

# should not be 5
$BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount


# then you can continue to use your TF_CONTRACT for other applications :D