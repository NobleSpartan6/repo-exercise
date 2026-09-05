use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct CacheRoot {
    pub label: String,
    pub path: PathBuf,
}

/// Deliberately narrow allowlist. No browser profiles, passwords, Downloads,
/// Documents, OS directories, package installations, or arbitrary temp trees.
pub fn cache_roots() -> Vec<CacheRoot> {
    let mut roots = Vec::new();
    let Some(home) = dirs::home_dir() else { return roots };
    if cfg!(target_os = "macos") {
        for (label, relative) in [
            ("Homebrew downloads", "Library/Caches/Homebrew/downloads"),
            ("Python pip cache", "Library/Caches/pip"),
            ("Python uv cache", "Library/Caches/uv"),
            ("Go build cache", "Library/Caches/go-build"),
            ("Chrome cache", "Library/Caches/Google/Chrome"),
            ("Edge cache", "Library/Caches/Microsoft Edge"),
            ("npm download cache", ".npm/_cacache"),
        ] {
            roots.push(CacheRoot { label: label.into(), path: home.join(relative) });
        }
    } else if cfg!(target_os = "windows") {
        // dirs uses the platform's local application data location, not a path
        // received from the UI. Only known cache subdirectories are eligible.
        if let Some(local) = dirs::data_local_dir() {
            for (label, relative) in [
                ("Python pip cache", "pip/Cache"),
                ("Python uv cache", "uv/cache"),
                ("Go build cache", "go-build"),
                ("npm download cache", "npm-cache/_cacache"),
                ("Chrome cache (Default profile)", "Google/Chrome/User Data/Default/Cache"),
                ("Edge cache (Default profile)", "Microsoft/Edge/User Data/Default/Cache"),
            ] {
                roots.push(CacheRoot { label: label.into(), path: local.join(relative) });
            }
        }
    }
    roots
}
