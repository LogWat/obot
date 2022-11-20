use clokwerk::{TimeUnits, AsyncScheduler};
use serenity::prelude::*;
use std::{
    time::Duration,
    sync::Arc,
    error::Error,
};

use crate::web::{
    handler,
};

pub async fn rss_scheduler(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    let mut scheduler = AsyncScheduler::new();
    let ctx_clone = ctx.clone();
    scheduler.every(30.minutes()).run(move || {
        let ctx = ctx_clone.clone();
        async move {
            let _ = handler::check_maps(&ctx).await;
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