use reqwest::multipart::Form;
use reqwest::Client;
use reqwest::Error;
use serde::de::DeserializeOwned;
use serde::Deserialize;

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

    async fn call_api<T:DeserializeOwned>(&self, api_function: &str, params: Form) -> Result<T, Error> {
        //let token = self.token.clone();
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
            .map_err(|e| Error::from(e))
            
    }

    pub async fn get_sticker_pack(&self, pack_name: &str) -> Result<TelResponse<StickerSet>, Error> {
        let form = Form::new().text("name", pack_name.to_owned());
        self.call_api("getStickerSet", form).await
    }
}

#[derive(Deserialize, Debug)]
pub struct Sticker {
    pub file_id: String,
    width: i32,
    height: i32,
    pub emoji: Option<String>,
    set_name: Option<String>,
    file_size: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct StickerSet {
    name: String,
    title: String,
    contains_masks: bool,
    pub stickers: Vec<Sticker>,
}

#[derive(Deserialize, Debug)]
pub struct TelResponse<T> {
    ok: bool,
    result: Option<T>,
    error_code: Option<i32>,
    description: Option<String>,
}
