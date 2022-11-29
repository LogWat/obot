use std::{
    env,
    sync::Arc,
};

use serenity::{
    async_trait,
    model::{gateway::Ready, prelude::*},
    framework::{
        standard::{
            macros::{hook},
        },
    },
    prelude::*,
};

use crate::scheduler;
use crate::cache::{CommandCounter};

pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    // This is called when the bot starts up.
    async fn ready(&self, ctx: Context, ready: Ready) {
        let log_channel_id: ChannelId = env::var("DISCORD_LOG_CHANNEL_ID")
            .expect("DISCORD_LOG_CHANNEL_ID must be set")
            .parse()
            .expect("DISCORD_LOG_CHANNEL_ID must be a valid channel ID");
        match log_channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Bot started")
                    .description(format!("{} is now online!", ready.user.name))
                    .color(0x00ffff)
            })
        }).await {
            Ok(_) => info!("Logged bot start"),
            Err(e) => error!("Failed to log bot start: {}", e),
        }

        // Start the scheduler
        let ctx_clone = Arc::new(ctx);
        let _ = scheduler::ascheduler(ctx_clone).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let self_id = ctx.http.get_current_user().await.unwrap().id;
        for mention in msg.mentions.iter() {
            if mention.id == self_id {
                match msg.channel_id.say(&ctx.http, format!("Hello, {}!", msg.author.name)).await {
                    Ok(_) => info!("Sent hello message to {}", msg.author.name),
                    Err(e) => error!("Failed to send hello message to {}: {}", msg.author.name, e),
                }
            }
        }
    }
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    match msg.channel_id.say(&ctx.http, format!("Unknown command: '{}'. Try `/help`", unknown_command_name))
        .await {
        Ok(_) => info!("Sent unknown command message to {}", msg.author.name),
        Err(e) => error!("Failed to send unknown command message to {}: {}", msg.author.name, e),
    }
}

// Command Counter
#[hook]
async fn before(ctx: &Context, _msg: &Message, command_name: &str) -> bool {
    let mut data = ctx.data.write().await;
    let counter = match data.get_mut::<CommandCounter>() {
        Some(c) => c,
        None => {
            error!("Expected CommandCounter in TypeMap.");
            return false;
        },
    };
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true
}