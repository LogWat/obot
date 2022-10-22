use std::{
    env,
    sync::{Arc, Mutex},
    collections::{HashMap, HashSet},
}

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    framework::{
        StandardFramework,
    }
    http::Http,
    prelude::*,
};

struct Handler;
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
                e.title("Bot started");
                    .description(format!("{} is now online!", ready.user.name));
                    .color(0x00ff00)
            })
        }).await.expect("Failed to send message");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = dotenv::dotenv() {
        println!("Failed to load .env file: {}", e);
    }

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let http = Http::new(&token);
    
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix(";")
            .owners(owners)
            .on_mention(Some(bot_id))
            .group(&GENERAL_GROUP)
        );

    // gatewayを通してどのデータにbotがアクセスできるようにするかを指定する
    // https://docs.rs/serenity/latest/serenity/model/gateway/struct.GatewayIntents.html
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_WEBHOOKS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MEMBERS
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}