use data_miner::miners;
use futures_util::{future::try_join_all, FutureExt};
use kv::{Config, Store};

#[tokio::main]
async fn main() {
    let config = Config::new("./mined_data/yields");
    let store = Store::new(config).unwrap();
    let bucket = store
        .bucket::<String, Vec<u8>>(Some("mining_yields_0"))
        .unwrap();

    // let data = bucket
    //     .get(&"dangerous_business_blog".to_string())
    //     .unwrap()
    //     .unwrap();

    // println!("{:?}", std::str::from_utf8(&data).unwrap());

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

    for (miner_name, data) in mining_yields.into_iter() {
        bucket.set(&miner_name, &data).unwrap();
    }
}
