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

if [ -z ${NEAR_ACCT+x} ]; then
  export NEAR_ACCT=weicat.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export TREASURY_ACCOUNT_ID=treasury.$NEAR_ACCT

# create all accounts
near create-account $TREASURY_ACCOUNT_ID --masterAccount $NEAR_ACCT

# Deploy all the contracts to their rightful places
near deploy --wasmFile ../res/treasury.wasm --accountId $TREASURY_ACCOUNT_ID --initFunction new --initArgs '{}'

echo "Setup Complete"