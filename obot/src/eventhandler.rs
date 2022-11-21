use std::{
    env,
    error::Error,
    sync::Arc,
};

use serenity::{
    async_trait,
    model::{gateway::Ready, prelude::*},
    prelude::*,
};

use crate::scheduler;

pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    // This is called when the bot starts up.
    async fn ready(&self, ctx: Context, ready: Ready) {
        let log_channel_id: ChannelId = env::var("DISCORD_LOG_CHANNEL_ID")
            .expect("DISCORD_LOG_CHANNEL_ID must be set")
            .parse()
            .expect("DISCORD_LOG_CHANNEL_ID must be a valid channel ID");
        log_channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Bot started")
                    .description(format!("{} is now online!", ready.user.name))
                    .color(0x00ffff)
            })
        }).await.expect("Failed to send message");

        // Start the scheduler
        let ctx_clone = Arc::new(ctx);
        let _ = scheduler::ascheduler(ctx_clone).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let self_id = ctx.http.get_current_user().await.unwrap().id;
        for mention in msg.mentions.iter() {
            if mention.id == self_id {
                msg.channel_id.say(&ctx.http, format!("Hello, {}!", msg.author.name))
                    .await
                    .expect("Failed to send message");
            }
        }
    }
}