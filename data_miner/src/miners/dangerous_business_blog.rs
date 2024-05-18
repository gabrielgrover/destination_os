use std::{collections::HashSet, error::Error};

use futures_util::{future::try_join_all, FutureExt};
use itertools::*;
use scraper::selectable::Selectable;

use crate::DataMiner;

pub struct DangerousBusinessBlog {
    name: String,
}

impl DangerousBusinessBlog {
    pub fn new() -> Self {
        DangerousBusinessBlog {
            name: "dangerous_business_blog".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DataMiner for DangerousBusinessBlog {
    async fn mine(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        dangerous_business_scrape().await
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

async fn dangerous_business_scrape() -> Result<Vec<u8>, Box<dyn Error>> {
    let urls = get_post_urls().await?;
    let all_blogs = get_all_blog_content(urls).await?;
    println!("all blogs: {}", all_blogs);
    let bytes: Vec<u8> = all_blogs.into();

    println!("content size {} bytes", bytes.len());

    Ok(bytes)
}

async fn get_all_blog_content(chunked_urls: Vec<Vec<String>>) -> Result<String, Box<dyn Error>> {
    let mut chunked_content = vec![];

    for urls in chunked_urls.into_iter() {
        let tasks: Vec<_> = urls
            .iter()
            .map(|url| {
                println!("Fetching url: {}", url);
                reqwest::get(url).then(|resp_result| async move {
                    match resp_result {
                        Ok(resp) => resp.text().await,
                        Err(e) => Err(e),
                    }
                })
            })
            .collect();
        let content = try_join_all(tasks).await?.iter().fold(
            "".to_string(),
            |mut joined_content, content| {
                joined_content.push_str(content);

                joined_content
            },
        );

        // Get all p tag children of the post-content div and collate their inner html
        let f = scraper::Html::parse_fragment(&content);
        let post_content_container_div_selector =
            scraper::Selector::parse(r#"div[class="post-content"]"#)?;
        let post_content_p_tag_selector = scraper::Selector::parse("p")?;

        if let Some(post_content_container_div) =
            f.select(&post_content_container_div_selector).next()
        {
            let collated_blog_content = post_content_container_div
                .select(&post_content_p_tag_selector)
                .into_iter()
                .fold("".to_string(), |mut collated, p_tag| {
                    collated.push_str(&p_tag.inner_html());

                    collated
                });

            chunked_content.push(collated_blog_content);
        }

        // let selector = scraper::Selector::parse("p")?;
        // let collated_blog_content =
        //     f.select(&selector)
        //         .into_iter()
        //         .fold("".to_string(), |mut collated, p_tag| {
        //             collated.push_str(&p_tag.inner_html());

        //             collated
        //         });

        // chunked_content.push(collated_blog_content);
    }

    Ok(chunked_content.join(" "))
}

async fn get_post_urls() -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let mut urls_set = HashSet::<String>::new();
    let mut page_count = 48;
    let mut tasks = vec![];

    loop {
        if page_count <= 0 {
            break;
        }

        let url = next_url(page_count);
        page_count -= 1;

        tasks.push(reqwest::get(url).then(|resp_result| async move {
            match resp_result {
                Ok(resp) => resp.text().await,
                Err(err) => Err(err),
            }
        }));
    }
    let content =
        try_join_all(tasks)
            .await?
            .iter()
            .fold("".to_string(), |mut joined_content, content| {
                joined_content.push_str(content);

                joined_content
            });
    let f = scraper::Html::parse_fragment(&content);
    let selector = scraper::Selector::parse("article").unwrap();

    for element in f.select(&selector) {
        let a_selector = scraper::Selector::parse("a").unwrap();

        for a_tag in element.select(&a_selector) {
            if let Some(url) = a_tag.attr("href") {
                let clean_url = url.replace("\\", "").trim_matches('"').to_string();
                urls_set.insert(clean_url);
            }
        }
    }

    let urls: Vec<_> = urls_set.into_iter().collect();
    let chunked_urls: Vec<_> = urls
        .into_iter()
        .chunks(100)
        .into_iter()
        .map(|c| c.collect())
        .collect();

    Ok(chunked_urls)
}

fn next_url(count: u8) -> String {
    let url = format!(
        r#"https://www.dangerous-business.com/wp-json/foundry-ra/v1/page/{}?page=0&pagename=blog&error=&m=&p=0&post_parent=&subpost=&subpost_id=&attachment=&attachment_id=0&name=&page_id=0&second=&minute=&hour=&day=0&monthnum=0&year=0&w=0&category_name=&tag=&cat=&tag_id=&author=&author_name=&feed=&tb=&paged=0&meta_key=&meta_value=&preview=&s=&sentence=&title=&fields=&menu_order=&embed=&ignore_sticky_posts=false&suppress_filters=false&cache_results=true&update_post_term_cache=true&update_menu_item_cache=false&lazy_load_term_meta=true&update_post_meta_cache=true&post_type=&posts_per_page=17&nopaging=false&comments_per_page=100&no_found_rows=false&order=DESC"#,
        count
    );

    url
}
