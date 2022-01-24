# Contract ABI

```
{
  "viewMethods": [
    "version",
    "get_delegations",
    "has_delegation_to_withdraw",
    "needs_stake_rebalance",
    "get_approved_action_types",
    "has_timeout_actions",
    "get_ft_list",
    "ft_balances",
    "ft_balance_of",
    "get_nft_list",
    "nft_holdings",
    "nft_tokens",
  ],
  "changeMethods": [
    "update_settings",
    "add_staking_pool",
    "remove_staking_pool",
    "auto_stake",
    "deposit_and_stake",
    "get_staked_balance",
    "unstake",
    "withdraw",
    "liquid_unstake",
    "yield_harvest",
    "add_allowed_actions",
    "remove_allowed_action",
    "create_actions",
    "call_cadence_action",
    "call_timeout_actions",
    "ft_transfer",
    "store_ft_balance_of",
    "compute_ft_balances",
    "nft_transfer",
  ],
}
```