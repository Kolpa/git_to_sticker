use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Client;
use reqwest::Error;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::path::Path;

pub struct TelegramBot {
    client: Client,
    token: String,
}

impl TelegramBot {
    pub fn new(token: &str) -> TelegramBot {
        TelegramBot {
            client: Client::new(),
            token: token.to_owned(),
        }
    }

    async fn call_api<T: DeserializeOwned>(
        &self,
        api_function: &str,
        params: Form,
    ) -> Result<T, Error> {
        self.client
            .post(&format!(
                "https://api.telegram.org/bot{}/{}",
                self.token, api_function
            ))
            .multipart(params)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn add_sticker_to_set(
        &self,
        user_id: &str,
        pack_name: &str,
        sticker_path: &Path,
        emojis: &str,
    ) -> Result<TelResponse<bool>, Error> {
        let file_content = tokio::fs::read(sticker_path).await.unwrap();
        let file_part = Part::bytes(file_content);
        let form = Form::new()
            .text("user_id", user_id.to_owned())
            .text("name", pack_name.to_owned())
            .part("png_sticker", file_part)
            .text("emojis", emojis.to_owned());
        self.call_api("addStickerToSet", form).await
    }
}

#[derive(Deserialize, Debug)]
pub struct Sticker {
    pub file_id: String,
    pub width: i32,
    pub height: i32,
    pub emoji: Option<String>,
    pub set_name: Option<String>,
    pub file_size: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct StickerSet {
    pub name: String,
    pub title: String,
    pub contains_masks: bool,
    pub stickers: Vec<Sticker>,
}

#[derive(Deserialize, Debug)]
pub struct TelResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub error_code: Option<i32>,
    pub description: Option<String>,
}
