use std::{
    env,
    error::Error,
    time,
};
use serenity::{
    model::{prelude::*},
    prelude::*,
};

use crate::cache::Database;
use crate::web::api;
use api::Api;

// check ranked, loved, qualified beatmaps (50maps)
// if there is a new map, post it to discord (using sqlx)
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
    for status in statuses.iter() {
        let maps = match api.get_beatmaps(&token, mode, status, false).await {
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
        let mut download_maps = Vec::new();
        for map in maps {
            if map_ids.contains(&(map.id as i64)) == false {
                sqlx::query!("INSERT INTO beatmapsets (id, title, artist, stat) VALUES (?, ?, ?, ?)",
                 map.id, map.title, map.artist, status)
                    .execute(&*db)
                    .await?;
                new_maps.push(map);
                // download only ranked and loved maps
                if status == &"ranked" || status == &"loved" {
                    download_maps.push(map);
                }
            }
        }

        if new_maps.len() > 0 {
            let mut map_list = String::new();
            map_list.push_str(&format!("```ansi\n"));
            for map in new_maps {

                // def star string
                let mut max_star = 0.0;
                let mut min_star = 1000.0;
                for s in map.star {
                    if s > max_star {
                        max_star = s;
                    }
                    if s < min_star {
                        min_star = s;
                    }
                }
                let mut max_star_color = String::new();
                match max_star {
                    0.0..=1.75 => max_star_color = String::from("\u001b[0;36m"),
                    1.76..=2.75 => max_star_color = String::from("\u001b[0;32m"),
                    2.76..=3.75 => max_star_color = String::from("\u001b[0;33m"),
                    3.76..=4.75 => max_star_color = String::from("\u001b[0;31m"),
                    4.76..=6.00 => max_star_color = String::from("\u001b[0;35m"),
                    _ => max_star_color = String::from("\u001b[0;30;45m"),
                }
                let mut min_star_color = String::new();
                match min_star {
                    0.0..=1.75 => min_star_color = String::from("\u001b[0;36m"),
                    1.76..=2.75 => min_star_color = String::from("\u001b[0;32m"),
                    2.76..=3.75 => min_star_color = String::from("\u001b[0;33m"),
                    3.76..=4.75 => min_star_color = String::from("\u001b[0;31m"),
                    4.76..=6.00 => min_star_color = String::from("\u001b[0;35m"),
                    _ => min_star_color = String::from("\u001b[0;30;45m"),
                }
                let mut star_string = String::new();
                if max_star == min_star {
                    star_string = format!("{}{}★\u001b[0m", max_star_color, max_star);
                } else {
                    star_string = format!("{}{}★\u001b[0m~{}{}★\u001b[0m", min_star_color, min_star, max_star_color, max_star);
                }

                map_list.push_str(&format!("[{}] \u001b[1m{}\u001b[0m (by {}) {}\n",
                    map.id, map.title, map.artist, star_string));
            }
            map_list.push_str(&format!("```"));
            let channel_id: ChannelId = env::var("DISCORD_MAP_CHANNEL_ID").unwrap().parse().unwrap();
            let color = match status {
                &"ranked" => 0x00ff00,
                &"loved" => 0xff00ff,
                &"qualified" => 0xffff00,
                _ => 0x000000,
            };
            channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(format!("New {} maps", status))
                        .description(map_list)
                        .color(color)
                });
                m
            }).await?;
        } else {
            info!("{}", format!("No new {} maps", status));
        }

    }
    Ok(())
}