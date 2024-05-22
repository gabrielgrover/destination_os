use crate::DataMiner;
use anyhow::anyhow;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::{future::try_join_all, FutureExt, StreamExt};
use itertools::*;
use std::error::Error;

pub struct WanderingBlondeBlog {
    name: String,
}

impl WanderingBlondeBlog {
    pub fn new() -> Self {
        WanderingBlondeBlog {
            name: "wandering_blonde_blog".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DataMiner for WanderingBlondeBlog {
    async fn mine(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
const BASE_URL: &str = "https://thewanderingblonde.com";
//2015/page/2/
async fn wandering_blonde_scrape() {
    //
}
