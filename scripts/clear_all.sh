#!/bin/bash
# Uncomment the desired network
export NEAR_ENV=testnet
# export NEAR_ENV=mainnet
# export NEAR_ENV=guildnet
# export NEAR_ENV=betanet

export FACTORY=testnet
# export FACTORY=near
# export FACTORY=registrar

if [ -z ${NEAR_ACCT+x} ]; then
  # export NEAR_ACCT=weicat.$FACTORY
  export NEAR_ACCT=vaultfactory.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export TREASURY_ACCOUNT_ID=treasury.$NEAR_ACCT

# clear and recreate all accounts
near delete $TREASURY_ACCOUNT_ID $NEAR_ACCT

echo "Clear Complete"