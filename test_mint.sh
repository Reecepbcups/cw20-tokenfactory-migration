# Mints tokenfactory tokens -> the contract (as the admin) to then distribute to users & burn the CW20 for

KEY="juno1"
KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl" # test_node.sh
CHAINID="juno-t1"
MONIKER="localjuno"
KEYALGO="secp256k1"
KEYRING="test" # export juno_KEYRING="TEST"
LOGLjunoL="info"
TRACE="" # "--trace"
junod config keyring-backend $KEYRING
junod config chain-id $CHAINID
junod config output "json"
export JUNOD_NODE="http://localhost:26657"
export JUNOD_COMMAND_ARGS="--gas 5000000 --gas-prices="0ujuno" -y --from $KEY --broadcast-mode block --output json --chain-id juno-t1"
# junod status

junod tx tokenfactory create-denom test $JUNOD_COMMAND_ARGS
# junod q tokenfactory denoms-from-creator juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl
TOKEN_FACTORY_DENOM="factory/juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl/test"

TX20=$(junod tx wasm store cw20_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash')
TXBURN=$(junod tx wasm store artifacts/cw20_burn_mint.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash')
CW20_CODEID=1
BURN_CODEID=2

CW20_TX_INIT=$(junod tx wasm instantiate "1" '{"name": "test","symbol":"symb","decimals":6,"initial_balances":[{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","amount":"100"}]}' --label "juno-cw20" $JUNOD_COMMAND_ARGS -y --admin $KEY_ADDR | jq -r '.txhash') && echo $CW20_TX_INIT
CW20_ADDR=$(junod query tx $CW20_TX_INIT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "$CW20_ADDR"
# export CW20_ADDR=juno14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9skjuwg8
# junod q wasm contract-state smart $CW20_ADDR '{"balance":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"}}'

CW20_BURN=$(junod tx wasm instantiate "2" '{"cw20_address":"juno14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9skjuwg8","tf_denom":"factory/juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl/test"}' --label "juno-cw20burn" $JUNOD_COMMAND_ARGS -y --admin $KEY_ADDR | jq -r '.txhash') && echo $CW20_BURN
BURN_ADDR=$(junod query tx $CW20_BURN --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "$BURN_ADDR"
# export BURN_ADDR=juno1aakfpghcanxtc45gpqlx8j3rq0zcpyf49qmhm9mdjrfx036h4z5squu0w2


# Change the admin of the tokenfactory denom to the burn address itself
junod tx tokenfactory change-admin $TOKEN_FACTORY_DENOM $BURN_ADDR $JUNOD_COMMAND_ARGS

# No longer do this since the contract can mint itself
# junod tx tokenfactory mint 1000factory/juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl/test $JUNOD_COMMAND_ARGS
# junod tx bank send $KEY_ADDR $BURN_ADDR 1000factory/juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl/test $JUNOD_COMMAND_ARGS
# junod q bank balances $BURN_ADDR

# === Actual logic time ===

function sendCw20Msg() {
    BASE64_MSG=$(echo -n "{"receive":{}}" | base64)
    export EXECUTED_MINT_JSON=`printf '{"send":{"contract":"%s","amount":"%s","msg":"%s"}}' $BURN_ADDR "5" $BASE64_MSG`

    TX=$(junod tx wasm execute "$CW20_ADDR" "$EXECUTED_MINT_JSON" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX
    # junod tx wasm execute "$CW20_ADDR" `printf '{"send":{"contract":"%s","amount":"5","msg":"e3JlZGVlbTp7fX0="}}' $BURN_ADDR` $JUNOD_COMMAND_ARGS
}

function returnAdminToUser() {
    # Calls the burn address 
    TX=$(junod tx wasm execute "$BURN_ADDR" '{"transfer_back_admin":{}}' $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX    
}

# junod tx wasm execute $CW20_ADDR '{"send":{"contract":"","amount":"100"}}' $JUNOD_COMMAND_ARGS

sendCw20Msg

echo -e "\n==== No balance, then 5 bal"
junod q bank balances $BURN_ADDR # no balance
junod q bank balances $KEY_ADDR # gets minted 5

echo -e"\n==== 1 then 0 denoms"
returnAdminToUser
junod q tokenfactory denoms-from-creator $KEY_ADDR  # should be 1
junod q tokenfactory denoms-from-creator $BURN_ADDR # should be 0