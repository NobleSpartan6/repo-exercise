use super::*;
#[cfg(any(windows, target_os = "macos"))]
#[test]
#[ignore = "Touches only one disposable file; explicitly enabled in native CI"]
fn native_trash_roundtrip() {
    let home = dirs::home_dir().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("burrow-recycle-test-")
        .tempdir_in(&home)
        .unwrap();
    let root = CacheRoot {
        label: "Disposable QA fixture".into(),
        path: dir.path().canonicalize().unwrap(),
    };
    let name = format!(
        "burrow-qa-{}-{}.cache",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = root.path.join(&name);
    fs::write(&path, [0u8; 13]).unwrap();
    filetime::set_file_mtime(
        &path,
        filetime::FileTime::from_system_time(SystemTime::now() - Duration::from_secs(30 * DAY)),
    )
    .unwrap();
    let before = fs::read(&path).unwrap();
    let report = clean_with(
        &preview(std::slice::from_ref(&root), 7, &Control::default())
            .unwrap()
            .files,
        std::slice::from_ref(&root),
        &Control::default(),
        |p| trash::delete(p).map_err(|e| e.to_string()),
    );
    assert_eq!(report.moved, 1, "{:?}", report.log);
    assert!(!path.exists());
    #[cfg(windows)]
    {
        let item = trash::os_limited::list()
            .unwrap()
            .into_iter()
            .find(|item| item.name == std::ffi::OsStr::new(&name))
            .expect("File must actually be recoverable in Recycle Bin");
        assert_eq!(item.original_parent.canonicalize().unwrap(), root.path);
        trash::os_limited::restore_all([item]).unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        let trashed = home.join(".Trash").join(&name);
        assert_eq!(fs::read(&trashed).unwrap(), before);
        fs::rename(trashed, &path).unwrap();
    }
    assert_eq!(fs::read(&path).unwrap(), before);
}
