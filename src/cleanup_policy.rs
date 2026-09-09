//! The app's cleanup boundary adds saved/user-reviewed restrictions to the core allowlist.
use crate::{
    engine::{self, Candidate, Cleanup, Control},
    platform::{self, CacheRoot},
    preferences::Preferences,
};
use std::collections::HashSet;

fn execute_with(
    files: &[Candidate],
    roots: &[CacheRoot],
    reviewed: &Preferences,
    current: &Preferences,
    control: &Control,
    run: impl FnOnce(&[Candidate], &Control) -> Cleanup,
) -> Cleanup {
    // Union, never replacement: an external deletion/reset of the saved file must
    // not silently forget protections still displayed in this Burrow instance.
    let protected = reviewed
        .protected
        .iter()
        .chain(&current.protected)
        .flat_map(|path| [Some(path.clone()), path.canonicalize().ok()])
        .flatten()
        .collect::<Vec<_>>();
    let disabled_roots = roots
        .iter()
        .filter(|root| {
            reviewed.disabled_caches.contains(&root.label)
                || current.disabled_caches.contains(&root.label)
        })
        .filter_map(|root| root.path.canonicalize().ok())
        .collect::<Vec<_>>();
    let mut eligible = Vec::new();
    let mut refused = Vec::new();
    let mut seen = HashSet::new();
    for file in files {
        if !seen.insert(&file.path) {
            continue;
        }
        if protected.iter().any(|p| file.path.starts_with(p)) || disabled_roots.contains(&file.root)
        {
            refused.push(format!(
                "NOT MOVED\t{}\tBlocked by current or reviewed cleanup preferences",
                file.path.display()
            ));
        } else {
            eligible.push(file.clone());
        }
    }
    let mut result = if eligible.is_empty() || control.cancelled() {
        Cleanup {
            cancelled: control.cancelled(),
            ..Default::default()
        }
    } else {
        run(&eligible, control)
    };
    result.refused += refused.len();
    result.log.extend(refused);
    result
}

/// Reload once immediately before this run, not just at startup/scan. Concurrent
/// changes during the run remain a filesystem race, not a transaction or lock.
pub fn clean_selected(
    files: &[Candidate],
    reviewed: &Preferences,
    control: &Control,
) -> Result<Cleanup, String> {
    let current = Preferences::load()
        .map_err(|e| format!("Cleanup paused: saved protections could not be read. {e}"))?;
    Ok(execute_with(
        files,
        &platform::cache_roots(),
        reviewed,
        &current,
        control,
        engine::clean_selected,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, CacheRoot, Vec<Candidate>) {
        let dir = tempfile::tempdir().unwrap();
        let root = CacheRoot {
            label: "Fixture".into(),
            path: dir.path().canonicalize().unwrap(),
        };
        let file = root.path.join("old.cache");
        std::fs::write(&file, "keep").unwrap();
        filetime::set_file_mtime(
            &file,
            filetime::FileTime::from_system_time(
                std::time::SystemTime::now() - std::time::Duration::from_secs(30 * 86400),
            ),
        )
        .unwrap();
        let files = engine::preview(std::slice::from_ref(&root), 7, &Control::default())
            .unwrap()
            .files;
        (dir, root, files)
    }
    #[test]
    fn protections_added_after_scan_and_previous_protections_both_apply() {
        let (_dir, root, files) = fixture();
        let protected = Preferences {
            protected: vec![root.path.clone()],
            ..Default::default()
        };
        for (old, new) in [
            (&protected, &Preferences::default()),
            (&Preferences::default(), &protected),
        ] {
            let result = execute_with(
                &files,
                std::slice::from_ref(&root),
                old,
                new,
                &Control::default(),
                |_, _| panic!("must not clean"),
            );
            assert_eq!((result.moved, result.refused), (0, 1));
            assert!(files[0].path.exists());
        }
    }
    #[test]
    fn disabled_roots_are_checked_by_root_identity_not_mutable_display_labels() {
        let (_dir, root, mut files) = fixture();
        let mut prefs = Preferences::default();
        prefs.disabled_caches.insert(root.label.clone());
        files[0].category = "Another label".into();
        let result = execute_with(
            &files,
            &[root],
            &Preferences::default(),
            &prefs,
            &Control::default(),
            |_, _| panic!("must not clean"),
        );
        assert_eq!(result.refused, 1);
    }
    #[test]
    fn unprotected_selection_reaches_only_the_existing_core_cleaner_once() {
        let (_dir, root, mut files) = fixture();
        files.push(files[0].clone());
        let result = execute_with(
            &files,
            &[root],
            &Preferences::default(),
            &Preferences::default(),
            &Control::default(),
            |eligible, _| {
                assert_eq!(eligible.len(), 1);
                Cleanup {
                    moved: 1,
                    moved_bytes: eligible[0].bytes,
                    ..Default::default()
                }
            },
        );
        assert_eq!((result.moved, result.refused), (1, 0));
        assert!(files[0].path.exists()); // mock runner, no real Trash operation
    }
}
