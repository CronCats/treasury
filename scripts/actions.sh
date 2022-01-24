#!/bin/bash
# set -e

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
export CRONCAT_MANAGER_ID=manager_v1.croncat.$FACTORY

# add/remove permissions
# near call $TREASURY_ACCOUNT_ID add_allowed_actions '{"actions": [{ "Transfer": { "token_id": "wrap.near", "receiver_id": "you.near", "amount": "100000000000000000000000000", "msg": "" }}]}' --accountId $TREASURY_ACCOUNT_ID
# near call $TREASURY_ACCOUNT_ID add_allowed_actions '{"actions": [{ "Budget": { "token_id": "wrap.near", "receiver_id": "you.near", "amount": "100000000000000000000000000", "msg": "" }}]}' --accountId $TREASURY_ACCOUNT_ID
# near call $TREASURY_ACCOUNT_ID remove_allowed_action '{"actions": [{ "token_id": "wrap.near", "receiver_id": "you.near", "amount": "100000000000000000000000000", "msg": "" }]}' --accountId $TREASURY_ACCOUNT_ID
# near view $TREASURY_ACCOUNT_ID get_approved_action_types
# near view $TREASURY_ACCOUNT_ID is_allowed_action '{ "token_id": "wrap.near", "receiver_id": "you.near", "amount": "100000000000000000000000000", "msg": "" }' --accountId $TREASURY_ACCOUNT_ID

# create actions
# TODO: Missing actions: Swap, FunctionCall
EPOCH=$(date +"%s")
ACTION_TIMEOUT_1min=$((($EPOCH * 1000 + 60000) * 1000000))
ACTION_TIMEOUT_12hr=$((($EPOCH * 1000 + 43200000) * 1000000))
# ACTION_TRANSFER='{"actions": [{"Transfer": {"priority": 1, "timeout": "'$ACTION_TIMEOUT_1min'", "payload": { "token_id": "wrap.testnet", "receiver_id": "you.testnet", "amount": "1000000000000000000000000", "msg": "transfer wrapped near"}}}]}'
# ACTION_BUDGET_TIMEOUT='{"priority": 0, "timeout": "'$ACTION_TIMEOUT_12hr'", "payload": { "token_id": "wrap.testnet", "receiver_id": "you.testnet", "amount": "1000000000000000000000000", "msg": "transfer wrapped near"}}'
# ACTION_BUDGET_CADENCE='{"priority": 1, "cadence": "0 3 * * * *", "payload": { "token_id": "wrap.testnet", "receiver_id": "you.testnet", "amount_percentile": "5", "msg": "transfer percentage of near"}}'
# near call $TREASURY_ACCOUNT_ID create_actions $ACTION_TRANSFER --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID create_actions '{"actions": [{"priority": 1, "timeout": "'$ACTION_TIMEOUT_1min'", "payload": { "Transfer": { "token_id": "wrap.testnet", "receiver_id": "you.testnet", "amount": "1000000000000000000000000", "msg": "transfer wrapped near"}}}]}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# # near call $TREASURY_ACCOUNT_ID create_actions '{"actions": ['$ACTION_BUDGET_TIMEOUT']}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# # near call $TREASURY_ACCOUNT_ID create_actions '{"actions": ['$ACTION_BUDGET_TIMEOUT','$ACTION_BUDGET_CADENCE']}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

# See everthing thats scheduled
#TBD

# Remove Actions
#TBD

# call cadence
# near call $TREASURY_ACCOUNT_ID call_cadence_action '{"cadence": "0 3 * * * *"}' --accountId $CRONCAT_MANAGER_ID --gas $MAX_GAS

# call timeout
# near call $TREASURY_ACCOUNT_ID call_timeout_actions --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

echo "Actions Complete"