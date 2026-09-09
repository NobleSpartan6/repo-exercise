//! Metadata-only discovery and an explicit, revalidated Trash transaction.
//! No production code in this module permanently deletes files.
use crate::platform::{self, CacheRoot};
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::fs::{self, Metadata};
use std::path::{Component, Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::time::{Duration, Instant, SystemTime};
use walkdir::WalkDir;

pub const MAX_CANDIDATES: usize = 20_000;
pub const MAX_ENTRIES: u64 = 500_000;
pub const TOP_FILES: usize = 200;
const MAX_DEPTH: usize = 128;
const TIME_BUDGET: Duration = Duration::from_secs(120);
const DAY: u64 = 86_400;

#[derive(Clone, Default)]
pub struct Control {
    pub cancel: Arc<AtomicBool>,
    pub visited: Arc<AtomicU64>,
}

impl Control {
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
    fn advance(&self, start: Instant) -> bool {
        !self.cancelled()
            && start.elapsed() < TIME_BUDGET
            && self.visited.fetch_add(1, Ordering::Relaxed) < MAX_ENTRIES
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Stamp {
    bytes: u64,
    modified: SystemTime,
    created: Option<SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

impl Stamp {
    fn read(meta: &Metadata) -> Result<Self, String> {
        if !meta.is_file() || is_link_or_placeholder(meta) {
            return Err("Not a regular local file".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if meta.nlink() != 1 {
                return Err("Hard-linked files are excluded".into());
            }
        }
        Ok(Self {
            bytes: meta.len(),
            modified: meta.modified().map_err(|e| e.to_string())?,
            created: meta.created().ok(),
            #[cfg(unix)]
            device: {
                use std::os::unix::fs::MetadataExt;
                meta.dev()
            },
            #[cfg(unix)]
            inode: {
                use std::os::unix::fs::MetadataExt;
                meta.ino()
            },
        })
    }
}

pub(crate) fn is_link_or_placeholder(meta: &Metadata) -> bool {
    if meta.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // REPARSE_POINT, OFFLINE, RECALL_ON_OPEN, RECALL_ON_DATA_ACCESS.
        if meta.file_attributes() & (0x400 | 0x1000 | 0x40000 | 0x400000) != 0 {
            return true;
        }
    }
    false
}

/// Check every existing component, not just the final file. On Windows this
/// rejects junctions/reparse points too. Path-based OS APIs still have a small
/// same-user TOCTOU window; see SECURITY.md, and never run this app elevated.
pub(crate) fn checked_path(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("An absolute path without '..' is required".into());
    }
    for part in path.ancestors() {
        let meta = fs::symlink_metadata(part).map_err(|e| e.to_string())?;
        if is_link_or_placeholder(&meta) {
            return Err("Links, junctions and cloud placeholders are excluded".into());
        }
    }
    path.canonicalize().map_err(|e| e.to_string())
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub path: PathBuf,
    pub root: PathBuf,
    pub category: String,
    pub bytes: u64,
    stamp: Stamp,
    min_age: Duration,
}

#[derive(Default, Debug)]
pub struct Preview {
    pub files: Vec<Candidate>,
    pub skipped: u64,
    pub warnings: Vec<String>,
    pub partial: bool,
    pub cancelled: bool,
    pub elapsed: Duration,
    pub visited: u64,
}

fn old_enough(stamp: &Stamp, age: Duration, now: SystemTime) -> bool {
    now.duration_since(stamp.modified)
        .map(|d| d >= age)
        .unwrap_or(false)
}

pub fn preview(roots: &[CacheRoot], days: u32, control: &Control) -> Result<Preview, String> {
    if !(7..=3650).contains(&days) {
        return Err("The minimum file age must be between 7 and 3650 days".into());
    }
    let start = Instant::now();
    let age = Duration::from_secs(u64::from(days) * DAY);
    let now = SystemTime::now();
    let count = AtomicUsize::new(0);
    let mut unique = Vec::new();
    let mut warnings = Vec::new();
    for root in roots {
        if !root.path.try_exists().unwrap_or(true) {
            continue;
        }
        match checked_path(&root.path) {
            Ok(path) if path.is_dir() => unique.push(CacheRoot {
                label: root.label.clone(),
                path,
            }),
            Ok(_) => warnings.push(format!("{}: not a folder", root.label)),
            Err(e) => warnings.push(format!("{}: {e}", root.label)),
        }
    }
    unique.sort_by_key(|r| r.path.components().count());
    let mut accepted: Vec<CacheRoot> = Vec::new();
    for root in unique {
        if !accepted.iter().any(|r| root.path.starts_with(&r.path)) {
            accepted.push(root);
        }
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .thread_name(|n| format!("burrow-scan-{n}"))
        .build()
        .map_err(|e| e.to_string())?;
    let parts: Vec<Preview> = pool.install(|| {
        accepted
            .par_iter()
            .map(|root| {
                let mut result = Preview::default();
                let mut walker = WalkDir::new(&root.path)
                    .follow_links(false)
                    .same_file_system(true)
                    .max_depth(MAX_DEPTH)
                    .into_iter();
                while let Some(entry) = walker.next() {
                    if !control.advance(start) || count.load(Ordering::Relaxed) >= MAX_CANDIDATES {
                        result.partial = true;
                        break;
                    }
                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => {
                            result.skipped += 1;
                            continue;
                        }
                    };
                    let meta = match fs::symlink_metadata(entry.path()) {
                        Ok(m) => m,
                        Err(_) => {
                            result.skipped += 1;
                            continue;
                        }
                    };
                    if is_link_or_placeholder(&meta) {
                        if entry.file_type().is_dir() {
                            walker.skip_current_dir();
                        }
                        result.skipped += 1;
                        continue;
                    }
                    if meta.is_dir() {
                        if entry.depth() == MAX_DEPTH {
                            result.partial = true;
                        }
                        continue;
                    }
                    let stamp = match Stamp::read(&meta) {
                        Ok(s) => s,
                        Err(_) => {
                            result.skipped += 1;
                            continue;
                        }
                    };
                    if stamp.bytes == 0 || !old_enough(&stamp, age, now) {
                        continue;
                    }
                    if count
                        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                            (n < MAX_CANDIDATES).then_some(n + 1)
                        })
                        .is_err()
                    {
                        result.partial = true;
                        break;
                    }
                    result.files.push(Candidate {
                        path: entry.path().to_path_buf(),
                        root: root.path.clone(),
                        category: root.label.clone(),
                        bytes: stamp.bytes,
                        stamp,
                        min_age: age,
                    });
                }
                result
            })
            .collect()
    });
    let mut result = Preview {
        warnings,
        ..Preview::default()
    };
    for part in parts {
        result.files.extend(part.files);
        result.skipped += part.skipped;
        result.partial |= part.partial;
    }
    result
        .files
        .sort_unstable_by(|a, b| b.bytes.cmp(&a.bytes).then(a.path.cmp(&b.path)));
    result.elapsed = start.elapsed();
    result.visited = control.visited.load(Ordering::Relaxed);
    result.cancelled = control.cancelled();
    result.partial |= result.cancelled;
    Ok(result)
}

fn validate(candidate: &Candidate, roots: &[PathBuf]) -> Result<(), String> {
    if !roots.contains(&candidate.root) {
        return Err("Cache is no longer allowlisted".into());
    }
    let root = checked_path(&candidate.root)?;
    let path = checked_path(&candidate.path)?;
    if path == root || !path.starts_with(&root) || path != candidate.path || root != candidate.root
    {
        return Err("File moved outside its reviewed cache".into());
    }
    let current = Stamp::read(&fs::symlink_metadata(&path).map_err(|e| e.to_string())?)?;
    if current != candidate.stamp || !old_enough(&current, candidate.min_age, SystemTime::now()) {
        return Err("File changed since preview; scan again".into());
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Cleanup {
    pub moved: usize,
    pub moved_bytes: u64,
    pub refused: usize,
    pub cancelled: bool,
    pub log: Vec<String>,
}

fn clean_with<F>(
    files: &[Candidate],
    roots: &[CacheRoot],
    control: &Control,
    mut recycle: F,
) -> Cleanup
where
    F: FnMut(&Path) -> Result<(), String>,
{
    let allowed: Vec<PathBuf> = roots
        .iter()
        .filter_map(|r| checked_path(&r.path).ok())
        .collect();
    let mut report = Cleanup::default();
    let mut seen = HashSet::new();
    for file in files {
        if control.cancelled() {
            report.cancelled = true;
            break;
        }
        if !seen.insert(&file.path) {
            continue;
        }
        control.visited.fetch_add(1, Ordering::Relaxed);
        match validate(file, &allowed).and_then(|()| recycle(&file.path)) {
            Ok(()) => {
                report.moved += 1;
                report.moved_bytes = report.moved_bytes.saturating_add(file.bytes);
                report.log.push(format!(
                    "MOVED TO TRASH\t{}\t{} bytes",
                    file.path.display(),
                    file.bytes
                ));
            }
            Err(error) => {
                report.refused += 1;
                report.log.push(format!(
                    "NOT CONFIRMED MOVED\t{}\t{error}",
                    file.path.display()
                ));
            }
        }
    }
    report
}

/// Only candidates from a preview can be selected. Re-discover the OS allowlist
/// immediately before acting; an analyzed folder is never a cleanup authority.
pub fn clean_selected(files: &[Candidate], control: &Control) -> Cleanup {
    clean_with(files, &platform::cache_roots(), control, |path| {
        trash::delete(path)
            .map_err(|e| format!("Trash operation failed: {e}. Inspect Trash before retrying."))
    })
}

#[derive(Clone, Debug)]
pub struct LargeFile {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Clone, Debug)]
pub struct FolderEntry {
    pub path: PathBuf,
    pub bytes: u64,
    pub files: u64,
    pub is_dir: bool,
}
pub const MAX_CHILDREN: usize = 2048;

#[derive(Debug, Default)]
pub struct Analysis {
    pub root: PathBuf,
    pub top: Vec<LargeFile>,
    pub children: Vec<FolderEntry>,
    pub other_bytes: u64,
    pub total_bytes: u64,
    pub files: u64,
    pub skipped: u64,
    pub visited: u64,
    pub partial: bool,
    pub cancelled: bool,
    pub elapsed: Duration,
}

/// Read-only, single pass. Retains at most TOP_FILES file records and MAX_CHILDREN child totals. Logical sizes are
/// not allocated bytes: sparse, compressed, cloned and hard-linked files differ.
pub fn analyze(folder: &Path, control: &Control) -> Result<Analysis, String> {
    let root = checked_path(folder)?;
    if !root.is_dir() {
        return Err("Choose a folder, not a file".into());
    }
    let start = Instant::now();
    let mut result = Analysis {
        root: root.clone(),
        ..Analysis::default()
    };
    let mut top = BinaryHeap::new();
    let mut children = std::collections::BTreeMap::<PathBuf, FolderEntry>::new();
    let mut walker = WalkDir::new(&root)
        .follow_links(false)
        .same_file_system(true)
        .max_depth(MAX_DEPTH)
        .into_iter();
    while let Some(entry) = walker.next() {
        if !control.advance(start) {
            result.partial = true;
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                result.skipped += 1;
                continue;
            }
        };
        let meta = match fs::symlink_metadata(entry.path()) {
            Ok(m) => m,
            Err(_) => {
                result.skipped += 1;
                continue;
            }
        };
        if is_link_or_placeholder(&meta) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            result.skipped += 1;
            continue;
        }
        if entry.depth() == 1 && meta.is_dir() && children.len() < MAX_CHILDREN {
            children
                .entry(entry.path().to_path_buf())
                .or_insert_with(|| FolderEntry {
                    path: entry.path().to_path_buf(),
                    bytes: 0,
                    files: 0,
                    is_dir: true,
                });
        }
        if meta.is_dir() {
            if entry.depth() == MAX_DEPTH {
                result.partial = true;
            }
            continue;
        }
        if !meta.is_file() {
            result.skipped += 1;
            continue;
        }
        result.files += 1;
        result.total_bytes = result.total_bytes.saturating_add(meta.len());
        if let Ok(relative) = entry.path().strip_prefix(&root)
            && let Some(first) = relative.components().next()
        {
            let key = root.join(first.as_os_str());
            if children.contains_key(&key) || children.len() < MAX_CHILDREN {
                let child = children.entry(key.clone()).or_insert_with(|| FolderEntry {
                    path: key,
                    bytes: 0,
                    files: 0,
                    is_dir: entry.depth() > 1,
                });
                child.bytes = child.bytes.saturating_add(meta.len());
                child.files = child.files.saturating_add(1);
            } else {
                result.other_bytes = result.other_bytes.saturating_add(meta.len());
            }
        }
        top.push(Reverse((meta.len(), entry.path().to_path_buf())));
        if top.len() > TOP_FILES {
            top.pop();
        }
    }
    result.children = children.into_values().collect();
    result
        .children
        .sort_by(|a, b| b.bytes.cmp(&a.bytes).then(a.path.cmp(&b.path)));
    result.top = top
        .into_iter()
        .map(|Reverse((bytes, path))| LargeFile { path, bytes })
        .collect();
    result
        .top
        .sort_unstable_by(|a, b| b.bytes.cmp(&a.bytes).then(a.path.cmp(&b.path)));
    result.elapsed = start.elapsed();
    result.visited = control.visited.load(Ordering::Relaxed);
    result.cancelled = control.cancelled();
    result.partial |= result.cancelled;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{FileTime, set_file_mtime};
    use tempfile::TempDir;

    fn fixture() -> (TempDir, CacheRoot) {
        let dir = tempfile::tempdir().unwrap();
        let root = CacheRoot {
            label: "Test-only cache".into(),
            path: dir.path().canonicalize().unwrap(),
        };
        (dir, root)
    }
    fn old_file(root: &CacheRoot, name: &str, bytes: u64) -> PathBuf {
        let path = root.path.join(name);
        let file = fs::File::create(&path).unwrap();
        file.set_len(bytes).unwrap();
        set_file_mtime(
            &path,
            FileTime::from_system_time(SystemTime::now() - Duration::from_secs(30 * DAY)),
        )
        .unwrap();
        path
    }
    fn plan(root: &CacheRoot) -> Vec<Candidate> {
        preview(std::slice::from_ref(root), 7, &Control::default())
            .unwrap()
            .files
    }
    #[test]
    fn excludes_recent_empty_and_future_files() {
        let (_dir, root) = fixture();
        old_file(&root, "old", 123);
        old_file(&root, "empty", 0);
        fs::write(root.path.join("new"), "important").unwrap();
        let future = old_file(&root, "future", 456);
        set_file_mtime(
            future,
            FileTime::from_system_time(SystemTime::now() + Duration::from_secs(DAY)),
        )
        .unwrap();
        let files = plan(&root);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].bytes, 123);
    }
    #[test]
    fn minimum_age_is_enforced() {
        let (_dir, root) = fixture();
        assert!(preview(&[root], 0, &Control::default()).is_err());
    }
    #[test]
    fn exact_age_boundary() {
        let (_dir, root) = fixture();
        old_file(&root, "file", 1);
        let files = plan(&root);
        let at_boundary = files[0].stamp.modified + Duration::from_secs(7 * DAY);
        assert!(old_enough(
            &files[0].stamp,
            Duration::from_secs(7 * DAY),
            at_boundary
        ));
        assert!(!old_enough(
            &files[0].stamp,
            Duration::from_secs(7 * DAY),
            at_boundary - Duration::from_secs(1)
        ));
    }
    #[test]
    fn changing_file_invalidates_preview() {
        let (_dir, root) = fixture();
        let path = old_file(&root, "old", 123);
        let files = plan(&root);
        fs::write(path, "changed").unwrap();
        let report = clean_with(&files, &[root], &Control::default(), |_| {
            panic!("must not recycle")
        });
        assert_eq!(report.refused, 1);
    }
    #[test]
    fn file_outside_allowlist_cannot_be_recycled() {
        let (_dir, root) = fixture();
        old_file(&root, "old", 1);
        let files = plan(&root);
        let report = clean_with(&files, &[], &Control::default(), |_| {
            panic!("must not recycle")
        });
        assert_eq!(report.refused, 1);
    }
    #[test]
    fn missing_file_is_reported_without_fallback() {
        let (_dir, root) = fixture();
        let path = old_file(&root, "old", 1);
        let files = plan(&root);
        fs::remove_file(path).unwrap();
        let report = clean_with(&files, &[root], &Control::default(), |_| {
            panic!("must not recycle")
        });
        assert_eq!(report.refused, 1);
    }
    #[test]
    fn native_trash_failure_never_calls_permanent_delete() {
        let (_dir, root) = fixture();
        let path = old_file(&root, "old", 13);
        let files = plan(&root);
        let report = clean_with(&files, &[root], &Control::default(), |_| {
            Err("Trash unavailable".into())
        });
        assert_eq!(report.refused, 1);
        assert_eq!(report.moved, 0);
        assert!(path.exists());
    }
    #[test]
    fn duplicate_selection_is_processed_once() {
        let (_dir, root) = fixture();
        old_file(&root, "old", 13);
        let mut files = plan(&root);
        files.push(files[0].clone());
        let mut calls = 0;
        let report = clean_with(&files, &[root], &Control::default(), |_| {
            calls += 1;
            Ok(())
        });
        assert_eq!(calls, 1);
        assert_eq!(report.moved_bytes, 13);
    }
    #[test]
    fn cancellation_is_not_success() {
        let (_dir, root) = fixture();
        old_file(&root, "old", 13);
        let files = plan(&root);
        let control = Control::default();
        control.stop();
        let report = clean_with(&files, std::slice::from_ref(&root), &control, |_| {
            panic!("must not recycle")
        });
        assert!(report.cancelled);
        assert_eq!(report.moved, 0);
        let scan = preview(&[root], 7, &control).unwrap();
        assert!(scan.partial && scan.cancelled);
        assert!(scan.files.is_empty());
    }
    #[test]
    fn overlapping_roots_do_not_duplicate_candidates() {
        let (_dir, root) = fixture();
        fs::create_dir(root.path.join("nested")).unwrap();
        old_file(&root, "nested/old", 12);
        let child = CacheRoot {
            label: "Child".into(),
            path: root.path.join("nested"),
        };
        assert_eq!(
            preview(&[root.clone(), child, root], 7, &Control::default())
                .unwrap()
                .files
                .len(),
            1
        );
    }
    #[test]
    fn analysis_is_sorted_bounded_and_read_only() {
        let (_dir, root) = fixture();
        for i in 1..=250 {
            old_file(&root, &format!("{i}.cache"), i);
        }
        let report = analyze(&root.path, &Control::default()).unwrap();
        assert_eq!(report.top.len(), TOP_FILES);
        assert_eq!(report.top[0].bytes, 250);
        assert_eq!(report.top[TOP_FILES - 1].bytes, 51);
        assert_eq!(report.files, 250);
        assert_eq!(report.total_bytes, 31_375);
        assert_eq!(fs::read_dir(root.path).unwrap().count(), 250);
    }
    #[test]
    fn parent_traversal_is_rejected() {
        let (_dir, root) = fixture();
        // PathBuf::join normalizes '..' away for Windows verbatim paths.
        // Construct the raw input without normalization so the guard, rather
        // than the path-builder's behavior, is what this regression tests.
        let mut raw = root.path.as_os_str().to_os_string();
        raw.push(std::path::MAIN_SEPARATOR_STR);
        raw.push("..");
        let path = Path::new(&raw);
        assert!(path.components().any(|c| matches!(c, Component::ParentDir)));
        assert!(checked_path(path).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_root_and_ancestor_are_rejected() {
        use std::os::unix::fs::symlink;
        let (_dir, root) = fixture();
        let (_outside_dir, outside) = fixture();
        old_file(&outside, "keep", 42);
        symlink(&outside.path, root.path.join("linked")).unwrap();
        assert!(plan(&root).is_empty());
        assert!(checked_path(&root.path.join("linked/keep")).is_err());
        let link = CacheRoot {
            label: "Link".into(),
            path: root.path.join("linked"),
        };
        assert!(plan(&link).is_empty());
    }
    #[cfg(unix)]
    #[test]
    fn ancestor_swap_after_preview_is_rejected() {
        use std::os::unix::fs::symlink;
        let (_dir, root) = fixture();
        let (_outside_dir, outside) = fixture();
        fs::create_dir(root.path.join("nested")).unwrap();
        old_file(&root, "nested/old", 42);
        old_file(&outside, "old", 42);
        let files = plan(&root);
        fs::rename(root.path.join("nested"), root.path.join("original")).unwrap();
        symlink(&outside.path, root.path.join("nested")).unwrap();
        let report = clean_with(&files, &[root], &Control::default(), |_| {
            panic!("must not recycle")
        });
        assert_eq!(report.refused, 1);
        assert!(outside.path.join("old").exists());
    }
    #[cfg(unix)]
    #[test]
    fn hard_links_are_not_cleanup_candidates() {
        let (_dir, root) = fixture();
        let first = old_file(&root, "one", 42);
        fs::hard_link(first, root.path.join("two")).unwrap();
        assert!(plan(&root).is_empty());
    }
    #[cfg(windows)]
    #[test]
    fn windows_junction_is_not_followed() {
        use std::process::Command;
        let (_dir, root) = fixture();
        let (_outside_dir, outside) = fixture();
        old_file(&outside, "keep", 42);
        let junction = root.path.join("junction");
        let output = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&outside.path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(plan(&root).is_empty());
        assert!(checked_path(&junction.join("keep")).is_err());
    }
}

#[cfg(all(test, any(windows, target_os = "macos")))]
#[path = "native_tests.rs"]
mod native_tests;

#[cfg(test)]
mod tree_tests {
    use super::*;
    #[test]
    fn child_totals_reconcile_with_parent() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join("a")).unwrap();
        std::fs::write(d.path().join("a/one"), [1; 11]).unwrap();
        std::fs::write(d.path().join("two"), [1; 7]).unwrap();
        let a = analyze(&d.path().canonicalize().unwrap(), &Control::default()).unwrap();
        assert_eq!(
            a.children.iter().map(|c| c.bytes).sum::<u64>() + a.other_bytes,
            18
        );
        assert_eq!(a.children.iter().filter(|c| c.is_dir).count(), 1);
        assert_eq!(a.files, 2);
    }
    #[test]
    fn child_count_is_bounded_and_overflow_is_counted() {
        let d = tempfile::tempdir().unwrap();
        for i in 0..MAX_CHILDREN + 3 {
            std::fs::write(d.path().join(format!("f{i}")), [1]).unwrap();
        }
        let a = analyze(&d.path().canonicalize().unwrap(), &Control::default()).unwrap();
        assert_eq!(a.children.len(), MAX_CHILDREN);
        assert_eq!(a.other_bytes, 3);
        assert_eq!(
            a.children.iter().map(|c| c.bytes).sum::<u64>() + a.other_bytes,
            a.total_bytes
        );
    }
}
