use serenity::{
    framework::standard::{
        macros::{command},
        CommandResult, Args,
    },
    model::{
        prelude::*,
    },
    prelude::*,
};

use crate::owner;
use crate::web::{
    api::{Api}, handler,
};

#[command]
#[description("最新50件の譜面情報により譜面データベースを強制的に更新します")]
async fn update_database(ctx: &Context, msg: &Message) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    match handler::check_maps(ctx).await {
        Ok(_) => {},
        Err(_e) => {},
    }

    Ok(())
}

// test command: get_maps
// fetch api and print maps
#[command]
#[description("最新のbeatmapsets 50件を表示します(ranked, loved, qualified)")]
#[max_args(1)]
#[min_args(0)]
#[usage("get_maps [mode] (default: ranked)")]
async fn get_maps(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    let api = Api::new();
    let token = match api.update_token().await {
        Ok(t) => t,
        Err(e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to update token: {}", e)).await?;
            return Ok(());
        }
    };
    let mode = "3"; // mania

    let status = match arg.current() {
        Some("ranked") => "ranked",
        Some("loved") => "loved",
        Some("qualified") => "qualified",
        _ => "ranked",
    };

    let maps = match api.get_beatmaps(&token, mode, status, false).await {
        Ok(m) => m,
        Err(e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to get beatmaps: {}", e)).await?;
            return Ok(());
        }
    };
    let mut map_list = String::new();
    for map in maps {
        map_list.push_str(&format!("{} - {} [{}]\n", map.artist, map.title, map.star[0]));
    }
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Ranked mania maps");
            e.description(map_list);
            e
        });
        m
    }).await.expect("Failed to send message");
    Ok(())
}

// test command: download_map
// fetch api and download map
#[command]
#[description("指定されたidの譜面をダウンロードします(最大10件)")]
#[max_args(10)]
#[min_args(1)]
#[usage("dlmaps [id]")]
async fn dlmaps(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    let api = Api::new();
    let token = match api.update_token().await {
        Ok(t) => t,
        Err(e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to update token: {}", e)).await?;
            return Ok(());
        }
    };
    let mut map_ids = Vec::new();
    for id in arg.iter::<String>() {
        map_ids.push(match id {
            Ok(i) => i,
            Err(e) => {
                msg.channel_id.say(&ctx.http, format!("Invalid id: {}", e)).await?;
                continue;
            }
        });
    }
    let maps = match api.get_beatmaps_by_ids(&token, map_ids).await {
        Ok(m) => m,
        Err(e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to get beatmaps: {}", e)).await?;
            return Ok(());
        }
    };
    
    if let Err(e) = api.download_beatmaps(maps, "~/Downloads/").await {
        msg.channel_id.say(&ctx.http, format!("Failed to download beatmaps: {}", e)).await?;
        return Ok(());
    }

    msg.channel_id.say(&ctx.http, "Downloaded beatmaps").await?;
    Ok(())
}