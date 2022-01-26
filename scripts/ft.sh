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
export META_TOKEN=meta-token.$FACTORY
export CHEDDAR_TOKEN=token-v3.cheddar.testnet

# near deploy --wasmFile ../res/treasury.wasm --accountId $TREASURY_ACCOUNT_ID --force

# near call $TREASURY_ACCOUNT_ID store_ft_balance_of '{"ft_account_id": "'$CHEDDAR_TOKEN'"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

# near call $TREASURY_ACCOUNT_ID compute_ft_balances '{"from_index": 0, "limit": 10}'

near view $TREASURY_ACCOUNT_ID get_ft_list

near view $TREASURY_ACCOUNT_ID ft_balances '{"from_index": "0", "limit": "10"}'

# NEAR_ENV=mainnet near view meta-token.near ft_balance_of '{"account_id": "croncat.sputnik-dao.near"}'
# near view $CHEDDAR_TOKEN ft_balance_of '{"account_id": "'$TREASURY_ACCOUNT_ID'"}'
# near view $TREASURY_ACCOUNT_ID ft_balance_of '{"account_id": "'$META_TOKEN'"}' --accountId $TREASURY_ACCOUNT_ID
# near view $TREASURY_ACCOUNT_ID ft_balance_of '{"account_id": "'$CHEDDAR_TOKEN'"}' --accountId $TREASURY_ACCOUNT_ID

near call $TREASURY_ACCOUNT_ID ft_transfer '{"ft_account_id": "'$CHEDDAR_TOKEN'", "to_account_id": "'$NEAR_ACCT'", "to_amount": "100000000000000000000000000"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS


echo "Token Flows Complete"