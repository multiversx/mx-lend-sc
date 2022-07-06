#Liquidate Notes

Deposit -> 2_000 USDC                           User: 2_000 LUSDC                                       Liquidity Pool: 2_000 USDC


Borrow: deposit 10 LEGLD (2000 USDC)   --->     User: (2_000 LUSDC) + 1_000 USDC + 1_000 BUSDC                Liquidity Pool: 1_000 USDC
                                                                                                                BorrowPosition {
                                                                                                                                timestamp,
                                                                                                                                lend_tokens {
                                                                                                                                    payment_lend_token_id: LEGLD,
                                                                                                                                    nonce,
                                                                                                                                    amount: 10
                                                                                                                                }
                                                                                                                                borrow_amount_in_tokens
                                                                                                                                payment_lend_token_id:LELGD
                                                                                                                            }
max_borrow = 2_000 * 0.7 = 1_400
borrowed = 1_000

Liquidate:

liquidator Send 1_000 USDC, receives 1_007 USDC in LEGLD

lend_amount_to_return = 


USD = 1
EGLD = 50

900 / 50 = 18






State for account: "0000000000000000aaddb418ccb49b5426d5c2504f526f7766580f6e45984e3b"
EGLD: 0

Storage: 
  pools_map.node_links: 0x0000000000000002
  pools_map.node_id
                   USDC-123456: 0x01
  pools_map.value: 0x555344432d313233343536
  0x706f6f6c5f616c6c6f7765642e6e6f64655f696400000000000000001ba0460af9324ac6df5b6ffb66be6df2547872c2f29ba4c2: 0x01
  pool_allowed.value: 0x0000000000000000ce29463110b6164dbb28dda28902586bf66e865e8c29c350
  pool_allowed.node_links: 0x0000000000000002
  pools_map.mapped
                  USDC-123456: 0x00000000000000001ba0460af9324ac6df5b6ffb66be6df2547872c2f29ba4c2
  pool_allowed.node_links: 0x0000000100000000
  liq_pool_template_address: 0x000000000000000083036503ca551673c09deec28df432a8d88debc7fa2ec91e
  pool_allowed.info: 0x00000002000000010000000200000002
  pools_map.mapped
                  WEGLD-123456: 0x0000000000000000ce29463110b6164dbb28dda28902586bf66e865e8c29c350
  pools_map.value: 0x5745474c442d313233343536
  pools_map.node_links: 0x0000000100000000
  0x706f6f6c5f616c6c6f7765642e6e6f64655f69640000000000000000ce29463110b6164dbb28dda28902586bf66e865e8c29c350: 0x02
  pools_map.node_id
                   WEGLD-123456: 0x02
  pools_map.info: 0x00000002000000010000000200000002
  pool_allowed.value: 0x00000000000000001ba0460af9324ac6df5b6ffb66be6df2547872c2f29ba4c2

State for account: "12771355e46cd47c71ed1721fd5319b383cca3a1f9fce3aa1c8cd3bd37af20d7"
EGLD: 100000000
ESDT Tokens:
  Token: LWEGLD-abcdef
      Nonce 1, balance: 0, attributes: 0x
  Token: USDC-123456
      Nonce 0, balance: 0, attributes: 0x

State for account: "0000000000000000d720a08b839a004c2e6386f5aecc19ec74807d1920cb6aeb"
EGLD: 0

Storage: 
  latest_price_feedWEGLDUSD: 0x36b0
  latest_price_feedUSDCUSD: 0x64

State for account: "00000000000000001ba0460af9324ac6df5b6ffb66be6df2547872c2f29ba4c2"
EGLD: 0
ESDT Tokens:
  Token: LUSDC-123456
      Nonce 0, balance: 0, attributes: 0x
      Nonce 1, balance: 0, attributes: 0x
  Token: USDC-123456
      Nonce 0, balance: 200000, attributes: 0x
  Token: LWEGLD-abcdef
      Nonce 1, balance: 286, attributes: 0x
  Token: BUSDC-123456
      Nonce 0, balance: 0, attributes: 0x
      Nonce 1, balance: 0, attributes: 0x

Storage: 
  borrow_position: 0x
  borrowed_amount: 0x
  deposit_position: 0x000000000000000000000003030d40
  pool_asset: 0x555344432d313233343536
  borrow_token: 0x42555344432d313233343536
  lend_token: 0x4c555344432d313233343536
  reserves: 0x030d40
  pool_params: 0x000000000000000402625a00000000043b9aca00000000042faf08000000000405f5e100
  priceAggregatorAddress: 0x0000000000000000d720a08b839a004c2e6386f5aecc19ec74807d1920cb6aeb
  liquidation_threshold: 0x29b92700

State for account: "2b32db6c2c0a6235fb1397e8225ea85e0f0e6e8c7b126d0016ccbde0e667151e"
EGLD: 100000000
ESDT Tokens:
  Token: LWEGLD-abcdef
      Nonce 1, balance: 0, attributes: 0x
  Token: BUSDC-123456
      Nonce 1, balance: 100000, attributes: 0x
  Token: WEGLD-123456
      Nonce 0, balance: 0, attributes: 0x
  Token: USDC-123456
      Nonce 0, balance: 100000, attributes: 0x
  Token: LUSDC-123456
      Nonce 1, balance: 200000, attributes: 0x

State for account: "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
EGLD: 0

State for account: "000000000000000083036503ca551673c09deec28df432a8d88debc7fa2ec91e"
EGLD: 0

State for account: "0000000000000000ce29463110b6164dbb28dda28902586bf66e865e8c29c350"
EGLD: 0
ESDT Tokens:
  Token: WEGLD-123456
      Nonce 0, balance: 1000, attributes: 0x
  Token: LWEGLD-abcdef
      Nonce 0, balance: 0, attributes: 0x
      Nonce 1, balance: 714, attributes: 0x
  Token: BWEGLD-abcdef
      Nonce 0, balance: 0, attributes: 0x

Storage: 
  borrow_token: 0x425745474c442d616263646566
  deposit_position: 0x000000000000000000000002011e
  reserves: 0x03e8
  pool_asset: 0x5745474c442d313233343536
  pool_params: 0x000000000000000402625a00000000043b9aca00000000042faf08000000000405f5e100
  priceAggregatorAddress: 0x0000000000000000d720a08b839a004c2e6386f5aecc19ec74807d1920cb6aeb
  liquidation_threshold: 0x29b92700
  lend_token: 0x4c5745474c442d616263646566