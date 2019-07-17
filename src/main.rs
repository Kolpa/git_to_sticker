use git2::Diff;
use git2::DiffDelta;
use git2::Object;
use git2::Repository;
use git2::Tree;

fn main() {
    let repo = Repository::open(".").expect("Repository not found");
    let diff: Diff = repo
        .revparse_single("HEAD~1^{tree}")
        .and_then(|rev: Object| rev.peel_to_tree())
        .and_then(|tree: Tree| repo.diff_tree_to_workdir_with_index(Some(&tree), None))
        .expect("Creating diff from tree to working dir not possible");

    diff.deltas().for_each(|file: DiffDelta| {
        println!(
            "File: {:?} Change type: {:?}",
            file.old_file().path(),
            file.status()
        )
    });
}
