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
use crate::utility;
use crate::web::{
    api::{Api, Beatmap}, handler as web_handler,
};
use crate::db::handler as db_handler;

// dbg command: init_database
// initialize database
#[command]
#[description("全ての譜面情報により譜面データベースを強制的に更新します")] 
#[max_args(1)]
#[min_args(0)]
#[usage("init_database [status] (default: all status)")]
async fn init_database(ctx: &Context, msg: &Message, mut arg: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }

    let keys = vec!["4", "7"];

    let status: Vec<&str> = match arg.single::<String>() {
        Ok(s) => {
            match s.as_str() {
                "ranked" => ["ranked"].to_vec(),
                "loved" => ["loved"].to_vec(),
                "qualified" => ["qualified"].to_vec(),
                "all" => ["ranked", "loved", "qualified"].to_vec(),
                _ => {
                    msg.channel_id.say(&ctx.http, "Invalid status: ranked, loved, qualified, all").await?;
                    return Ok(());
                }
            }
        }
        Err(_e) => {
            msg.channel_id.say(&ctx.http, "Failed to parse status").await?;
            return Ok(());
        }
    };

    let api = match Api::new(ctx).await {
        Ok(a) => a,
        Err(e) => {
            msg.channel_id.say(&ctx.http, "[ERROR] Failed to initialize api! Please inform the owner...").await?;
            error!("Failed to initialize api: {}", e);
            return Ok(());
        }
    };

    let mut cursor = String::new();
    let mut beatmapsets: Vec<Beatmap> = Vec::new();
    for k in keys {
        for s in &status {
            loop {
                let res = match api.get_beatmapsets_with_cursor("3", s, k, &cursor).await {
                    Ok(r) => r,
                    Err(_e) => {
                        msg.channel_id.say(&ctx.http, "[ERROR] Failed to fetch beatmapsets! Please inform the owner...").await?;
                        return Ok(());
                    }
                };
                cursor = res.1;
                beatmapsets.extend(res.0);
                if cursor.is_empty() {
                    break;
                }    
            }
            // get beatmapsets from db

        }
    }



    Ok(())
}

#[command]
#[description("最新50件の譜面情報により譜面データベースを強制的に更新します")]
async fn update_database(ctx: &Context, msg: &Message) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    match web_handler::check_maps(ctx).await {
        Ok(_) => {},
        Err(_e) => {},
    }

    Ok(())
}

// test command: newmaps
#[command]
#[description("最新のbeatmapsets情報10件を表示します(ranked, loved, qualified)")]
#[usage("newmaps [status] (default: ranked) [key] (default: 4)")]
#[max_args(2)]
#[min_args(0)]
#[usage("newmaps [mode] (default: ranked)")]
async fn newmaps(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }

    let api = match Api::new(ctx).await {
        Ok(a) => a,
        Err(e) => {
            msg.channel_id.say(&ctx.http, "[ERROR] Failed to initialize api! Please inform the owner...").await?;
            error!("Failed to initialize api: {}", e);
            return Ok(());
        }
    };

    let mode = "3"; // mania
    // default status is ranked
    let status = match arg.current() {
        Some("ranked") => "ranked",
        Some("loved") => "loved",
        Some("qualified") => "qualified",
        _ => "ranked",
    };
    let key = match arg.current() {
        Some("4") => "4",
        Some("7") => "7",
        _ => "4",
    };
    let cursor = String::new();

    let beatmapsets = match api.get_beatmapsets_with_cursor(mode, status, key, &cursor).await {
        Ok(b) => b,
        Err(e) => {
            msg.channel_id.say(&ctx.http, "[ERROR] Failed to fetch beatmapsets... Please inform the owner!").await?;
            error!("Failed to fetch beatmapsets: {}", e);
            return Ok(());
        }
    };

    // top 10 beatmapsets
    let mut beatmapsets = beatmapsets.0;
    let for_db = beatmapsets.clone();
    beatmapsets.truncate(10);

    match web_handler::simple_beatmap_send(ctx, &beatmapsets, &msg.channel_id).await {
        Ok(_) => {},
        Err(e) => {
            msg.channel_id.say(&ctx.http, "[ERROR] Failed to send beatmapsets... Please inform the owner!").await?;
            error!("Failed to send beatmapsets: {}", e);
            return Ok(());
        }
    };

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
    let api = match Api::new(ctx).await {
        Ok(a) => a,
        Err(e) => {
            msg.channel_id.say(&ctx.http, "[ERROR] Failed to initialize api! Please inform the owner...").await?;
            error!("Failed to initialize api: {}", e);
            return Ok(());
        }
    };
    let mut map_ids = Vec::new();
    let mut marg = arg.clone();
    while let Ok(id) = marg.single::<String>() {
        map_ids.push(id);
    }
    let maps = match api.get_beatmaps_by_ids(map_ids).await {
        Ok(m) => m,
        Err(e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to get beatmaps: {}", e)).await?;
            return Ok(());
        }
    };
    
    let dir = utility::get_env_from_context(ctx, "map_path").await;
    if let Err(e) = api.download_beatmaps(maps, &dir).await {
        msg.channel_id.say(&ctx.http, format!("Failed to download beatmaps: {}", e)).await?;
        return Ok(());
    }

    msg.channel_id.say(&ctx.http, "Downloaded beatmaps").await?;
    Ok(())
}

// test command: mapset_info
// fetch api and print mapset info
#[command]
#[description("指定されたidの譜面情報を表示します (最大10件)")]
#[max_args(10)]
#[min_args(1)]
#[usage("mapset_info [mapset id]")]
async fn mapset_info(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }

    let api = match Api::new(ctx).await {
        Ok(a) => a,
        Err(e) => {
            msg.channel_id.say(&ctx.http, "[ERROR] Failed to initialize api! Please inform the owner...").await?;
            error!("Failed to initialize api: {}", e);
            return Ok(());
        }
    };

    let mut mapset_ids = Vec::new();
    let mut marg = arg.clone();
    while let Ok(id) = marg.single::<String>() {
        mapset_ids.push(id);
    }

    let mapset = match api.get_beatmaps_by_ids(mapset_ids).await {
        Ok(m) => m,
        Err(_e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to get mapset!\n Please inform the owner")).await?;
            return Ok(());
        }
    };

    // 1件ずつembedで表示
    for map in mapset {
        match web_handler::send_beatmap(&ctx, &map, &msg.channel_id).await {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to send beatmap: {}", e);
                continue;
            }
        }
    }

    Ok(())
}