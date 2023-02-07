use std::env;

use serenity::prelude::*;

use crate::cache::*;

pub fn env_helper(key: &str) -> String {
    match env::var(key) {
        Ok(v) => v,
        Err(e) => {
            panic!("Failed to load environment variable {}: {}", key, e);
        }
    }
}

pub async fn get_env_from_context(ctx: &Context, key: &str) -> String {
    match ctx.data.read().await.get::<Env>().unwrap().lock().await.get(key) {
        Some(v) => v.clone(),
        None => {
            panic!("Failed to load environment variable {}: {}", key, "Not found");
        }
    }
}