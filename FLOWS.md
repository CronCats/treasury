# Features & Flows

The following guide shows how this contract can be used, and serves as documentation on how to go through each feature.

## General

#### Initialization

```bash
near call treasury.testnet new --accountId treasury.testnet
```

----

## Staking

#### Add Staking Pool

```bash
near call treasury.testnet add_staking_pool '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
```

#### Remove Staking Pool

```bash
near call treasury.testnet remove_staking_pool '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
```

#### Deposit & Stake

If there are specific staking pools you would like to use, this will be the shortest path:

```bash
near call treasury.testnet deposit_and_stake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
```

#### Auto Stake

NOTE: While this shows how to call the stake rebalance checks, it's intended to be called directly via [Croncat](https://cron.cat)

```bash
near call treasury.testnet auto_stake --accountId manager_v1.croncat.testnet
```

#### Unstake

```bash
near call treasury.testnet unstake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
```

#### Withdraw

```bash
near call treasury.testnet withdraw '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
```

#### Liquid Unstake

```bash
near call treasury.testnet liquid_unstake '{"pool_account_id": "steak.factory.testnet", "amount": "100000000000000000000000000"}' --accountId treasury.testnet
```

#### Staking Yield Harvest

NOTE: While this capability is possible, it's highly experimental. Use with caution.

```bash
near call treasury.testnet yield_harvest '{"pool_account_id": "steak.factory.testnet"}' --accountId treasury.testnet
```

----

## Actions

#### Add Allowed Action

```bash
near call treasury.testnet add_allowed_actions '{"actions": [{ "token_id": "wrap.near", "receiver_id": "you.near", "amount": "1", "msg": "" }]}' --accountId treasury.testnet
```

#### Remove Allowed Action

```bash
near call treasury.testnet remove_allowed_action '{"token_id": "wrap.near", "receiver_id": "you.near", "amount": "1", "msg": ""}' --accountId treasury.testnet
```

#### Create Actions

```bash
near call treasury.testnet create_actions '{"actions": [{ ...Action... }]}' --accountId treasury.testnet
```

**Example Action Payload**

Each action specifies when it should be called, by either a timeout or cadence.

* Cadence is a recurring cron-spec call
* Timeout allows for a one time call, after some time has occurred
* Priority is either a 1 or 0, allowing basic ordering

_NOTE: Only specify cadence or timeout_

```json
{
  "priority": 1,
  "timeout": "43200000",
  "cadence": "0 0 * * * *",
  "payload": {...ActionType...},
}
```

Full example:

```json
{
  "priority": 1,
  "timeout": "43200000",
  "cadence": "0 0 * * * *",
  "payload": {
    "token_id": "wrap.testnet",
    "receiver_id": "you.testnet",
    "amount": "1000000000000000000000000",
    "msg": "transfer wrapped near",
  },
}
```

**Example Action Types Payloads**

##### Transfer

```json
{
  "token_id": "wrap.testnet",
  "receiver_id": "you.testnet",
  "amount": "1000000000000000000000000",
  "msg": "transfer wrapped near",
}
```

##### Budget

Budget has two ways of payout specification.

1. Whole payment amount - an amount of token or near.
2. Percentage payment amount - Will take the current account balance and calculate the percent amount to be paid at time of execution. Only works for near currently.

```json
{
  "token_id": "wrap.testnet",
  "receiver_id": "you.testnet",
  "amount": "1000000000000000000000000",
  "msg": "transfer wrapped near",
}
```

```json
{
  "receiver_id": "you.testnet",
  "amount_percentile": "5",
  "msg": "transfer percentage of near",
}
```

##### Swap

```json
{
  "contract_id": "v2.ref-finance.near",
  "pool_id": 79,
  "token_in": "token.v2.ref-finance.near",
  "token_out": "wrap.near",
  "amount_in": "142445118507604278183",
  "min_amount_out": "33286939953575500000000000",
}
```

##### Harvest

```json
{
  "contract_id": "",
  "method_name": "",
  "args": "",
  "deposit": "",
  "gas": "",
}
```

##### FunctionCall

```json
{
  "receiver_id": "",
  "actions": [
    {
      "method_name": "",
      "args": "",
      "deposit": "",
      "gas": "",
    },
  ],
}
```

#### Remove Actions

```bash
near call treasury.testnet remove_actions '{"action": [{ ...Action... }]}' --accountId treasury.testnet
```

#### Cadence Action

NOTE: While this shows how to call the cadence action, it's intended to be called directly via [Croncat](https://cron.cat)

```bash
near call treasury.testnet call_cadence_action '{"cadence": "0 0 * * * *"}' --accountId manager_v1.croncat.testnet
```

#### Timeout Actions

NOTE: While this shows how to call the timeout actions, it's intended to be called directly via [Croncat](https://cron.cat) triggers.

```bash
near call treasury.testnet call_timeout_actions --accountId manager_v1.croncat.testnet
```

----

## Fungible Tokens

#### Token Transfer

```bash
near call treasury.testnet ft_transfer '{"ft_account_id": "wrap.testnet", "to_account_id": "user.account.testnet", "to_amount": "100000000000000000000000000000000"}' --accountId treasury.testnet
```

#### Store FT Balances

```bash
near call treasury.testnet store_ft_balance_of '{"ft_account_id": "wrap.testnet"}' --accountId treasury.testnet
```

TBD: `near call treasury.testnet compute_ft_balances '{"from_index": 0, "limit": 10}'`

----

## Non-Fungible Tokens

#### NFT Transfer

```bash
near call treasury.testnet nft_transfer '{"nft_account_id": "image.testnet", "to_account_id": "user.account.testnet", "to_token_id": "100000000000000000000000000000000"}' --accountId treasury.testnet
```

----

## Ownership

#### Update Settings

```bash
near call treasury.testnet nft_transfer '{ SEE EXAMPLES BELOW }' --accountId treasury.testnet
```

**Example Settings Payloads**

```json
{
  "owner_id": "owner.treasury.testnet",
}
```

```json
{
  "croncat_id": "manager_v1.croncat.testnet",
}
```

```json
{
  "stake_threshold": {
    "denominator": 100,
    "liquid": 30,
    "staked": 70,
    "deviation": 5,
    "extreme_deviation": 15,
    "eval_period": 43200000,
    "eval_cadence": "0 0 * * * *"
  },
}
```
