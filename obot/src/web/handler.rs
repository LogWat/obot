use std::{
    error::Error,
    time,
    collections::HashMap,
};
use serenity::{
    model::{prelude::*},
    prelude::*,
};
use itertools::Itertools;

use crate::utility;
use crate::db::handler::DBHandler;
use crate::web::api;
use api::Api;

use super::api::Beatmap;

// Beatmap構造体からいい感じにEmbed Messageを送る
pub async fn send_beatmap(ctx: &Context, beatmapset: &Beatmap, channel_id: &ChannelId) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (color, title_str) = match beatmapset.statu.as_str() {
        "ranked" => (0x00ff00, "Ranked"),
        "loved" => (0xff00ff, "Loved"),
        "qualified" => (0xffff00, "Qualified"),
        _ => (0xeeeeee, "Graveyard"),
    };

    let star_str = star_string(&beatmapset.stars, &beatmapset.keys);
    let url = api::get_url(ctx, &beatmapset).await;

    match channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(&format!("[{}] {} ({})", beatmapset.id, beatmapset.title, title_str))
                .color(color)
                .image(&beatmapset.card_url)
                .url(&url)
                .field("Artist", &beatmapset.artist, true)
                .field("Creator", &beatmapset.creator, true)
                .field("Star ", &star_str, false)
        });
        m
    }).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}


// 複数件のBeatmapset情報を送りたい場合
pub async fn simple_beatmap_send(ctx: &Context, beatmapsets: &Vec<Beatmap>, channel_id: &ChannelId) -> Result<(), Box<dyn Error + Send + Sync>> {
    let status = beatmapsets[0].statu.as_str();
    let (color, title_str) = match status {
        "ranked" => (0x00ff00, "Ranked"),
        "loved" => (0xff00ff, "Loved"),
        "qualified" => (0xffff00, "Qualified"),
        _ => (0xeeeeee, "Graveyard"),
    };

    let mut msg = String::new();
    for beatmapset in beatmapsets {
        let star_str = simple_starstr(&beatmapset.stars, &beatmapset.keys);
        let url = api::get_url(ctx, &beatmapset).await;
        msg.push_str(&format!("[({}) {}]({}) {}\n", beatmapset.id, beatmapset.title, url, star_str));
    }

    match channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(&format!("{} Beatmapsets ({})", title_str, beatmapsets.len()))
                .color(color)
                .description(&msg)
        });
        m
    }).await {
        Ok(_) => {},
        Err(e) => return Err(Box::new(e)),
    };

    Ok(())
}


// starから色付き文字列を返す
// キー数ごとに難易度を表示
pub fn star_string(star: &str, keys: &str) -> String {
    let star = star_to_vec(&star);
    let keys = keys_to_vec(&keys);

    let mut key: HashMap<&String, Vec<f32>> = HashMap::new();
    for (k, s) in keys.iter().zip(star.iter()) {
        key.entry(k).or_insert(Vec::new()).push(*s);
    }

    let mut star_str = String::new();
    star_str.push_str("```ansi\n");

    for k in key.keys().sorted() {
        let stars = key.get(k).unwrap();
        let max_star = stars.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_star = stars.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let mxsc = if *max_star >= 0.0 && *max_star <= 1.75 {
            String::from("[0;36m")
        } else if *max_star >= 1.76 && *max_star <= 3.5 {
            String::from("[0;32m")
        } else if *max_star >= 2.51 && *max_star <= 4.5 {
            String::from("[0;33m")
        } else if *max_star >= 4.51 && *max_star <= 5.5 {
            String::from("[0;31m")
        } else if *max_star >= 5.51 && *max_star <= 6.0 {
            String::from("[0;35m")
        } else {
            String::from("[0;30;45m")
        };
    
        let mnsc = if *min_star >= 0.0 && *min_star <= 1.75 {
            String::from("[0;36m")
        } else if *min_star >= 1.76 && *min_star <= 3.5 {
            String::from("[0;32m")
        } else if *min_star >= 2.51 && *min_star <= 4.5 {
            String::from("[0;33m")
        } else if *min_star >= 4.51 && *min_star <= 5.5 {
            String::from("[0;31m")
        } else if *min_star >= 5.51 && *min_star <= 6.0 {
            String::from("[0;35m")
        } else {
            String::from("[0;30;45m")
        };

        star_str.push_str(&format!("{}k: ", k));

        if max_star == min_star {
            star_str.push_str(&format!("{}{}[0m\n", mxsc, max_star));
        } else {
            star_str.push_str(&format!("{}{}[0m ~ {}{}[0m\n", mnsc, min_star, mxsc, max_star));
        }
    }


    star_str.push_str(&format!("```"));
    star_str
}


// gen string like "(key): (star) ~ (star)"
pub fn simple_starstr(star: &str, keys: &str) -> String {
    let star = star_to_vec(&star);
    let keys = keys_to_vec(&keys);

    let mut key: HashMap<&String, Vec<f32>> = HashMap::new();
    for (k, s) in keys.iter().zip(star.iter()) {
        key.entry(k).or_insert(Vec::new()).push(*s);
    }

    let mut star_str = String::new();
    for k in key.keys().sorted() {
        let stars = key.get(k).unwrap();
        let max_star = stars.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_star = stars.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        if max_star == min_star {
            star_str.push_str(&format!("{}k: {} ", k, max_star));
        } else {
            star_str.push_str(&format!("{}k: {} ~ {} ", k, min_star, max_star));
        }
    }

    star_str
}

pub fn keys_to_vec(keys: &str) -> Vec<String> {
    let mut key_vec = Vec::new();
    for k in keys.split(',') {
        key_vec.push(k.to_string());
    }
    key_vec
}

pub fn star_to_vec(star: &str) -> Vec<f32> {
    let mut star_vec = Vec::new();
    for s in star.split(',') {
        if s.is_empty() {
            continue;
        }
        star_vec.push(s.parse::<f32>().unwrap());
    }
    star_vec
}

// スケジューラから呼び出される関数
// 各status, 4k, 7kの最新譜面50件を取得し，DBと比較して新規譜面があればDBに追加して特定のチャンネルに通知
pub async fn check_maps(ctx: &Context) -> Result<(), Box<dyn Error + Send + Sync>> {

    let now = time::SystemTime::now();
    info!("{}", format!("check_maps started at {:?}", now));

    let api = match Api::new(ctx).await {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    let db = DBHandler::new(ctx).await;

    let statuses = ["ranked", "loved", "qualified"];
    let keys = ["4", "7"];
    let mode = "3"; // mania only
    let mut download_maps = Vec::new();
    for status in statuses.iter() {
        for key in keys.iter() {
            let maps = match api.get_beatmapsets_with_cursor(mode, status, key, "").await {
                Ok(m) => m,
                Err(_e) => {
                    warn!("get_beatmapsets_with_cursor [{}] failed", status);
                    continue;
                }
            };

            let mut new_maps = Vec::new();
            for map in maps.0.iter() {
                if map.statu != status.to_string() {
                    continue;
                }
                let res = match db.check_existence(&map.id, status).await {
                    Ok(b) => b,
                    Err(e) => {
                        error!("{}", format!("Failed to check existence: {}", e));
                        continue;
                    }
                };
                if !res {
                    new_maps.push(map.clone());
                    download_maps.push(map.clone());
                }
            }

            // 送信だけ
            let env_name = format!("{}k_{}", key, status);
            let channel_id = match utility::get_env_from_context(ctx, &env_name).await.parse::<ChannelId>() {
                Ok(c) => c,
                Err(_e) => {
                    warn!("get_env_from_context [{}] failed", env_name);
                    continue;
                }
            };
            if new_maps.len() > 0 {
                info!("{}", format!("{} new {} maps ({}k)", new_maps.len(), status, key));
                for map in new_maps {
                    match send_beatmap(ctx, &map, &channel_id).await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{}", format!("Failed to send beatmap: {}", e));
                            continue;
                        }
                    }
                }
            } else {
                info!("{}", format!("No new {} maps", status));
            }
        }
    }

    // DBに追加
    for map in download_maps.iter() {
        match db.insert(map).await {
            Ok(_) => {},
            Err(e) => error!("{}", format!("Failed to insert map: {}", e)),
        }
    }

    // download maps
    let path = utility::get_env_from_context(ctx, "map_path").await;
    match api.download_beatmaps(download_maps, &path).await {
        Ok(_) => info!("{}", format!("Downloaded maps")),
        Err(e) => error!("{}", format!("Failed to download maps: {}", e)),
    }

    Ok(())
}
