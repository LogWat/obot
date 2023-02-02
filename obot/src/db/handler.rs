use std::error::Error;

use serenity::{
    prelude::*,
};

use std::sync::{Arc};

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

    pub async fn insert(&self, status: &str, beatmapset: &Beatmap) -> Result<(), Box<dyn Error>> {
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

    // select_by: select beatmapset by id, title, artist, creator, or cursor (stars is other method)
    // キー数で指定しちゃうと量が多すぎるので，keyは
    pub async fn select(&self, select_by: &str, status: &str, value: &str) -> Result<Vec<Beatmap>, Box<dyn Error>> {
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
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "loved" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM loved_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "qualified" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM qualified_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "graveyard" => {
                match select_by {
                    "id" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE id = ?", value).fetch_all(&*db).await,
                    "title" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE title = ?", value).fetch_all(&*db).await,
                    "artist" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE artist = ?", value).fetch_all(&*db).await,
                    "creator" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE creator = ?", value).fetch_all(&*db).await,
                    "cursor" => sqlx::query_as!(Beatmap, "SELECT * FROM graveyard_beatmapsets WHERE cursor = ?", value).fetch_all(&*db).await,
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid status"))),
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

    // select_top: select by specifying the amount of top beatmaps to be selected
    // select_by: * (all), keys
    // [!] too many selections shouldn't be made
    pub async fn select_top(&self, status: &str, select_by: &str, keys: &str, range: &str) -> Result<Vec<Beatmap>, Box<dyn Error>> {
        let db = self.db.lock().await;
        let mut beatmapsets = Vec::new();
        
        let res = match status {
            "ranked" => {
                match select_by {
                    "*" => sqlx::query!("SELECT * FROM ranked_beatmapsets ORDER BY stars DESC LIMIT ?", range).fetch_all(&*db).await,
                    "keys" => sqlx::query!("SELECT * FROM (SELECT * FROM ranked_beatmapsets WHERE keys = ?) ORDER BY stars DESC LIMIT ?", keys, range).fetch_all(&*db).await,
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "loved" => {
                match select_by {
                    "*" => sqlx::query!("SELECT * FROM loved_beatmapsets ORDER BY stars DESC LIMIT ?", range).fetch_all(&*db).await,
                    "keys" => sqlx::query!("SELECT * FROM loved_beatmapsets ORDER BY keys DESC LIMIT ?", range).fetch_all(&*db).await,
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "qualified" => {
                match select_by {
                    "*" => sqlx::query!("SELECT * FROM qualified_beatmapsets ORDER BY stars DESC LIMIT ?", range).fetch_all(&*db).await,
                    "keys" => sqlx::query!("SELECT * FROM qualified_beatmapsets ORDER BY keys DESC LIMIT ?", range).fetch_all(&*db).await,
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            "graveyard" => {
                match select_by {
                    "*" => sqlx::query!("SELECT * FROM graveyard_beatmapsets ORDER BY stars DESC LIMIT ?", range).fetch_all(&*db).await,
                    "keys" => sqlx::query!("SELECT * FROM graveyard_beatmapsets ORDER BY keys DESC LIMIT ?", range).fetch_all(&*db).await,
                    _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid select_by"))),
                }
            },
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid status"))),
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