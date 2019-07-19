use git2::{Delta, Diff, DiffDelta, DiffFile, Error as GitError, Repository};
use serde::Deserialize;
use serde_json::error::Error as JsonError;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
mod telegram_api;
use dotenv::dotenv;

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
    let sticker_file: File = File::open("stickers.json").unwrap();
    serde_json::from_reader(sticker_file)
}

fn main() {
    dotenv().ok();
    let test: telegram_api::TelegramBot =
        telegram_api::TelegramBot::new(&env::var("BOT_TOKEN").unwrap());

    let test1 = test.get_sticker_pack("HPKaddi");

    print!("{:?}", test1);

    let repo = Repository::open(".").unwrap();

    let diff: Diff = parse_diff_from_repo(&repo).unwrap();

    let stickers: StickersObj = parse_sticker_json().unwrap();

    let pngs = diff
        .deltas()
        .filter(|delta: &DiffDelta| file_is_png(delta.new_file()));

    for png in pngs {
        if png.status() == Delta::Added {
            let file_path: &Path = png.new_file().path().unwrap();
            let _sticker: StickerObj = resolve_sticker_for_image(file_path, &stickers).unwrap();
        }
    }
}
