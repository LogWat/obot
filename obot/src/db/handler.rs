#![allow(dead_code)]
use serenity::{
    prelude::*,
};

use std::sync::{Arc};
use std::io::{Error as StdError, ErrorKind};
use std::error::Error;

use crate::cache::Database;
use crate::web::api;
use api::Beatmap;

pub struct DBHandler {
    db: Arc<Mutex<sqlx::SqlitePool>>,
}

// TODO: statusごとにSQL文作ってる問題を解決する
// TODO: cursor_stirng の更新処理
impl DBHandler {
    pub async fn new(ctx: &Context) -> Self {
        let data = ctx.data.read().await;
        let db = data.get::<Database>().unwrap().clone();
        Self { db }
    }

    pub async fn insert(&self, beatmapset: &Beatmap) -> Result<(), Box<dyn Error + Sync + Send>> {
        let db = self.db.lock().await;

        match beatmapset.statu.as_str() {
            "ranked" => {
                match sqlx::query!(r#"
                INSERT INTO ranked_beatmapsets
                (id, title, artist, creator, stars, keys, mp3_url, card_url, cursor, statu)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                beatmapset.id, beatmapset.title, beatmapset.artist, beatmapset.creator, beatmapset.stars, beatmapset.keys, beatmapset.mp3_url, beatmapset.card_url, beatmapset.cursor, beatmapset.statu
                ).execute(&*db).await {
                    Ok(_) => {},
                    Err(e) => return Err(Box::new(e)),
                }
            },
            "loved" => {
                match sqlx::query!(r#"
                INSERT INTO loved_beatmapsets
                (id, title, artist, creator, stars, keys, mp3_url, card_url, cursor, statu)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                beatmapset.id, beatmapset.title, beatmapset.artist, beatmapset.creator, beatmapset.stars, beatmapset.keys, beatmapset.mp3_url, beatmapset.card_url, beatmapset.cursor, beatmapset.statu
                ).execute(&*db).await {
                    Ok(_) => {},
                    Err(e) => return Err(Box::new(e)),
                }
            },
            "qualified" => {
                match sqlx::query!(r#"
                INSERT INTO qualified_beatmapsets
                (id, title, artist, creator, stars, keys, mp3_url, card_url, cursor, statu)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                beatmapset.id, beatmapset.title, beatmapset.artist, beatmapset.creator, beatmapset.stars, beatmapset.keys, beatmapset.mp3_url, beatmapset.card_url, beatmapset.cursor, beatmapset.statu
                ).execute(&*db).await {
                    Ok(_) => {},
                    Err(e) => return Err(Box::new(e)),
                }
            },
            _ => {
                match sqlx::query!(r#"
                INSERT INTO graveyard_beatmapsets
                (id, title, artist, creator, stars, keys, mp3_url, card_url, cursor, statu)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                beatmapset.id, beatmapset.title, beatmapset.artist, beatmapset.creator, beatmapset.stars, beatmapset.keys, beatmapset.mp3_url, beatmapset.card_url, beatmapset.cursor, beatmapset.statu
                ).execute(&*db).await {
                    Ok(_) => {},
                    Err(e) => return Err(Box::new(e)),
                }
            },
        }

        Ok(())
    }

    pub async fn get_db_size(&self, status: &str, key: &str) -> Result<i32, Box<dyn Error + Sync + Send>> {
        let db = self.db.lock().await;
        let key_str = format!("%{}%", key);
        let size: i32 = match status {
            "ranked" => {
                match sqlx::query!("SELECT COUNT(*) as count FROM ranked_beatmapsets WHERE keys LIKE ?", key_str).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
            "loved" => {
                match sqlx::query!("SELECT COUNT(*) as count FROM loved_beatmapsets WHERE keys LIKE ?", key_str).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
            "qualified" => {
                match sqlx::query!("SELECT COUNT(*) as count FROM qualified_beatmapsets WHERE keys LIKE ?", key_str).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
            _ => {
                match sqlx::query!("SELECT COUNT(*) as count FROM graveyard_beatmapsets WHERE keys LIKE ?", key_str).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
        };

        Ok(size)
    }

    pub async fn check_existence(&self, id: &i64, status: &str) -> Result<bool, Box<dyn Error + Sync + Send>> {
        let db = self.db.lock().await;
        let res = match status {
            "ranked" => {
                match sqlx::query!("SELECT COUNT(*) as count FROM ranked_beatmapsets WHERE id = ?", id).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
            "loved" => {
                match sqlx::query!("SELECT COUNT(*) as count FROM loved_beatmapsets WHERE id = ?", id).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
            "qualified" => {
                match sqlx::query!("SELECT COUNT(*) as count FROM qualified_beatmapsets WHERE id = ?", id).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
            _ => {
                match sqlx::query!("SELECT COUNT(*) as count FROM graveyard_beatmapsets WHERE id = ?", id).fetch_one(&*db).await {
                    Ok(r) => r.count,
                    Err(e) => return Err(Box::new(e)),
                }
            },
        };

        if res == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    // select_by: select beatmapset by id, title, artist, creator, or cursor (stars is other method)
    // keysは"4, 7"のようにカンマ区切りで格納されているので，"4"とか"7"が含まれているかどうかで検索する必要がある
    pub async fn select(&self, select_by: &str, status: &str, value: &str) -> Result<Vec<Beatmap>, Box<dyn Error + Sync + Send>> {
        let db = self.db.lock().await;
        let mut beatmapsets = Vec::new();
        let key_str = format!("%{}%", value);
        let res = match status {
            "ranked" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE keys LIKE ?", key_str).fetch_all(&*db).await,
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets").fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "loved" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE keys LIKE ?", key_str).fetch_all(&*db).await,
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets").fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "qualified" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE keys LIKE ?", key_str).fetch_all(&*db).await,
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets").fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "graveyard" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE keys LIKE ?", key_str).fetch_all(&*db).await,
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets").fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid status"))),
        };

        match res {
            Ok(res) => {
                for beatmapset in res {
                    beatmapsets.push(beatmapset);
                }
            },
            Err(e) => return Err(Box::new(e)),
        }

        Ok(beatmapsets)
    }


    // select_by: keys, stars
    pub async fn select_with_limit(&self, select_by: &str, status: &str, value: &str, limit: i64, offset: i64) -> Result<Vec<Beatmap>, Box<dyn Error + Sync + Send>> {
        let db = self.db.lock().await;
        let key_str = format!("%{}%", value);

        let res = match status {
            "ranked" => {
                match select_by {
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets LIMIT ? OFFSET ?", limit, offset).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE keys LIKE ? LIMIT ? OFFSET ?", key_str, limit, offset).fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "loved" => {
                match select_by {
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets LIMIT ? OFFSET ?", limit, offset).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE keys LIKE ? LIMIT ? OFFSET ?", key_str, limit, offset).fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "qualified" => {
                match select_by {
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets LIMIT ? OFFSET ?", limit, offset).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE keys LIKE ? LIMIT ? OFFSET ?", key_str, limit, offset).fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            _ => {
                match select_by {
                    "*" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets LIMIT ? OFFSET ?", limit, offset).fetch_all(&*db).await,
                    "keys" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE keys LIKE ? LIMIT ? OFFSET ?", key_str, limit, offset).fetch_all(&*db).await,
                    _ => return Err(Box::new(StdError::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
        };

        match res {
            Ok(res) => return Ok(res),
            Err(e) => return Err(Box::new(e)),
        }
    }
}