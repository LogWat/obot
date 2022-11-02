use std::{
    env,
    collections::{HashMap},
    error::Error,
};

use serde_json::{Value};

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

    pub async fn get_token(&self) -> Result<String, Box<dyn Error>> {
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
}
