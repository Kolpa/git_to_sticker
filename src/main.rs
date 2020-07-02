use git2::{Delta, Diff, DiffDelta, DiffFile, Error as GitError, Repository};
use serde::Deserialize;
use serde_json::error::Error as JsonError;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
mod telegram_api;
use dotenv::dotenv;
use log::{error, info};
use pretty_env_logger;

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

async fn add_file_to_pack(
    telegram: &telegram_api::TelegramBot,
    sticker_obj: StickerObj,
    file_path: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    let pack_name = &env::var("PACK_NAME")?;
    let user_id = &env::var("USER_ID")?;
    info!("Adding {} to Sticker Pack {}", sticker_obj.file, pack_name);
    let result = telegram
        .add_sticker_to_set(user_id, pack_name, file_path, &sticker_obj.emoji)
        .await?
        .ok;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let telegram_bot: telegram_api::TelegramBot =
        telegram_api::TelegramBot::new(&env::var("BOT_TOKEN")?);

    let repo = Repository::open(".")?;

    info!("Checking for added png files");
    let diff: Diff = parse_diff_from_repo(&repo)?;

    let stickers: StickersObj = parse_sticker_json()?;

    let pngs: Vec<DiffDelta> = diff
        .deltas()
        .filter(|delta: &DiffDelta| file_is_png(delta.new_file()))
        .collect();

    info!("{} new pngs found", pngs.len());
    for png in pngs {
        if png.status() == Delta::Added {
            let file_path: &Path = png.new_file().path().unwrap();
            let _sticker: Option<StickerObj> = resolve_sticker_for_image(file_path, &stickers);
            if let Some(sticker) = _sticker {
                add_file_to_pack(&telegram_bot, sticker, file_path).await?;
            } else {
                error!(
                    "{} not found in Stickers JSON",
                    file_path.to_str().unwrap_or("NONE")
                );
            }
        }
    }
    Ok(())
}
