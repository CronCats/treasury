#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]
use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{Contract, Network, Worker};

// Priority ordered mods
mod utils;
// mod init;
mod ft_impl;

// Core runtime contracts
const TREASURY_WASM: &str = "../res/treasury.wasm";
const FUNGIBLE_TOKEN_WASM: &str = "../res/fungible_token.wasm";
const NON_FUNGIBLE_TOKEN_WASM: &str = "../res/non_fungible_token.wasm";

async fn treasury_init(worker: Worker<impl Network>, contract: &Contract) -> anyhow::Result<()> {
    worker
        .call(
            contract,
            "new".to_string(),
            json!({}).to_string().into_bytes(),
            None,
        )
        .await?;

    let result = worker
        .view(contract.id().clone(), "version".to_string(), Vec::new())
        .await?;
    let version = serde_json::to_string_pretty(&result).expect("No version method");
    assert!(version.len() > 3, "No version found");

    Ok(())
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let agent = worker.dev_create().await?;
    println!("AGENT: {}", agent.id());

    let treasury = utils::dev_deploy(worker.clone(), TREASURY_WASM)
        .await
        .expect("Treasury deploy failed");
    let ft = utils::dev_deploy(worker.clone(), FUNGIBLE_TOKEN_WASM)
        .await
        .expect("Fungible Token deploy failed");
    let nft = utils::dev_deploy(worker.clone(), NON_FUNGIBLE_TOKEN_WASM)
        .await
        .expect("Non-Fungible Token deploy failed");

    println!("Treasury ID: {}", treasury.id());
    println!("FT ID: {}", ft.id());
    println!("NFT ID: {}", nft.id());

    // initialize each contract with basics:
    treasury_init(worker.clone(), &treasury).await?;
    ft_impl::init(worker.clone(), &ft).await?;
    // ft_impl::mint_to_account(worker.clone(), agent.clone(), ft.id()).await?;

    agent
        .call(&worker, ft.id().clone(), "storage_deposit".into())
        .with_args(json!({}).to_string().into_bytes())
        .with_deposit(1250000000000000000000)
        .transact()
        .await?;

    let bal1 = worker
        .view(
            ft.id().clone(),
            "ft_balance_of".to_string(),
            json!({"account_id":agent.id().to_string()})
                .to_string()
                .into_bytes(),
        )
        .await?;
    let balance_1: U128 = serde_json::from_str(&bal1).expect("No result method");
    assert_eq!(balance_1.0, 0, "Invalid FT Balance");

    ft.call(&worker, "ft_transfer".into())
        .with_args(
            json!({
                "receiver_id": agent.id().to_string(),
                "amount": "1000",
            })
            .to_string()
            .into_bytes(),
        )
        .with_deposit(1)
        .transact()
        .await?;

    let bal2 = worker
        .view(
            ft.id().clone(),
            "ft_balance_of".to_string(),
            json!({"account_id":agent.id().to_string()})
                .to_string()
                .into_bytes(),
        )
        .await?;
    let balance_2: U128 = serde_json::from_str(&bal2).expect("No result method");
    assert_eq!(balance_2.0, 1000, "Invalid FT Balance");

    Ok(())
}
