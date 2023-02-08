use serenity::{
    framework::standard::{
        macros::{command},
        CommandResult, Args,
    },
    model::{
        prelude::*,
    },
    builder::CreateEmbed,
    prelude::*,
};

use crate::owner;
use crate::utility;
use crate::web::{
    api::{Api, Beatmap}, handler as web_handler,
};
use crate::db::handler::DBHandler;

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
                _ => ["ranked", "loved", "qualified"].to_vec(),
            }
        }
        Err(_e) => {
            msg.channel_id.say(&ctx.http, "[ERROR] Failed to parse status! Please inform the owner...").await?;
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

    let db = DBHandler::new(ctx).await;

    let mut cursor = String::new();
    let mut beatmapsets: Vec<Beatmap> = Vec::new();
    let mut over_loop_checker = 0;
    for k in keys {
        for s in &status {
            loop {
                let res = match api.get_beatmapsets_with_cursor("3", s, k, &cursor).await {
                    Ok(r) => r,
                    Err(_e) => {
                        msg.channel_id.say(&ctx.http, &format!("[ERROR] Failed to get beatmapsets! (status: {}, key: {})", s, k)).await?;
                        break;
                    }
                };
                cursor = res.1;
                beatmapsets.extend(res.0);
                if cursor.is_empty() {
                    break;
                }    
                over_loop_checker += 1;
                if over_loop_checker > 1000 {
                    msg.channel_id.say(&ctx.http, &format!("[ERROR] Too many loops! (status: {}, key: {})", s, k)).await?;
                    break;
                }
            }
            // get beatmapsets from db
            for map in &beatmapsets {
                let res = match db.check_existence(&map.id, &map.statu).await {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to check existence: {}", e);
                        continue;
                    }
                };
                if !res {
                    match db.insert(&map).await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to insert: {}", e);
                            continue;
                        }
                    }
                }
            }
            // send msg to channel
            msg.channel_id.say(&ctx.http, &format!("Finished to initialize database (status: {}, key: {}) => {} beatmapsets", s, k, beatmapsets.len())).await?;
            beatmapsets.clear();
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
#[usage("[status] (default: ranked) [key] (default: 4)")]
#[max_args(2)]
#[min_args(0)]
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

    let mut marg = arg.clone();
    let mode = "3"; // mania
    // default status is ranked
    let status = match marg.single::<String>() {
        Ok(s) => {
            match s.as_str() {
                "ranked" => "ranked",
                "loved" => "loved",
                "qualified" => "qualified",
                "all" => "all",
                _ => "ranked",
            }
        }
        Err(_e) => "ranked",
    };
    let key = match marg.single::<String>() {
        Ok(k) => k,
        Err(_e) => "4".to_string(),
    };
    let cursor = String::new();

    let beatmapsets = match api.get_beatmapsets_with_cursor(mode, status, &key, &cursor).await {
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

    // update database
    let db = DBHandler::new(ctx).await;
    for map in for_db {
        match db.check_existence(&map.id, &map.statu).await {
            Ok(b) => {
                if b == false {
                    match db.insert(&map).await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to insert beatmap: {}", e);
                            continue;
                        }
                    }
                }
            },
            Err(e) => {
                error!("Failed to check existence: {}", e);
                continue;
            }
        }
    }

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

#[command]
#[description("譜面情報格納DBのサイズを表示します")]
#[usage("[status] (default: ranked), [key] (default: 4)")]
#[max_args(2)]
#[min_args(0)]
async fn dbsize(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    let mut marg = arg.clone();
    let status: Vec<&str> = match marg.single::<String>() {
        Ok(s) => {
            match s.as_str() {
                "ranked" => vec!["ranked"],
                "loved" => vec!["loved"],
                "qualified" => vec!["qualified"],
                "all" => vec!["ranked", "loved", "qualified"],
                _ => vec!["ranked"]
            }
        },
        Err(_) => vec!["ranked"]
    };
    let key: Vec<&str> = match marg.single::<String>() {
        Ok(s) => {
            match s.as_str() {
                "4" => vec!["4"],
                "7" => vec!["7"],
                "all" => vec!["4", "7"],
                _ => vec!["4"]
            }
        },
        Err(_) => vec!["4"]
    };


    let db = DBHandler::new(ctx).await;
    let mut embed = CreateEmbed::default();
    embed.title("DB size");
    embed.color(0x00ffff);
    for k in &key {
        for s in &status {
            let size = match db.get_db_size(s, k).await {
                Ok(s) => s,
                Err(e) => {
                    msg.channel_id.say(&ctx.http, format!("Failed to get {}({}k) db size...", s, k)).await?;
                    error!("Failed to get {}({}k) db size: {}", s, k, e);
                    return Ok(());
                }
            };
            embed.field(format!("{}({}k)", s, k), format!("{} mapsets", size), true);
        }
    }

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.0 = embed.0;
            e
        });
        m
    }).await?;

    Ok(())
}

#[command]
#[description("譜面情報格納DBの先頭の譜面情報を表示します(最大10件)")]
#[usage("[status] (default: ranked), [key] (default: 4) [num] (default: 1)")]
#[max_args(4)]
#[min_args(0)]
async fn dbtop(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    
    let mut marg = arg.clone();
    let status: Vec<&str> = match marg.single::<String>() {
        Ok(s) => {
            match s.as_str() {
                "ranked" => vec!["ranked"],
                "loved" => vec!["loved"],
                "qualified" => vec!["qualified"],
                "all" => vec!["ranked", "loved", "qualified"],
                _ => vec!["ranked"]
            }
        },
        Err(_) => vec!["ranked"]
    };
    let key: Vec<&str> = match marg.single::<String>() {
        Ok(s) => {
            match s.as_str() {
                "4" => vec!["4"],
                "7" => vec!["7"],
                "all" => vec!["4", "7"],
                _ => vec!["4"]
            }
        },
        Err(_) => vec!["4"]
    };
    let num: usize = match marg.single::<String>() {
        Ok(s) => {
            match s.parse::<usize>() {
                Ok(n) => n,
                Err(_) => 1
            }
        },
        Err(_) => 1
    };
    if num > 10 {
        msg.channel_id.say(&ctx.http, "num must be less than 10").await?;
        return Ok(());
    }

    let db = DBHandler::new(ctx).await;
    for k in &key {
        for s in &status {
            let db_size = match db.get_db_size(s, k).await {
                Ok(s) => s,
                Err(e) => {
                    msg.channel_id.say(&ctx.http, format!("Failed to get {}({}k) db size...", s, k)).await?;
                    error!("Failed to get {}({}k) db size: {}", s, k, e);
                    return Ok(());
                }
            };
            let offset = if db_size > num as i32 { db_size - num as i32 } else { 0 };
            let topmapsets = match db.select_with_limit("keys", s, k, num as i64, offset as i64).await {
                Ok(t) => t,
                Err(e) => {
                    msg.channel_id.say(&ctx.http, format!("Failed to get {}({}k) db top...", s, k)).await?;
                    error!("Failed to get {}({}k) db top: {}", s, k, e);
                    return Ok(());
                }
            };
            match web_handler::simple_beatmap_send(ctx, &topmapsets, &msg.channel_id).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to send beatmap: {}", e);
                    continue;
                }
            }
        }
    }

    Ok(())
}