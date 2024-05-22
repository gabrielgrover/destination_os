use data_miner::miners;
use dotenvy::dotenv;
use futures_util::{future::try_join_all, FutureExt};
use serde::{Deserialize, Serialize};
use std::{env, io::Write, path::PathBuf};

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found.");
    let base_store_path = env::var("STORE_PATH").expect("STORE_PATH env var not fount.");
    let data_miners = miners();
    let mining_yields = try_join_all(
        data_miners
            .iter()
            .map(|m| {
                m.mine()
                    .then(move |result| async { result.map(|data| (m.name(), data)) })
            })
            .collect::<Vec<_>>(),
    )
    .await
    .unwrap();

    for mining_yield in mining_yields.into_iter() {
        let path = PathBuf::from(&base_store_path);
        store_yield(mining_yield, path).expect("Failed to store mining yield");
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MiningYield {
    pub name: String,
    pub data: Vec<u8>,
}

fn store_yield(y: (String, Vec<u8>), mut base_path: PathBuf) -> anyhow::Result<()> {
    let m = MiningYield {
        name: y.0,
        data: y.1,
    };
    base_path.push(format!("{}.json", &m.name));

    let data_str = serde_json::to_vec(&m)?;
    let mut doc_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(base_path)?;

    doc_file.write_all(&data_str)?;
    doc_file.flush()?;

    Ok(())
}
