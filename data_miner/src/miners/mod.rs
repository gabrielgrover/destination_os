mod dangerous_business_blog;

use dangerous_business_blog::*;

use crate::DataMiner;

pub fn miners() -> Vec<Box<dyn DataMiner>> {
    let ms: Vec<Box<dyn DataMiner>> = vec![Box::new(DangerousBusinessBlog::new())];

    ms
}
