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
export CRONCAT_MANAGER_ID=manager_v1.croncat.$FACTORY
# export DAO_ID=croncat.sputnikv2.$FACTORY
export DAO_ID=croncat.sputnik-dao.$FACTORY

# near deploy --wasmFile ../res/treasury.wasm --accountId $TREASURY_ACCOUNT_ID --force

# owner stuff
near call $TREASURY_ACCOUNT_ID update_settings '{"croncat_id": "'$CRONCAT_MANAGER_ID'"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID update_settings '{"owner_id": "'$DAO_ID'"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS

near call $TREASURY_ACCOUNT_ID add_payable_account '{"account_id": "'$NEAR_ACCT'"}' --accountId $TREASURY_ACCOUNT_ID --depositYocto 1 --gas $MAX_GAS
near call $TREASURY_ACCOUNT_ID add_payable_account '{"account_id": "'$DAO_ID'"}' --accountId $TREASURY_ACCOUNT_ID --depositYocto 1 --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID add_payable_account '{"account_id": "ion.testnet"}' --accountId $TREASURY_ACCOUNT_ID --depositYocto 1 --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID remove_payable_account '{"account_id": "ion.testnet"}' --accountId $TREASURY_ACCOUNT_ID --depositYocto 1 --gas $MAX_GAS
# near call $TREASURY_ACCOUNT_ID transfer '{"receiver_id": "cron.testnet",  "amount": "10000000000000000000000000"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
# Should fail:
# near call $TREASURY_ACCOUNT_ID transfer '{"receiver_id": "ion.testnet",  "amount": "10000000000000000000000000"}' --accountId $TREASURY_ACCOUNT_ID --gas $MAX_GAS
near view $TREASURY_ACCOUNT_ID get_accounts_payable

echo "Bootstrap Complete"