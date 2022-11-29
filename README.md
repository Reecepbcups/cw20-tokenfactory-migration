# cw20-burn

Deposit an old CW20 token and receive the new token-factory native token.

## cw20_burn_balance

Begin:

- Set this contract as the token factory admin (for minting)
OR
- send some of the native tokens to this address

## cw20_burn_mint

Begin:

- Set the contract as the minter  / admin of the token factory denom.

## Next

- The user sends their CW20 to this contract. In turn, it will burn the CW20 and mint/give you the new token-factory native token

## Finally

- You can now use the new token-factory native token as you wish
