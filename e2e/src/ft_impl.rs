use near_sdk::{
    serde::{Deserialize, Serialize},
    serde_json::json,
};
use workspaces::*;

const EXPECTED_FT_METADATA: &str = r#"{
    "spec": "ft-1.0.0",
    "name": "Treasury Demo Token",
    "symbol": "TREASURE",
    "decimals": 18
}"#;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
struct FtMetadata {
    spec: String,
    name: String,
    symbol: String,
    decimals: u8,
}

fn expected() -> FtMetadata {
    serde_json::from_str(EXPECTED_FT_METADATA).unwrap()
}

pub async fn init(worker: Worker<impl Network>, contract: &Contract) -> anyhow::Result<()> {
    worker
        .call(
            contract,
            "new".to_string(),
            json!({
                "owner_id": contract.id().to_string(),
                "total_supply": "1000000000000000",
                "metadata": {
                    "spec": "ft-1.0.0",
                    "name": "Treasury Demo Token",
                    "symbol": "TREASURE",
                    "decimals": 18
                }
            })
            .to_string()
            .into_bytes(),
            None,
        )
        .await?;

    let result = worker
        .view(contract.id().clone(), "ft_metadata".to_string(), Vec::new())
        .await?;
    let metadata: FtMetadata = serde_json::from_str(&result).expect("No ft_metadata method");
    // println!("metadata: {:#?}", metadata);
    assert_eq!(metadata, expected(), "FT Metadata wrong");

    Ok(())
}

// // TODO: Mint to user
// pub async fn mint_to_account(
//     worker: Worker<impl Network>,
//     account: Worker<impl Account>,
//     contract_id: &AccountId,
// ) -> anyhow::Result<()> {
//     let outcome = account
//         .call(
//             &worker,
//             contract_id.to_string(),
//             "storage_deposit".into(),
//             None,
//         )
//         .with_args(json!({}).to_string().into_bytes())
//         .transact()
//         .await?;
//     println!("MINT outcome: {:#?}", outcome);

//     let result = account
//         .view(contract.id().clone(), "ft_metadata".to_string(), Vec::new())
//         .await?;
//     let metadata: FtMetadata = serde_json::from_str(&result).expect("No ft_metadata method");
//     // println!("metadata: {:#?}", metadata);
//     assert_eq!(metadata, expected(), "FT Metadata wrong");

//     Ok(())
// }

// TODO: Transfer to treasury
