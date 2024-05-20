mod dangerous_business_blog;
mod neverending_footsteps_blog;

use dangerous_business_blog::*;
use neverending_footsteps_blog::*;

use crate::DataMiner;

pub fn miners() -> Vec<Box<dyn DataMiner>> {
    let ms: Vec<Box<dyn DataMiner>> = vec![
        Box::new(DangerousBusinessBlog::new()),
        Box::new(NeverendingFootstepsBlog::new()),
    ];

    ms
}
