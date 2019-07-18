use git2::{Delta, Diff, DiffDelta, DiffFile, Object, Repository, Tree};
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize, Clone)]
struct Sticker {
    file: String,
    emoji: String,
}

#[derive(Deserialize)]
struct Stickers {
    stickers: Vec<Sticker>,
}

fn file_is_png(file: DiffFile) -> bool {
    file.path().map_or(false, |path: &Path| {
        path.extension().map_or(false, |ext: &OsStr| ext.eq("png"))
    })
}

//TODO: This is quite Broken
fn resolve_sticker_for_image(path: &Path, stickers_obj: &Stickers) -> Option<Sticker> {
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

fn main() {
    let repo = Repository::open(".").expect("Repository not found");

    let diff: Diff = repo
        .revparse_single("HEAD~1^{tree}")
        .and_then(|rev: Object| rev.peel_to_tree())
        .and_then(|tree: Tree| repo.diff_tree_to_workdir_with_index(Some(&tree), None))
        .expect("Creating diff from tree to working dir not possible");

    let pngs = diff
        .deltas()
        .filter(|delta: &DiffDelta| file_is_png(delta.new_file()));

    let file: File = File::open("stickers.json").expect("Could not open File stickers.json");
    let stickers: Stickers =
        serde_json::from_reader(file).expect("Could not parse File stickers.json");

    for png in pngs {
        if png.status() == Delta::Added {
            let filePath: &Path = png
                .new_file()
                .path()
                .expect("Could not get File Path from Diff");

            resolve_sticker_for_image(filePath, &stickers);
        }
    }
}
