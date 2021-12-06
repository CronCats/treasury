use serde_json::json;

use workspaces::prelude::*;

const TREASURY_WASM_FILEPATH: &str = "./res/treasury.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(TREASURY_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await.unwrap();

    let outcome = worker
        .call(
            &contract,
            "new".to_string(),
            json!({})
            .to_string()
            .into_bytes(),
            None,
        )
        .await?;

    println!("new_default: {:#?}", outcome);

    let result = worker
        .view(
            contract.id().clone(),
            "version".to_string(),
            Vec::new(),
        )
        .await?;

    println!(
        "--------------\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    println!("Dev Account ID: {}", contract.id());

    Ok(())
}
