use serenity::{
    framework::standard::{
        macros::{command},
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
};

use crate::cache::SharedManagerContainer;
use crate::owner;

#[command]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Shutdown command received");
    let data = ctx.data.read().await;
    let manager = data.get::<SharedManagerContainer>().cloned().unwrap();
    let mut manager = manager.lock().await;

    if owner::is_owner(&ctx, msg.author.id).await {
        msg.channel_id.say(&ctx.http, "Shutting down...").await?;
        manager.shutdown_all().await;
    } else {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
    }

    Ok(())
}