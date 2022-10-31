use clokwerk::{TimeUnits, AsyncScheduler, Job};
use serenity::prelude::*;
use std::{
    time::Duration,
    sync::Arc,
    error::Error,
};

use crate::web::api::*;

pub async fn rss_scheduler(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    let mut scheduler = AsyncScheduler::new();
    let ctx_clone = ctx.clone();
    scheduler.every(1.hour()).run(move || {
        let ctx = ctx_clone.clone();
        async move {
            let data = ctx.data.read().await;
            let rss = data.get::<super::rss::Rss>().unwrap().clone();
            let mut rss = rss.lock().await;
            rss.update().await;
        }
    });
    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    Ok(())
}