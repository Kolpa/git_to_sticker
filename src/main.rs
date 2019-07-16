use git2::Repository;

fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let rev = match repo.revparse_single("HEAD~1") {
        Ok(rev) => rev,
        Err(e) => panic!("failed to parse Rev HEAD~1: {}", e),
    };

    let commit = match rev.as_commit() {
        Some(commit) => commit,
        None => panic!("failed to grab Commit"),
    };

    let tree = match commit.tree() {
        Ok(tree) => tree,
        Err(e) => panic!("could not grab Tree for Commit: {}", e),
    };

    let diff = match repo.diff_tree_to_workdir_with_index(Some(&tree), None) {
        Ok(diff) => diff,
        Err(e) => panic!("failed to get Diff: {}", e),
    };

    if let Err(e) = diff.foreach(
        &mut |file, _progress| {
            println!(
                "File: {:?} Change type: {:?}",
                file.old_file().path(),
                file.status()
            );
            true
        },
        None,
        None,
        None,
    ) {
        panic!("failed to list Files: {}", e);
    };
}
