use git2::{Delta, Diff, DiffDelta, DiffFile, Error as GitError, Repository};
use serde::Deserialize;
use serde_json::error::Error as JsonError;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
mod telegram_api;
use dotenv::dotenv;
use log::{info, error};

#[derive(Deserialize, Clone)]
struct StickerObj {
    file: String,
    emoji: String,
}

#[derive(Deserialize)]
struct StickersObj {
    stickers: Vec<StickerObj>,
}

fn file_is_png(file: DiffFile) -> bool {
    file.path().map_or(false, |path: &Path| {
        path.extension().map_or(false, |ext: &OsStr| ext.eq("png"))
    })
}

fn resolve_sticker_for_image(path: &Path, stickers_obj: &StickersObj) -> Option<StickerObj> {
    for sticker in &stickers_obj.stickers {
        if path
            .file_name()
            .map_or(false, |name: &OsStr| name.eq(sticker.file.as_str()))
        {
            return Some(sticker.clone());
        }
    }

    None
}

fn parse_diff_from_repo(repo: &Repository) -> Result<Diff, GitError> {
    let tree = repo.revparse_single("HEAD~1^{tree}")?.peel_to_tree()?;
    repo.diff_tree_to_workdir_with_index(Some(&tree), None)
}

fn parse_sticker_json() -> Result<StickersObj, JsonError> {
    let sticker_json_path = "stickers.json";
    info!("Opening Sticker JSON at {}", sticker_json_path);
    let sticker_file: File = File::open(sticker_json_path).unwrap();
    serde_json::from_reader(sticker_file)
}

async fn add_file_to_pack(telegram: &telegram_api::TelegramBot, sticker_obj: StickerObj, file_path: &Path) -> bool {
    let pack_name = &env::var("PACK_NAME").unwrap();
    info!("Adding {} to Sticker Pack {}", sticker_obj.file, pack_name);
    telegram.add_sticker_to_set(
        &env::var("USER_ID").unwrap(),
        &env::var("PACK_NAME").unwrap(),
        file_path,
        &sticker_obj.emoji
    ).await.unwrap().ok
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let telegram_bot: telegram_api::TelegramBot =
        telegram_api::TelegramBot::new(&env::var("BOT_TOKEN").unwrap());

    //let sticker_pack = telegram_bot.get_sticker_pack("HPKaddi").await;

    let repo = Repository::open(".").unwrap();

    info!("Checking for added png files");
    let diff: Diff = parse_diff_from_repo(&repo).unwrap();

    let stickers: StickersObj = parse_sticker_json().unwrap();

    let pngs: Vec<DiffDelta> = diff
        .deltas()
        .filter(|delta: &DiffDelta| file_is_png(delta.new_file()))
        .collect();

    info!("{} new pngs found", pngs.len());
    for png in pngs {
        if png.status() == Delta::Added {
            let file_path: &Path = png.new_file().path().unwrap();
            let _sticker: Option<StickerObj> = resolve_sticker_for_image(file_path, &stickers);
            if _sticker.is_none() {
                error!("{} not found in Stickers JSON", file_path.to_str().unwrap_or("NONE"));
            }
            else {
                add_file_to_pack(&telegram_bot, _sticker.unwrap(), file_path).await;
            }
        }
    }
}
