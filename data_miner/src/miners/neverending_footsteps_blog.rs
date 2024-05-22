use itertools::*;
use std::error::Error;

use crate::DataMiner;

use anyhow::anyhow;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::{future::try_join_all, FutureExt, StreamExt};

pub struct NeverendingFootstepsBlog {
    name: String,
}

impl NeverendingFootstepsBlog {
    pub fn new() -> Self {
        NeverendingFootstepsBlog {
            name: "neverending_footsteps_blog".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DataMiner for NeverendingFootstepsBlog {
    async fn mine(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let content = neverending_footsteps_scrape().await?;
        println!("{} content size {} bytes", self.name, content.len());

        Ok(content)
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

async fn neverending_footsteps_scrape() -> Result<Vec<u8>, Box<dyn Error>> {
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
    let chunked_urls: Vec<Vec<String>> = urls
        .into_iter()
        .chunks(100)
        .into_iter()
        .map(|c| c.collect())
        .collect();

    let mut all_blog_content = vec![];

    for chunk in chunked_urls.into_iter() {
        println!("Scraping chunk...");
        let content_collection = get_all_blog_content(chunk, &browser).await?;
        println!("Chunk scrape succesful");

        for content in content_collection.into_iter() {
            all_blog_content.push(content);
        }
    }

    browser.close().await?;
    handle.await?;

    Ok(all_blog_content)
}

async fn get_post_urls(browser: &Browser) -> Result<Vec<String>, Box<dyn Error>> {
    let url = "https://www.neverendingfootsteps.com/archive/".to_string();
    let page = browser.new_page(url).await?;
    let a_tags = page.find_elements("a").await?;
    let mut urls = vec![];

    for a_tag in a_tags.into_iter() {
        if let Some(class_name) = a_tag.attribute("class").await? {
            if !class_name.contains("sya_postlink") {
                continue;
            }

            if let Some(url) = a_tag.attribute("href").await? {
                urls.push(url);
            }
        }
    }

    page.close().await?;

    Ok(urls)
}

async fn get_all_blog_content(
    urls: Vec<String>,
    browser: &Browser,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let pages = try_join_all(urls.into_iter().map(|url| browser.new_page(url))).await?;
    let mut tasks = vec![];

    for page in pages.iter() {
        let page_url = page.url().await?.unwrap();

        let task = page
            .find_element("div.vw-post-content")
            .then(|maybe_div| async move {
                let maybe_p_tags = match maybe_div {
                    Ok(div) => div.find_elements("p").await,
                    Err(e) => Err(e),
                };

                println!("Scraping: {}", page_url);

                match maybe_p_tags {
                    Ok(p_tags) => try_join_all(p_tags.into_iter().map(|p_tag| async move {
                        if let Some(text) = p_tag.inner_text().await? {
                            Ok(text)
                        } else {
                            Ok(String::new())
                        }
                    }))
                    .await
                    .map(|p_tag_texts| {
                        if p_tag_texts.is_empty() {
                            println!("No content found for url: {}", page_url)
                        }
                        p_tag_texts.join(" ")
                    }),

                    Err(e) => Err(anyhow!(e.to_string())),
                }
            });

        tasks.push(task);
    }

    let scrape_results = try_join_all(tasks).await?.join(" ");

    Ok(scrape_results.into())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn should_get_post_urls() {
//         let urls = get_post_urls().await.unwrap();
//
//         println!("{:?}", urls);
//
//         assert!(false);
//
//         assert!(!urls.is_empty());
//     }
// }
