#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]

use serde_json::json;

use workspaces::prelude::*;

// Core runtime contracts
const TREASURY_WASM: &str = "../res/treasury.wasm";
const FUNGIBLE_TOKEN_WASM: &str = "../res/fungible_token.wasm";
const NON_FUNGIBLE_TOKEN_WASM: &str = "../res/non_fungible_token.wasm";

// Optionals
// const DEFI_WASM: &str = "../res/defi.wasm";
// const APPROVAL_RECEIVER_WASM: &str = "../res/approval_receiver.wasm";
// const TOKEN_RECEIVER_WASM: &str = "../res/token_receiver.wasm";

#[tokio::test]
pub async fn deploy_main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let treasury_file = std::fs::read(TREASURY_WASM)?;
    let ft_file = std::fs::read(FUNGIBLE_TOKEN_WASM)?;
    let nft_file = std::fs::read(NON_FUNGIBLE_TOKEN_WASM)?;

    let treasury = worker.dev_deploy(treasury_file).await.expect("Treasury deploy failed");
    let ft = worker.dev_deploy(ft_file).await.expect();
    let nft = worker.dev_deploy(nft_file).await.unwrap();

    println!("Treasury ID: {}", treasury.id());
    println!("FT ID: {}", ft.id());
    println!("NFT ID: {}", nft.id());

    let outcome = worker
        .call(
            &treasury,
            "new".to_string(),
            json!({}).to_string().into_bytes(),
            None,
        )
        .await?;

    println!("treasury_default: {:#?}", outcome);

    let result = worker
        .view(treasury.id().clone(), "version".to_string(), Vec::new())
        .await?;
    let version = serde_json::to_string_pretty(&result).expect("No version method");
    assert!(version.len() > 3, "No version found");

    println!(
        "--------------\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    Ok(())
}
