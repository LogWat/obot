#![allow(dead_code)]

use std::{
    collections::{HashMap},
    error::Error,
    fs::File,
    mem,
};
use futures::future;
use serde_json::{Value};
use serenity::prelude::*;

use crate::utility;

#[derive(Debug, Clone)]
pub struct Beatmap {
    pub id: i64, // 再検索時に必要
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub stars: String, // ","で区切って文字列にして保存
    pub keys: String,  // starと同じ(順番は揃えること！)
    pub mp3_url: String,
    pub card_url: String,
    pub cursor: String,
    pub statu: String, // db的には不要
}


#[derive(Debug, Clone)]
pub struct Api {
    pub http: reqwest::Client,
    pub user_id: u64,
    pub secret: String,
    pub base_url: String,
    pub download_base_url: String,
    token: String,
}

pub async fn get_url(ctx: &Context, beatmap: &Beatmap) -> String {
    let base_url = utility::get_env_from_context(ctx, "api_base").await;
    format!("{}/beatmapsets/{}", base_url, beatmap.id)
}

impl Api {
    pub async fn new(ctx: &Context) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let secret = utility::get_env_from_context(ctx, "api_secret").await;
        let user_id = utility::get_env_from_context(ctx, "user_id").await.parse::<u64>()?;
        let base_url = utility::get_env_from_context(ctx, "api_base").await;
        let download_base_url = utility::get_env_from_context(ctx, "download_base").await;
        
        // get token
        let url = format!("{}/oauth/token", base_url);
        let mut params = HashMap::new();
        params.insert("client_id", user_id.to_string());
        params.insert("client_secret", secret.to_string());
        params.insert("grant_type", "client_credentials".to_string());
        params.insert("scope", "public".to_string());
        let token = match reqwest::Client::new().post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await {
                Ok(res) => {
                    match res.text().await {
                        Ok(text) => {
                            let json: Value = serde_json::from_str(&text)?;
                            let token = json["access_token"].as_str().unwrap();
                            token.to_string()
                        },
                        Err(e) => return Err(Box::new(e)),
                    }
                }
                Err(e) => return Err(Box::new(e)),
            };

        let http = reqwest::Client::new();
        let api = Api {
            http,
            user_id,
            secret,
            base_url,
            download_base_url,
            token,
        };

        Ok(api)
    }

    pub async fn get_beatmapsets_with_cursor(
        &self,
        mode: &str, // mania: 3
        status: &str, // ranked, loved, qualified, pending, graveyard
        key: &str, // 4, 7...
        cursor_string: &str,
    ) -> Result<(Vec<Beatmap>, String), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/beatmapsets/search?m={}&s={}&q=key%3D{}&nsfw=&cursor_string={}",
        self.base_url, mode, status, key, cursor_string);
        let text = match self.req_with_token(&url).await {
            Ok(text) => text,
            Err(e) => return Err(e),
        };
        let beatmapsets = match self.text2beatmapsets(&text) {
            Ok((beatmapsets, cursor)) => {
                let mut mapsets = beatmapsets.clone();
                mapsets.reverse(); // 新しいものを配列中で最後尾にする => DBへの追加順を考慮
                (mapsets, cursor)
            },
            Err(e) => return Err(e.to_string().into()),
        };

        Ok(beatmapsets)
    }

    // idからbeatmapset構造体を取得
    pub async fn get_beatmaps_by_ids(
        &self,
        ids: Vec<String>
    ) -> Result<Vec<Beatmap>, Box<dyn Error + Send + Sync>> {
        let mut bmsets = Vec::new();
        for id in ids {
            let url = &format!("{}/api/v2/beatmapsets/{}", self.base_url, id);
            let text = match self.req_with_token(&url).await {
                Ok(text) => text,
                Err(e) => {
                    error!("req_with_token error: {}", e);
                    continue;
                }
            };
            let beatmapsets = match self.text2beatmapsets(&text) {
                Ok((beatmapsets, _)) => beatmapsets,
                Err(e) => {
                    error!("text2beatmapsets error: {}", e);
                    continue;
                }
            };
            bmsets.extend(beatmapsets);
        }
        Ok(bmsets)
    }

    // 譜面download用method
    pub async fn download_beatmaps(
        &self,
        maps: Vec<Beatmap>,
        path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut tasks = Vec::new();
        for (i, beatmapset) in maps.iter().enumerate() {
            tasks.push(self.download(beatmapset, path));
            if (i + 1) % 5 == 0 || i == maps.len() - 1 {
                // 5つずつ
                // TODO: この不細工な実装をどうにかする
                let mut tasks_copy = Vec::new();
                mem::swap(&mut tasks, &mut tasks_copy);
                let ret = future::join_all(tasks_copy)
                    .await
                    .into_iter()
                    .collect::<Result<Vec<()>, Box<dyn Error + Send + Sync>>>();
                match ret {
                    Ok(_) => (),
                    Err(e) => warn!("download was failed: {}", e),
                }
                tasks.clear();
            }
        }

        Ok(())
    }

    // private
    async fn req_with_token(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let res = self.http.get(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;
        let text = res.text().await?;
        Ok(text)
    }

    // private
    async fn download(&self, beatmapset: &Beatmap, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/{}?n=1", self.download_base_url, beatmapset.id);
        let res = self.http.get(&url).send().await?;
        let _ = std::fs::create_dir_all(format!("{}{}", path, beatmapset.statu));
        let mut file = File::create(format!("{}{}/{}-{}.osz", path, beatmapset.statu, beatmapset.id, beatmapset.title))?;
        let mut content = std::io::Cursor::new(res.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
        Ok(())
    }

    fn text2beatmapsets(&self, text: &str) -> Result<(Vec<Beatmap>, String), Box<dyn Error>> {
        let json: Value = serde_json::from_str(&text)?;
        let mut beatmapsets = Vec::new();
    
        if let Some(_) = json["artist"].as_str() {
            beatmapsets.push(&json);
        } else {
            for beatmapset in json["beatmapsets"].as_array().unwrap() {
                beatmapsets.push(beatmapset);
            }
        }

        let cursor_string = match json["cursor_string"].as_str() {
            Some(cursor_string) => cursor_string.to_string(),
            None => "".to_string(),
        };
    
        let beatmapsets = beatmapsets.iter().map(|beatmapset| {
            let title = beatmapset["title"].as_str().unwrap();
            let artist = beatmapset["artist"].as_str().unwrap();
            let creator = beatmapset["creator"].as_str().unwrap();
            let card_url = beatmapset["covers"]["card@2x"].as_str().unwrap();
            let mp3_url = format!("https:{}",
                beatmapset["preview_url"].as_str().unwrap()
            );
            let id = beatmapset["id"].as_u64().unwrap();
            let beatmaps = beatmapset["beatmaps"].as_array().unwrap();
            let status = beatmaps[0]["status"].as_str().unwrap();
            let star = beatmaps.iter().map(|beatmap| {
                beatmap["difficulty_rating"].as_f64().unwrap() as f64
            }).collect::<Vec<f64>>();
            let key = beatmaps.iter().map(|beatmap| {
                beatmap["cs"].as_f64().unwrap() as f64
            }).collect::<Vec<f64>>();

            let mut star_str = String::new();
            let mut key_str = String::new();
            for (s, k) in star.iter().zip(key.iter()) {
                star_str.push_str(&format!("{},", s));
                key_str.push_str(&format!("{},", k));
            }
            star_str = star_str[0..star_str.len() - 1].to_string();
            key_str = key_str[0..key_str.len() - 1].to_string();

            Beatmap {
                id: id as i64,
                title: title.to_string(),
                artist: artist.to_string(),
                creator: creator.to_string(),
                stars: star_str,
                keys: key_str,
                card_url: card_url.to_string(),
                mp3_url,
                cursor: cursor_string.to_string(),
                statu: status.to_string(),
            }
        }).collect::<Vec<Beatmap>>();
        Ok((beatmapsets, cursor_string))
    }

}
