use serenity::{
    prelude::*,
};

use std::sync::{Arc};
use std::io::{Error, ErrorKind};

use crate::cache::Database;
use crate::web::api;
use api::Beatmap;

pub struct DBHandler {
    db: Arc<Mutex<sqlx::SqlitePool>>,
}

impl DBHandler {
    pub async fn new(ctx: &Context) -> Self {
        let data = ctx.data.read().await;
        let db = data.get::<Database>().unwrap().clone();
        Self { db }
    }

    pub async fn insert(&self, status: &str, beatmapset: &Beatmap) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().await;

        match status {
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

    fn get_beatmapsets_with_key(&self, beatmapsets: Vec<Beatmap>, key: &str) -> Vec<Beatmap> {
        let mut beatmapsets_with_key = Vec::new();
        for beatmapset in beatmapsets {
            if beatmapset.keys.contains(key) {
                beatmapsets_with_key.push(beatmapset);
            }
        }
        beatmapsets_with_key
    }

    // select_by: select beatmapset by id, title, artist, creator, or cursor (stars is other method)
    // keysは"4, 7"のようにカンマ区切りで格納されているので，"4"とか"7"が含まれているかどうかで検索する必要がある
    pub async fn select(&self, select_by: &str, status: &str, value: &str) -> Result<Vec<Beatmap>, Box<dyn std::error::Error>> {
        let db = self.db.lock().await;
        let mut beatmapsets = Vec::new();
        let res = match status {
            "ranked" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => {
                        let beatmapsets_tmp = sqlx::query_as!(Beatmap, "SELECT * FROM ranked_beatmapsets").fetch_all(&*db).await?;
                        Ok(self.get_beatmapsets_with_key(beatmapsets_tmp, value))
                    },
                    _ => return Err(Box::new(Error::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "loved" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => {
                        let beatmapsets_tmp = sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets").fetch_all(&*db).await?;
                        Ok(self.get_beatmapsets_with_key(beatmapsets_tmp, value))
                    },
                    _ => return Err(Box::new(Error::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "qualified" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => {
                        let beatmapsets_tmp = sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets").fetch_all(&*db).await?;
                        Ok(self.get_beatmapsets_with_key(beatmapsets_tmp, value))
                    },
                    _ => return Err(Box::new(Error::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "graveyard" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    "keys" => {
                        let beatmapsets_tmp = sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets").fetch_all(&*db).await?;
                        Ok(self.get_beatmapsets_with_key(beatmapsets_tmp, value))
                    },
                    _ => return Err(Box::new(Error::new(ErrorKind::Other, "Invalid select_by"))),
                }
            },
            _ => return Err(Box::new(Error::new(ErrorKind::Other, "Invalid status"))),
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
}