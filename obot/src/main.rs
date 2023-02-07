mod cache;
mod owner;
mod eventhandler;
mod scheduler;
mod commands;
mod web;
mod db;
mod utility;

use std::{
    env,
    collections::{HashSet, HashMap},
    sync::{Arc},
    error::Error,
};

use serenity::{
    model::{prelude::*},
    framework::{
        StandardFramework,
        standard::{
            macros::{group},
        },
    },
    http::Http,
    prelude::*,
};

use cache::*;
use eventhandler::*;
use crate::commands::{
    dbg::*, help::*, game::*,
};
use crate::utility::*;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[group]
#[description("Owner commands")]
#[summary("サーバの管理者のみが実行できるコマンドです(ほぼデバッグ用)")]
#[commands(shutdown, delmsg, infoc)]
struct Owner;

#[group]
#[description("General commands")]
#[summary("一般ユーザーが実行できるコマンドです")]
#[commands(todo)]
struct General;

#[group]
#[description("Game commands")]
#[summary("ゲームに関するコマンドです(一部管理者権限が必要)")]
#[commands(newmaps, update_database, dlmaps, mapset_info, init_database)]
struct Game;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = dotenv::dotenv() {
        warn!("Failed to load .env file: {}", e);
    }

    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Failed to run migrations");

    let http = Http::new(&token);
    
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(_) => {
                    error!("Could not access the bot id");
                    return Ok(());
                }
            }
        },
        Err(why) => {
            warn!("Could not access application info: {:?}", why);
            (HashSet::new(), UserId(0))
        }
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .owners(owners.clone())
            .prefix("/")
            .on_mention(Some(bot_id))
        )
        .unrecognised_command(unknown_command)
        .help(&MY_HELP)
        .group(&OWNER_GROUP)
        .group(&GENERAL_GROUP)
        .group(&GAME_GROUP);

    // gatewayを通してどのデータにbotがアクセスできるようにするかを指定する
    // https://docs.rs/serenity/latest/serenity/model/gateway/struct.GatewayIntents.html
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_WEBHOOKS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Error creating client");


    // 複数回使う環境変数は最初にロードしてcacheに入れておく
    let mut env_hashmap = HashMap::new();
    env_hashmap.insert("log_channel".to_string(), env_helper("DISCORD_LOG_CHANNEL_ID"));
    env_hashmap.insert("4k_ranked".to_string(), env_helper("DISCORD_4KMAP_RANKED_CHANNEL_ID"));
    env_hashmap.insert("4k_loved".to_string(), env_helper("DISCORD_4KMAP_LOVED_CHANNEL_ID"));
    env_hashmap.insert("4k_qualified".to_string(), env_helper("DISCORD_4KMAP_QUALIFIED_CHANNEL_ID"));
    env_hashmap.insert("7k_ranked".to_string(), env_helper("DISCORD_7KMAP_RANKED_CHANNEL_ID"));
    env_hashmap.insert("7k_loved".to_string(), env_helper("DISCORD_7KMAP_LOVED_CHANNEL_ID"));
    env_hashmap.insert("7k_qualified".to_string(), env_helper("DISCORD_7KMAP_QUALIFIED_CHANNEL_ID"));
    env_hashmap.insert("api_base".to_string(), env_helper("API_BASE"));
    env_hashmap.insert("download_base".to_string(), env_helper("DOWNLOAD_BASE"));
    env_hashmap.insert("user_id".to_string(), env_helper("USER_ID"));
    env_hashmap.insert("api_secret".to_string(), env_helper("API_SECRET"));
    env_hashmap.insert("map_path".to_string(), env_helper("MAP_PATH"));

    {
        let mut data = client.data.write().await;
        data.insert::<SharedManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Owners>(Arc::new(Mutex::new(owners)));
        data.insert::<Database>(Arc::new(Mutex::new(database)));
        data.insert::<Env>(Arc::new(Mutex::new(env_hashmap)));
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}