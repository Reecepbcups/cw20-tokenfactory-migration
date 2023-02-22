# CW20 Migrations to TokenFactory

Deposit/Send in an old CW20 token and receive the new token-factory native token :D

Modes: Mint from [Core Middlware Contract](https://github.com/Reecepbcups/tokenfactory-core-contract) or send from this contract's bank balance

## Mint




## Balance



---

This contract will accept token-factory denominations from a user (ex: admin of the denom) and allow for liquid tokens to convert via balances.


Steps:
- send some of the native tokens to this address
- Launch a frontend to interact with said contract (see test_balance.sh `sendCw20Msg` for example on how it works)
- The user sends their CW20 to this contract. In turn, it will burn the CW20 and mint/give you the new token-factory native token

## cw20_burn_mint

This contract mints a token from the  and in return burns the CW20 asset for the user. 

Begin:

- Initialize the core-contract
- The user sends their CW20 to this contract. In turn, it will burn the CW20 and mint/give you the new token-factory native token

## Finally

- You can now use the new token-factory native token as you wish

---

## Other Ideas

Will work on these after Juno v12 testnet launch

<https://hackmd.io/@reecepbcups/cw20-to-tokenfactory>

- CW20 standard contract with a migrate function (bankSend the factory denom to the contract, upload new CW20-tf-migrate if total CW20 supply <= held tokenfactory, convert all to the new denom)
^ Will we hit a gas limit issue? since juno is only 10m per block

- IBC convert denoms, send to null address? since bank doesn't have burn

- DAODAO native converts with VoteModule / CW20 wrappers