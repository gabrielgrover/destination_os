use crate::DataMiner;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::{future::try_join_all, StreamExt};
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
        let content = wandering_blonde_scrape().await?;
        println!("{} content size {} bytes", self.name, content.len());

        Ok(content)
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

const BASE_URL: &str = "https://thewanderingblonde.com";

async fn wandering_blonde_scrape() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .headless_mode(chromiumoxide::browser::HeadlessMode::New)
            .build()?,
    )
    .await?;
    let handle = tokio::task::spawn(async move {
        loop {
            if let Some(_) = handler.next().await {
                continue;
            }

            break;
        }
    });

    let urls = get_post_urls(&browser).await?;
    let all_content = get_all_blog_content(urls, &browser).await?;

    browser.close().await?;
    handle.await?;

    Ok(all_content)
}

async fn get_all_blog_content(urls: Vec<String>, browser: &Browser) -> anyhow::Result<Vec<u8>> {
    let pages = try_join_all(urls.into_iter().map(|url| browser.new_page(url))).await?;
    let mut tasks = vec![];

    for page in pages.into_iter() {
        let maybe_page_url = page.url().await?;

        let task = async move {
            if let Some(page_url) = &maybe_page_url {
                println!("Scraping wandering_blonde_blog url: {}", page_url);
            }

            let p_tags = page.find_elements("p").await?;
            let mut p_tag_tasks = vec![];

            for p_tag in p_tags.into_iter() {
                let p_tag_task = async move {
                    if let Some(text) = p_tag.inner_text().await? {
                        Ok::<String, anyhow::Error>(text)
                    } else {
                        Ok(String::new())
                    }
                };

                p_tag_tasks.push(p_tag_task)
            }

            let content: Vec<u8> = try_join_all(p_tag_tasks).await?.join(" ").into_bytes();

            if let Some(page_url) = &maybe_page_url {
                println!(
                    "Scraping wandering_blonde_blog url: {} successful",
                    page_url
                );
            }

            page.close().await?;

            Ok::<Vec<u8>, anyhow::Error>(content)
        };

        tasks.push(task);
    }

    let all_content: Vec<u8> = try_join_all(tasks).await?.into_iter().flatten().collect();

    Ok(all_content)
}

async fn get_post_urls(browser: &Browser) -> anyhow::Result<Vec<String>> {
    let archive_tuples = vec![
        (2015, 5),
        (2016, 4),
        (2017, 3),
        (2018, 2),
        (2019, 3),
        (2020, 2),
    ];

    let mut tasks = vec![];

    for (year, page_count) in archive_tuples.into_iter() {
        for page in 1..page_count + 1 {
            let url = format!("{}/{}/page/{}", BASE_URL, year, page);

            let task = async move {
                let page = browser.new_page(&url).await?;
                let a_tags = page.find_elements("a").await?;
                let mut urls = vec![];

                for a_tag in a_tags.into_iter() {
                    if let Some(class_name) = a_tag.attribute("class").await? {
                        if class_name != "more-link" {
                            continue;
                        }

                        if let Some(post_url) = a_tag.attribute("href").await? {
                            urls.push(post_url);
                        }
                    }
                }

                page.close().await?;

                Ok::<Vec<String>, anyhow::Error>(urls)
            };

            tasks.push(task);
        }
    }

    let post_urls: Vec<String> = try_join_all(tasks).await?.into_iter().flatten().collect();

    Ok(post_urls)
}

// mod tests {
//     use super::*;
//     use chromiumoxide::{Browser, BrowserConfig};
//     use futures_util::StreamExt;
//
//     #[tokio::test]
//     async fn should_get_post_urls() {
//         let (mut browser, mut handler) = Browser::launch(
//             BrowserConfig::builder()
//                 .headless_mode(chromiumoxide::browser::HeadlessMode::New)
//                 .build()
//                 .unwrap(),
//         )
//         .await
//         .unwrap();
//
//         let handle = tokio::task::spawn(async move {
//             loop {
//                 if let Some(_) = handler.next().await {
//                     continue;
//                 }
//
//                 break;
//             }
//         });
//
//         let urls = get_post_urls(&browser).await.unwrap();
//
//         browser.close().await.unwrap();
//         handle.await.unwrap();
//
//         assert!(!urls.is_empty());
//     }
// }
