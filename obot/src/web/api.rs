#![allow(dead_code)]

use std::{
    env,
    collections::{HashMap},
    error::Error,
    fs::File,
    mem,
};
use futures::future;
use serde_json::{Value};

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
}

pub fn get_url(beatmap: &Beatmap) -> String {
    let base_url = env::var("API_BASE").expect("API_BASE must be set");
    format!("{}/beatmapsets/{}", base_url, beatmap.id)
}

impl Api {
    pub fn new() -> Self {
        let secret = env::var("API_SECRET").expect("API_SECRET must be set");
        let user_id = env::var("USER_ID").expect("USER_ID must be set").parse::<u64>().unwrap();
        let base_url = env::var("API_BASE").expect("API_BASE must be set");
        let download_base_url = env::var("DOWNLOAD_BASE").expect("DOWNLOAD_BASE must be set");
        let http = reqwest::Client::new();
        Self {
            http,
            user_id,
            secret,
            base_url,
            download_base_url,
        }
    }

    pub async fn update_token(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/oauth/token", self.base_url);
        let mut params = HashMap::new();
        params.insert("client_id", self.user_id.to_string());
        params.insert("client_secret", self.secret.to_string());
        params.insert("grant_type", "client_credentials".to_string());
        params.insert("scope", "public".to_string());
        let token = match self.http.post(&url)
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
                            Ok(token.to_string())
                        },
                        Err(e) => return Err(Box::new(e)),
                    }
                }
                Err(e) => return Err(Box::new(e)),
            };

        token
    }

    pub async fn get_beatmapsets_with_cursor(
        &self,
        token: &str,
        mode: &str, // mania: 3
        status: &str, // ranked, loved, qualified, pending, graveyard
        key: &str, // 4, 7...
        cursor_string: &str,
    ) -> Result<(Vec<Beatmap>, String), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/beatmapsets/search?m={}&s={}&q=key%3D{}&nsfw=&cursor_string={}",
        self.base_url, mode, status, key, cursor_string);
        let text = match self.req_with_token(&token, &url).await {
            Ok(text) => text,
            Err(e) => return Err(e),
        };
        let beatmapsets = match self.text2beatmapsets(&text) {
            Ok((beatmapsets, cursor)) => (beatmapsets, cursor),
            Err(e) => return Err(e.to_string().into()),
        };

        Ok(beatmapsets)
    }

    pub async fn get_beatmaps(
        &self, 
        token: &str,
        mode: &str,
        status: &str,
        key: &str,
        all_flag: bool,
    ) -> Result<Vec<Beatmap>, Box<dyn Error + Send + Sync>> {
        let mut bmsets = Vec::new();
        let mut cursor_string = String::new();
        let mut url = format!("{}/api/v2/beatmapsets/search?m={}&s={}&q=key%3D{}&nsfw=&cursor_string={}",
        self.base_url, mode, status, key, cursor_string);

        loop {
            let text = match self.req_with_token(&token, &url).await {
                Ok(text) => text,
                Err(e) => {
                    error!("Failed to get beatmaps: {}", e);
                    break;
                }
            };
            let beatmapsets = match self.text2beatmapsets(&text) {
                Ok((beatmapsets, cursor)) => {
                    cursor_string = cursor;
                    beatmapsets
                },
                Err(e) => {
                    error!("Failed to parse beatmaps: {}", e);
                    break;
                }
            };
            bmsets.extend(beatmapsets);
            if all_flag {
                url = format!("{}/api/v2/beatmapsets/search?m={}&s={}&q=key%3D4&nsfw=&cursor_string={}", self.base_url, mode, status, cursor_string);
            } else {
                break;
            }
            if cursor_string.is_empty() {
                break;
            }
        }

        Ok(bmsets)
    }

    // idからbeatmapset構造体を取得
    pub async fn get_beatmaps_by_ids(
        &self, 
        token: &str,
        ids: Vec<String>
    ) -> Result<Vec<Beatmap>, Box<dyn Error + Send + Sync>> {
        let mut bmsets = Vec::new();
        for id in ids {
            let url = &format!("{}/api/v2/beatmapsets/{}", self.base_url, id);
            let text = match self.req_with_token(&token, &url).await {
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
        token: &str,
        url: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let res = self.http.get(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
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
            let favourite_count = beatmapset["favourite_count"].as_u64().unwrap();
            let beatmaps = beatmapset["beatmaps"].as_array().unwrap();
            let status = beatmaps[0]["status"].as_str().unwrap();
            let star = beatmaps.iter().map(|beatmap| {
                beatmap["difficulty_rating"].as_f64().unwrap() as f32
            }).collect::<Vec<f32>>();
            let key = beatmaps.iter().map(|beatmap| {
                beatmap["cs"].as_f64().unwrap() as f32
            }).collect::<Vec<f32>>();

            let mut star_str = String::new();
            let mut key_str = String::new();
            for (s, k) in star.iter().zip(key.iter()) {
                star_str.push_str(&format!("{}, ", s));
                key_str.push_str(&format!("{}, ", k));
            }
            star_str.pop();
            star_str.pop();

            Beatmap {
                id: id as i64,
                title: title.to_string(),
                artist: artist.to_string(),
                creator: creator.to_string(),
                stars: star_str,
                keys: key_str,
                card_url: card_url.to_string(),
                mp3_url,
                cursor: cursor_string,
                statu: status.to_string(),
            }
        }).collect::<Vec<Beatmap>>();
        Ok((beatmapsets, cursor_string))
    }

}
