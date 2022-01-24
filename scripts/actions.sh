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
  export NEAR_ACCT=weicat.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export TREASURY_ACCOUNT_ID=treasury.$NEAR_ACCT

# add/remove permissions
near call treasury.testnet add_allowed_actions '{"actions": [{ "token_id": "wrap.near", "receiver_id": "you.near", "amount": "100000000000000000000000000", "msg": "" }]}' --accountId treasury.testnet
# near call treasury.testnet remove_allowed_action '{"actions": [{ "token_id": "wrap.near", "receiver_id": "you.near", "amount": "100000000000000000000000000", "msg": "" }]}' --accountId treasury.testnet

# create actions
EPOCH=date +%s
ACTION_TIMEOUT_1min=echo "$(($EPOCH * 1000 + 60000))"
ACTION_TIMEOUT_12hr=echo "$(($EPOCH * 1000 + 43200000))"
ACTION_TRANSFER='{"priority": 1, "timeout": "'$ACTION_TIMEOUT_1min'", "payload": { "token_id": "wrap.testnet", "receiver_id": "you.testnet", "amount": "1000000000000000000000000", "msg": "transfer wrapped near"}}'
ACTION_BUDGET_TIMEOUT='{"priority": 0, "timeout": "'$ACTION_TIMEOUT_12hr'", "payload": { "token_id": "wrap.testnet", "receiver_id": "you.testnet", "amount": "1000000000000000000000000", "msg": "transfer wrapped near"}}'
ACTION_BUDGET_CADENCE='{"priority": 1, "cadence": "0 3 * * * *", "payload": { "token_id": "wrap.testnet", "receiver_id": "you.testnet", "amount_percentile": "5", "msg": "transfer percentage of near"}}'
near call $TREASURY_ACCOUNT_ID create_actions '{"actions": ['$ACTION_TRANSFER']}' --accountId $TREASURY_ACCOUNT_ID
near call $TREASURY_ACCOUNT_ID create_actions '{"actions": ['$ACTION_BUDGET_TIMEOUT']}' --accountId $TREASURY_ACCOUNT_ID
near call $TREASURY_ACCOUNT_ID create_actions '{"actions": ['$ACTION_BUDGET_TIMEOUT','$ACTION_BUDGET_CADENCE']}' --accountId $TREASURY_ACCOUNT_ID

# See everthing thats scheduled
#TBD

# call cadence


# call timeout


echo "Actions Complete"