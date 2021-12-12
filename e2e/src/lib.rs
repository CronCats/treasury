#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]
use serde_json::json;
// use workspaces::prelude::*;
use workspaces::{Contract, Network, Worker};

// Priority ordered mods
mod utils;
// mod init;

// Core runtime contracts
const TREASURY_WASM: &str = "../res/treasury.wasm";
const FUNGIBLE_TOKEN_WASM: &str = "../res/fungible_token.wasm";
const NON_FUNGIBLE_TOKEN_WASM: &str = "../res/non_fungible_token.wasm";

// Optionals
// const DEFI_WASM: &str = "../res/defi.wasm";
// const APPROVAL_RECEIVER_WASM: &str = "../res/approval_receiver.wasm";
// const TOKEN_RECEIVER_WASM: &str = "../res/token_receiver.wasm";

async fn treasury_init(worker: Worker<impl Network>, contract: &Contract) -> anyhow::Result<()> {
    let outcome = worker
        .call(
            contract,
            "new".to_string(),
            json!({}).to_string().into_bytes(),
            None,
        )
        .await?;

    println!("treasury_default: {:#?}", outcome);

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

    let treasury = utils::dev_deploy(worker.clone(), TREASURY_WASM).await.expect("Treasury deploy failed");
    let ft = utils::dev_deploy(worker.clone(), FUNGIBLE_TOKEN_WASM).await.expect("Fungible Token deploy failed");
    let nft = utils::dev_deploy(worker.clone(), NON_FUNGIBLE_TOKEN_WASM).await.expect("Non-Fungible Token deploy failed");

    println!("Treasury ID: {}", treasury.id());
    println!("FT ID: {}", ft.id());
    println!("NFT ID: {}", nft.id());

    // initialize each contract with basics:
    treasury_init(worker.clone(), &treasury).await?;

    // let wasm = std::fs::read(TREASURY_WASM_FILEPATH)?;
    // let contract = worker.dev_deploy(wasm).await.unwrap();

    // let outcome = worker
    //     .call(
    //         &contract,
    //         "new".to_string(),
    //         json!({})
    //         .to_string()
    //         .into_bytes(),
    //         None,
    //     )
    //     .await?;

    // println!("new_default: {:#?}", outcome);

    // let result = worker
    //     .view(
    //         contract.id().clone(),
    //         "version".to_string(),
    //         Vec::new(),
    //     )
    //     .await?;

    // println!(
    //     "--------------\n{}",
    //     serde_json::to_string_pretty(&result).unwrap()
    // );

    // println!("Dev Account ID: {}", contract.id());

    Ok(())
}
