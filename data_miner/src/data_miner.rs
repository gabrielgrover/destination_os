use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait DataMiner: Send + Sync {
    async fn mine(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn name(&self) -> String;
}
