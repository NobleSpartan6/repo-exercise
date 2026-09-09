//! Explicit, single-file Trash reviews for Analyze. This does not widen cache discovery.
use crate::{
    engine::{self, Cleanup, Control},
    preferences::Preferences,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

const REVIEW_LIFETIME: Duration = Duration::from_secs(300);

/// Only `review` can create this capability. Display access is read-only so UI
/// state cannot redirect a previously reviewed action to another file.
#[derive(Clone, Debug)]
pub struct FileReview {
    root: PathBuf,
    path: PathBuf,
    stamp: Stamp,
    reviewed: Instant,
}
impl FileReview {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn bytes(&self) -> u64 {
        self.stamp.bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Stamp {
    bytes: u64,
    modified: SystemTime,
    created: Option<SystemTime>,
    #[cfg(unix)]
    identity: (u64, u64),
}
impl Stamp {
    fn read(path: &Path) -> Result<Self, String> {
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !meta.is_file() || engine::is_link_or_placeholder(&meta) {
            return Err("Only a regular local file can be reviewed. Folders, links and cloud placeholders are excluded.".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if meta.nlink() != 1 {
                return Err("Hard-linked files are excluded.".into());
            }
        }
        Ok(Self {
            bytes: meta.len(),
            modified: meta.modified().map_err(|e| e.to_string())?,
            created: meta.created().ok(),
            #[cfg(unix)]
            identity: {
                use std::os::unix::fs::MetadataExt;
                (meta.dev(), meta.ino())
            },
        })
    }
}

fn location(
    root: &Path,
    path: &Path,
    home: &Path,
    preferences: &Preferences,
) -> Result<(PathBuf, PathBuf), String> {
    let root = engine::checked_path(root)?;
    let path = engine::checked_path(path)?;
    let home = engine::checked_path(home)?;
    if !root.is_dir() || path == root || !path.starts_with(&root) || !path.starts_with(&home) {
        return Err("Analyze can move individual files inside your home folder only. Use the system file manager for other locations.".into());
    }
    if preferences.excludes(&path)
        || preferences.protected.iter().any(|p| p.canonicalize().is_ok_and(|p| path.starts_with(p)))
    {
        return Err("This file is inside a protected folder. Nothing was moved.".into());
    }
    if path.ancestors().any(|p| {
        p.extension().is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("app"))
    }) {
        return Err("Use Apps to review an app bundle; Analyze cannot remove files inside it.".into());
    }
    // Never turn this action into a way to empty Trash or remove Burrow's own
    // protection settings. These locations are intentionally not user-file cleanup.
    for reserved in [home.join(".Trash"), home.join(".local/share/Trash")] {
        if path.starts_with(reserved) {
            return Err("Existing Trash contents are not eligible.".into());
        }
    }
    if let Some(data) = dirs::data_local_dir()
        && data.join("Burrow").canonicalize().is_ok_and(|p| path.starts_with(p))
    {
        return Err("Burrow's saved settings and cleanup totals are protected.".into());
    }
    if std::env::current_exe().and_then(|p| p.canonicalize()).is_ok_and(|p| p == path) {
        return Err("Burrow cannot remove its running executable.".into());
    }
    Ok((root, path))
}

fn review_in(
    root: &Path,
    path: &Path,
    home: &Path,
    preferences: &Preferences,
    control: &Control,
) -> Result<FileReview, String> {
    if control.cancelled() {
        return Err("File review stopped. Nothing was moved.".into());
    }
    let (root, path) = location(root, path, home, preferences)?;
    let stamp = Stamp::read(&path)?;
    Ok(FileReview { root, path, stamp, reviewed: Instant::now() })
}

pub fn review(
    root: &Path,
    path: &Path,
    preferences: &Preferences,
    control: &Control,
) -> Result<FileReview, String> {
    review_in(root, path, &dirs::home_dir().ok_or("Home folder is unavailable")?, preferences, control)
}

fn execute_with(
    review: &FileReview,
    home: &Path,
    preferences: &Preferences,
    control: &Control,
    recycle: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<Cleanup, String> {
    if control.cancelled() {
        return Err("Stopped. The file was not moved.".into());
    }
    if review.reviewed.elapsed() > REVIEW_LIFETIME {
        return Err("This file review expired. Review the file again.".into());
    }
    let (root, path) = location(&review.root, &review.path, home, preferences)?;
    if root != review.root || path != review.path || Stamp::read(&path)? != review.stamp {
        return Err("The file or its location changed after review. Analyze and review it again.".into());
    }
    if control.cancelled() {
        return Err("Stopped. The file was not moved.".into());
    }
    recycle(&path).map_err(|e| format!("Move not confirmed: {e}. Inspect Trash before retrying."))?;
    Ok(Cleanup {
        moved: 1,
        moved_bytes: review.bytes(),
        refused: 0,
        cancelled: false,
        log: vec![format!("Moved to Trash: {} ({} bytes)", path.display(), review.bytes())],
    })
}

pub fn move_to_trash(review: &FileReview, control: &Control) -> Result<Cleanup, String> {
    // Load again at the destructive boundary, not just when the review opened.
    // An unreadable preferences file is an error, not permission to ignore it.
    let preferences = Preferences::load()?;
    execute_with(
        review,
        &dirs::home_dir().ok_or("Home folder is unavailable")?,
        &preferences,
        control,
        |path| trash::delete(path).map_err(|e| e.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, PathBuf, FileReview) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let path = root.join("review.txt");
        fs::write(&path, "original").unwrap();
        let plan = review_in(&root, &path, &root, &Preferences::default(), &Control::default()).unwrap();
        (dir, root, plan)
    }
    #[test]
    fn review_does_not_touch_contents() {
        let (_dir, _root, plan) = fixture();
        assert_eq!(plan.bytes(), 8);
        assert_eq!(fs::read_to_string(plan.path()).unwrap(), "original");
    }
    #[test]
    fn changed_file_never_reaches_trash() {
        let (_dir, root, plan) = fixture();
        fs::write(plan.path(), "changed and longer").unwrap();
        assert!(execute_with(&plan, &root, &Preferences::default(), &Control::default(), |_| panic!("must not recycle")).is_err());
    }
    #[test]
    fn new_protection_is_checked_at_execution() {
        let (_dir, root, plan) = fixture();
        let prefs = Preferences { protected: vec![root.clone()], ..Default::default() };
        assert!(execute_with(&plan, &root, &prefs, &Control::default(), |_| panic!("must not recycle")).is_err());
    }
    #[test]
    fn cancellation_and_expiration_never_reach_trash() {
        let (_dir, root, mut plan) = fixture();
        let control = Control::default();
        control.stop();
        assert!(execute_with(&plan, &root, &Preferences::default(), &control, |_| panic!("must not recycle")).is_err());
        plan.reviewed = Instant::now() - Duration::from_secs(301);
        assert!(execute_with(&plan, &root, &Preferences::default(), &Control::default(), |_| panic!("must not recycle")).is_err());
    }
    #[test]
    fn failures_are_not_successes_and_have_no_delete_fallback() {
        let (_dir, root, plan) = fixture();
        let result = execute_with(&plan, &root, &Preferences::default(), &Control::default(), |_| Err("Recycle Bin unavailable".into()));
        assert!(result.unwrap_err().contains("not confirmed"));
        assert!(plan.path().exists());
    }
    #[test]
    fn confirmed_moves_count_only_the_reviewed_file() {
        let (_dir, root, plan) = fixture();
        let other = root.join("keep.txt");
        fs::write(&other, "keep").unwrap();
        let result = execute_with(&plan, &root, &Preferences::default(), &Control::default(), |path| fs::remove_file(path).map_err(|e| e.to_string())).unwrap();
        assert_eq!((result.moved, result.moved_bytes, result.refused), (1, 8, 0));
        assert!(other.exists());
        assert!(execute_with(&plan, &root, &Preferences::default(), &Control::default(), |_| panic!("must not recycle twice")).is_err());
    }
    #[test]
    fn rejects_folders_outside_scope_and_app_contents() {
        let (_dir, root, plan) = fixture();
        let sub = root.join("sub");
        fs::create_dir(&sub).unwrap();
        let prefs = Preferences::default();
        let control = Control::default();
        assert!(review_in(&root, &sub, &root, &prefs, &control).is_err());
        assert!(review_in(&sub, plan.path(), &root, &prefs, &control).is_err());
        assert!(review_in(&root, plan.path(), &sub, &prefs, &control).is_err());
        let app = root.join("Example.app");
        fs::create_dir(&app).unwrap();
        let inside = app.join("data");
        fs::write(&inside, "keep").unwrap();
        assert!(review_in(&root, &inside, &root, &prefs, &control).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn links_and_replacements_are_refused() {
        use std::os::unix::fs::symlink;
        let (_dir, root, plan) = fixture();
        let replacement = root.join("replacement");
        fs::write(&replacement, "original").unwrap();
        fs::remove_file(plan.path()).unwrap();
        symlink(&replacement, plan.path()).unwrap();
        assert!(execute_with(&plan, &root, &Preferences::default(), &Control::default(), |_| panic!("must not recycle")).is_err());
        fs::remove_file(plan.path()).unwrap();
        fs::hard_link(&replacement, plan.path()).unwrap();
        assert!(review_in(&root, plan.path(), &root, &Preferences::default(), &Control::default()).is_err());
    }
}
