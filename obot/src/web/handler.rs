use std::{
    env,
    error::Error,
    time,
    collections::HashMap,
};
use serenity::{
    model::{prelude::*},
    prelude::*,
};
use itertools::Itertools;

use crate::cache::Database;
use crate::web::api;
use api::Api;

use super::api::Beatmap;

// Beatmap構造体からいい感じにEmbed Messageを送る
pub async fn send_beatmap(ctx: &Context, beatmapset: &Beatmap, channel_id: &ChannelId) -> Result<(), Box<dyn Error>> {
    let (color, title_str) = match beatmapset.statu.as_str() {
        "ranked" => (0x00ff00, "Ranked"),
        "loved" => (0xff00ff, "Loved"),
        "qualified" => (0xffff00, "Qualified"),
        _ => (0xeeeeee, "Graveyard"),
    };

    let star_str = star_string(&beatmapset.stars, &beatmapset.keys);
    let url = api::get_url(&beatmapset);

    match channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(&format!("[{}] {} ({})", beatmapset.id, beatmapset.title, title_str))
                .color(color)
                .image(&beatmapset.card_url)
                .url(&url)
                .field("Artist", &beatmapset.artist, true)
                .field("Creator", &beatmapset.creator, true)
                .field("Star :star:", &star_str, false)
        });
        m
    }).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
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
            star_str.push_str(&format!("{}{}[0m", mxsc, max_star));
        } else {
            star_str.push_str(&format!("{}{}[0m ~ {}{}[0m\n", mnsc, min_star, mxsc, max_star));
        }
    }


    star_str.push_str(&format!("```"));
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
        star_vec.push(s.parse::<f32>().unwrap());
    }
    star_vec
}

// check ranked, loved, qualified beatmaps (50maps)
// if there is a new map, post it to discord (using sqlx)
// TODO: データベースに存在してるかどうかは上50件のみで判定するようにする
pub async fn check_maps(ctx: &Context) -> Result<(), Box<dyn Error + Send + Sync>> {

    let now = time::SystemTime::now();
    info!("{}", format!("check_maps started at {:?}", now));

    let api = Api::new();
    let token = match api.update_token().await {
        Ok(t) => t,
        Err(e) => {
            error!("{}", format!("Failed to update token: {}", e));
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let db = data.get::<Database>().unwrap();

    let statuses = ["ranked", "loved", "qualified"];
    let mode = "3"; // mania only
    let mut download_maps = Vec::new();
    for status in statuses.iter() {
        let maps = match api.get_beatmaps(&token, mode, status, "4", false).await {
            Ok(m) => m,
            Err(_e) => return Ok(()),
        };
        let mut map_ids = Vec::new();
        let db = db.lock().await;
        let rows = sqlx::query!(
            "SELECT id FROM beatmapsets WHERE stat = ?",
            status)
            .fetch_all(&*db)
            .await?;
        for row in rows {
            map_ids.push(row.id);
        }

        let mut new_maps = Vec::new();
        for map in maps {
            if map_ids.contains(&(map.id as i64)) == false {
                sqlx::query!("INSERT INTO beatmapsets (id, title, artist, stat) VALUES (?, ?, ?, ?)",
                 map.id, map.title, map.artist, status)
                    .execute(&*db)
                    .await?;
                new_maps.push(map.clone());
                // download only ranked and loved maps
                if status == &"ranked" || status == &"loved" {
                    download_maps.push(map.clone());
                }
            }
        }

        if new_maps.len() > 0 {
            let mut map_list = String::new();
            map_list.push_str(&format!("```ansi\n"));
            for map in new_maps {

                map_list.push_str(&format!("[{}] [1m{}[0m (by {}) ",map.id, map.title, map.artist));
                map_list.push_str(&format!("{}\n", star_string(&map.stars, &map.keys)));
            }
            map_list.push_str(&format!("```"));
            let channel_id: ChannelId = env::var("DISCORD_MAP_CHANNEL_ID").unwrap().parse().unwrap();
            let color = match status {
                &"ranked" => 0x00ff00,
                &"loved" => 0xff00ff,
                &"qualified" => 0xffff00,
                _ => 0x000000,
            };
            /*
            channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(format!("New {} maps", status))
                        .description(map_list)
                        .color(color)
                });
                m
            }).await?;
            */
        } else {
            info!("{}", format!("No new {} maps", status));
        }
    }

    // download maps
    let path = env::var("MAP_PATH").unwrap();
    match api.download_beatmaps(download_maps, &path).await {
        Ok(_) => info!("{}", format!("Downloaded maps")),
        Err(e) => error!("{}", format!("Failed to download maps: {}", e)),
    }

    Ok(())
}
