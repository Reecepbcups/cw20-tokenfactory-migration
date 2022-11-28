# cw20-burn

Deposit an old CW20 token and receive the new token-factory native token.

Begin:

- Set this contract as the token factory admin (for minting)
OR
- send some of the native tokens to this address

Next:

- Send the CW20 to this contract. In turn, it will burn the CW20 and mint/give you the new token-factory native token

End:

- You can now use the new token-factory native token as you wish
