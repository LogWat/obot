#![allow(dead_code)]

use std::{
    env,
    collections::{HashMap},
    error::Error,
};

use serde_json::{Value};

pub struct Beatmap {
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub cover_url: String,
    pub id: u32, // 再検索時に必要
    pub favourite_count: u32,
    pub mode: String,
    pub status: String,
    pub star: Vec<f32>,
}


#[derive(Debug)]
pub struct Api {
    pub http: reqwest::Client,
    pub user_id: u64,
    pub secret: String,
    pub base_url: String,
}

impl Api {
    pub fn new() -> Self {
        let secret = env::var("API_KEY").expect("GAME_SECRET must be set");
        let user_id = env::var("USER_ID").expect("USER_ID must be set").parse::<u64>().unwrap();
        let base_url = env::var("BASE_URL").expect("BASE_URL must be set");
        let http = reqwest::Client::new();
        Self {
            http,
            user_id,
            secret,
            base_url,
        }
    }

    pub async fn update_token(&self) -> Result<String, Box<dyn Error>> {
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

    pub async fn get_beatmaps(
        &self,
        token: &str,
        mode: &str,
        status: &str,
    ) -> Result<Vec<Beatmap>, Box<dyn Error>> {
        let url = format!("{}/beatmapsets/search?m={}&s={}&q=key%3D4", self.base_url, mode, status);
        
        let beatmaps = match self.http.get(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await {
                Ok(res) => {
                    match res.text().await {
                        Ok(text) => {
                            let json: Value = serde_json::from_str(&text)?;
                            let beatmapsets = json["beatmapsets"].as_array().unwrap();
                            let beatmapsets = beatmapsets.iter().map(|beatmapset| {
                                let title = beatmapset["title"].as_str().unwrap();
                                let artist = beatmapset["artist"].as_str().unwrap();
                                let creator = beatmapset["creator"].as_str().unwrap();
                                let cover_url = beatmapset["covers"]["cover"].as_str().unwrap();
                                let id = beatmapset["id"].as_u64().unwrap();
                                let favourite_count = beatmapset["favourite_count"].as_u64().unwrap();
                                let beatmaps = beatmapset["beatmaps"].as_array().unwrap();
                                let mode = beatmaps[0]["mode"].as_str().unwrap();
                                let status = beatmaps[0]["status"].as_str().unwrap();
                                let star = beatmaps.iter().map(|beatmap| {
                                    beatmap["difficulty_rating"].as_f64().unwrap() as f32
                                }).collect::<Vec<f32>>();
                                Beatmap {
                                    title: title.to_string(),
                                    artist: artist.to_string(),
                                    creator: creator.to_string(),
                                    cover_url: cover_url.to_string(),
                                    id: id as u32,
                                    favourite_count: favourite_count as u32,
                                    mode: mode.to_string(),
                                    status: status.to_string(),
                                    star,
                                }
                            }).collect::<Vec<Beatmap>>();
                            Ok(beatmapsets)
                        },
                        Err(e) => return Err(Box::new(e)),
                    }
                }
                Err(e) => return Err(Box::new(e)),
            };

        beatmaps
    }
}
