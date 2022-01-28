#!/bin/bash
# This file is used for starting a fresh set of all contracts & configs
set -e

if [ -d "res" ]; then
  echo ""
else
  mkdir res
fi

cd "`dirname $0`"

if [ -z "$KEEP_NAMES" ]; then
  export RUSTFLAGS='-C link-arg=-s'
else
  export RUSTFLAGS=''
fi

# build the things
cargo build --all --target wasm32-unknown-unknown --release
cp ../target/wasm32-unknown-unknown/release/*.wasm ../res/

# Uncomment the desired network
export NEAR_ENV=testnet
# export NEAR_ENV=mainnet
# export NEAR_ENV=guildnet
# export NEAR_ENV=betanet

export FACTORY=testnet
# export FACTORY=near
# export FACTORY=registrar

export MAX_GAS=300000000000000

if [ -z ${NEAR_ACCT+x} ]; then
  # export NEAR_ACCT=weicat.$FACTORY
  export NEAR_ACCT=vaultfactory.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export TREASURY_ACCOUNT_ID=treasury.$NEAR_ACCT
export CRON_ACCOUNT=manager_v1.croncat.$FACTORY

near deploy --wasmFile ../res/treasury.wasm --accountId $TREASURY_ACCOUNT_ID --force

# add/remove staking pools
# near call $TREASURY_ACCOUNT_ID add_staking_pool '{"pool_account_id": "node0"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID add_staking_pool '{"pool_account_id": "meta-pool.near", "liquid_unstake_function": "liquid_unstake", "yield_function": "harvest_meta", "withdraw_function": "withdraw_all"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID add_staking_pool '{"pool_account_id": "steak.poolv1.near"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID remove_staking_pool '{"pool_account_id": "hotones.pool.f863973.m0"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near view $TREASURY_ACCOUNT_ID get_delegations
# near view $TREASURY_ACCOUNT_ID get_config
# near view $TREASURY_ACCOUNT_ID get_info

# Triggers:
near view $TREASURY_ACCOUNT_ID needs_stake_rebalance
near view $TREASURY_ACCOUNT_ID has_delegation_to_withdraw

# deposit&stake
# near call $TREASURY_ACCOUNT_ID deposit_and_stake '{"pool_account_id": "meta-v2.pool.testnet"}' --accountId $TREASURY_ACCOUNT_ID --amount 13 --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID deposit_and_stake '{"pool_account_id": "hotones.pool.f863973.m0"}' --accountId $TREASURY_ACCOUNT_ID --amount 10 --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID deposit_and_stake '{"pool_account_id": "meta-pool.near", "amount": "10000000000000000000000000"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID deposit_and_stake '{"pool_account_id": "steak.poolv1.near", "amount": "10000000000000000000000000"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

# # update internal balance
# near call $TREASURY_ACCOUNT_ID get_staked_balance '{"pool_account_id": "meta-pool.near"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID get_staked_balance '{"pool_account_id": "steak.poolv1.near"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near view $TREASURY_ACCOUNT_ID get_delegations

# liquidunstake
# near call $TREASURY_ACCOUNT_ID liquid_unstake '{"pool_account_id": "meta-v2.pool.testnet", "amount": "5000000000000000000000000"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID liquid_unstake '{"pool_account_id": "meta-v2.pool.testnet"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

# unstake
# near call $TREASURY_ACCOUNT_ID unstake '{"pool_account_id": "meta-v2.pool.testnet"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID unstake '{"pool_account_id": "hotones.pool.f863973.m0", "amount": "5000000000000000000000000"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

# withdraw
# near call $TREASURY_ACCOUNT_ID withdraw '{"pool_account_id": "meta-v2.pool.testnet"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

# withdraw
# near call $TREASURY_ACCOUNT_ID yield_harvest '{"pool_account_id": "meta-v2.pool.testnet"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

# Croncat scheduler
# Staking: Auto-Stake Retrieve Balances
# CRONCAT_ARGS=`echo "{\"pool_account_id\": \"meta-pool.near\"}" | base64`
# # CRONCAT_ARGS=`echo "{\"pool_account_id\": \"hotones.pool.f863973.m0\"}" | base64`
# CRONCAT_FIXED_ARGS=`echo $CRONCAT_ARGS | tr -d '\r' | tr -d ' '`
# near call $CRON_ACCOUNT create_task '{"contract_id": "'$TREASURY_ACCOUNT_ID'","function_id": "get_staked_balance","cadence": "0 0 */1 * * *","recurring": true,"deposit": "0","gas": 120000000000000,"arguments": "'$CRONCAT_FIXED_ARGS'"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS --depositYocto 5000000000000000000000000
# near call $CRON_ACCOUNT remove_task '{"task_hash": "D4iuhttbR+wzfTOs/HP9SEQB9HFyCrzUeNl11Vp9X/U="}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS


echo "Staking Flows Complete"