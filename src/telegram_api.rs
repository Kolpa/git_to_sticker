extern crate reqwest;
extern crate serde;
extern crate serde_json;

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

    fn call_api<T: DeserializeOwned>(&self, api_function: &str, params: Form) -> Result<T, Error> {
        let token = self.token.clone();
        self.client
            .post(&format!(
                "https://api.telegram.org/bot{}/{}",
                token, api_function
            ))
            .multipart(params)
            .send()?
            .json()
            .map_err(|e| Error::from(e))
    }

    pub fn get_sticker_pack(&self, pack_name: &str) -> Result<TelResponse<StickerSet>, Error> {
        let form = Form::new().text("name", pack_name.to_owned());
        self.call_api("getStickerSet", form)
    }
}

#[derive(Deserialize, Debug)]
pub struct Sticker {
    file_id: String,
    width: i32,
    height: i32,
    emoji: Option<String>,
    set_name: Option<String>,
    file_size: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct StickerSet {
    name: String,
    title: String,
    contains_masks: bool,
    stickers: Vec<Sticker>,
}

#[derive(Deserialize, Debug)]
pub struct TelResponse<T> {
    ok: bool,
    result: Option<T>,
    error_code: Option<i32>,
    description: Option<String>,
}
