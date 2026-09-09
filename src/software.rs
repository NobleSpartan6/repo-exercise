//! Application and startup inventories. OS uninstall strings are data, never code.
#[cfg(any(windows, target_os = "macos"))]
use crate::command;
use crate::{
    engine::{self, Control},
    human_bytes,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

pub const MAX_APPS: usize = 2000;
#[derive(Clone, Debug, Default)]
pub struct Application {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub origin: String,
    pub path: Option<PathBuf>,
    pub bundle_id: String,
    pub estimated_bytes: Option<u64>,
}
#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub apps: Vec<Application>,
    pub notes: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct StartupItem {
    pub name: String,
    pub location: String,
    pub command: String,
}

fn text(value: &serde_json::Value, key: &str) -> String {
    value[key]
        .as_str()
        .unwrap_or("")
        .chars()
        .filter(|c| !c.is_control())
        .take(4096)
        .collect()
}
fn json_rows(input: &str) -> Result<Vec<serde_json::Value>, String> {
    let value: serde_json::Value = serde_json::from_str(input.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("Could not read the OS inventory: {e}"))?;
    match value {
        serde_json::Value::Array(v) => Ok(v),
        serde_json::Value::Null => Ok(Vec::new()),
        v if v.is_object() => Ok(vec![v]),
        _ => Err("Unexpected OS inventory format".into()),
    }
}
pub fn parse_windows_apps(input: &str) -> Result<Inventory, String> {
    let mut rows = json_rows(input)?;
    let mut source_notes = Vec::new();
    if rows.len() == 1 && rows[0].get("apps").is_some() {
        let mut envelope = rows.pop().ok_or("Missing inventory")?;
        let apps = envelope.get_mut("apps").ok_or("Missing apps")?.take();
        rows = match apps {
            serde_json::Value::Array(rows) => rows,
            _ => return Err("Unexpected app inventory format".into()),
        };
        if let Some(notes) = envelope["notes"].as_array() {
            for note in notes.iter().take(12).filter_map(serde_json::Value::as_str) {
                source_notes.push(
                    note.chars()
                        .filter(|c| !c.is_control())
                        .take(4096)
                        .collect::<String>(),
                );
            }
        }
    }
    let partial = rows.len() > MAX_APPS;
    let mut apps: Vec<_> = rows
        .into_iter()
        .take(MAX_APPS)
        .filter_map(|v| {
            let name = text(&v, "name");
            if name.is_empty() {
                return None;
            }
            let p = text(&v, "path");
            Some(Application {
                name,
                version: text(&v, "version"),
                publisher: text(&v, "publisher"),
                origin: text(&v, "origin"),
                path: (!p.is_empty()).then(|| PathBuf::from(p)),
                estimated_bytes: v["bytes"].as_u64().filter(|n| *n > 0),
                bundle_id: String::new(),
            })
        })
        .collect();
    apps.sort_by_cached_key(|a| a.name.to_lowercase());
    let mut notes=vec!["Windows desktop and Store registrations. Portable apps may not appear. Sizes are publisher estimates, not measured disk use.".into()];
    notes.extend(source_notes);
    if partial {
        notes.push(format!("Showing the first {MAX_APPS} registrations."));
    }
    Ok(Inventory { apps, notes })
}

pub fn inventory(control: &Control) -> Result<Inventory, String> {
    #[cfg(windows)]
    {
        let script = r#"$items = [System.Collections.Generic.List[object]]::new();
        $notes = [System.Collections.Generic.List[string]]::new();
        foreach($r in @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall')) {
          try { if(Test-Path -LiteralPath $r) { foreach($k in Get-ChildItem -LiteralPath $r) {
            if($items.Count -ge 2001) { break }
            try {
              $v=Get-ItemProperty -LiteralPath $k.PSPath;
              if($v.DisplayName -and !$v.SystemComponent) {
                $items.Add([PSCustomObject]@{name=[string]$v.DisplayName;version=[string]$v.DisplayVersion;publisher=[string]$v.Publisher;path=[string]$v.InstallLocation;origin='Desktop';bytes=([long]$v.EstimatedSize*1024)});
              }
            } catch { if($notes.Count -lt 12){$notes.Add('Some desktop app registrations could not be read.')} }
          }}} catch { if($notes.Count -lt 12){$notes.Add('A desktop app source could not be read: '+$_.Exception.Message)} }
        }
        try { foreach($a in Get-AppxPackage -ErrorAction Stop) {
          if($items.Count -ge 2001) { break }
          if(!$a.IsFramework -and !$a.IsResourcePackage) {
            $items.Add([PSCustomObject]@{name=[string]$a.Name;version=[string]$a.Version;publisher=[string]$a.Publisher;path=[string]$a.InstallLocation;origin='Store';bytes=0});
          }
        }} catch { if($notes.Count -lt 12){$notes.Add('Store apps are unavailable; showing available desktop registrations. '+$_.Exception.Message)} }
        if($items.Count -ge 2001){$notes.Add('Inventory reached the 2,000-app display limit; other registrations may be absent.')}
        ConvertTo-Json -InputObject @{apps=@($items | Sort-Object name,version,path -Unique | Select-Object -First 2001);notes=@($notes | Select-Object -First 12)} -Depth 4 -Compress"#;
        parse_windows_apps(&command::powershell(script, control)?)
    }
    #[cfg(target_os = "macos")]
    {
        let mut result = Inventory::default();
        let mut roots = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
        ];
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join("Applications"));
        }
        for root in roots {
            if !root.exists() {
                continue;
            }
            let mut walker = walkdir::WalkDir::new(root)
                .max_depth(3)
                .follow_links(false)
                .into_iter();
            while let Some(entry) = walker.next() {
                if control.cancelled() {
                    return Err("Application scan cancelled.".into());
                }
                if result.apps.len() >= MAX_APPS {
                    result
                        .notes
                        .push("App inventory reached its size limit.".into());
                    break;
                }
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        if result.notes.len() < 12 {
                            result.notes.push(e.to_string());
                        }
                        continue;
                    }
                };
                if entry.path().extension().is_some_and(|e| e == "app") {
                    walker.skip_current_dir();
                    match read_bundle(entry.path()) {
                        Ok(app) => result.apps.push(app),
                        Err(e) => {
                            if result.notes.len() < 12 {
                                result
                                    .notes
                                    .push(format!("{}: {e}", entry.path().display()));
                            }
                        }
                    }
                }
            }
        }
        result.apps.sort_by_cached_key(|a| a.name.to_lowercase());
        result.notes.push("Applications folders only. Sizes are measured when you choose an app, not during discovery.".into());
        Ok(result)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = control;
        Err("App discovery is available on Mac and Windows.".into())
    }
}

fn read_plist(path: &Path) -> Result<plist::Value, String> {
    use std::io::Read;
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || engine::is_link_or_placeholder(&meta) || meta.len() > 512 * 1024 {
        return Err("Not a small local property list.".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(512 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 512 * 1024 {
        return Err("Property list exceeds the size limit.".into());
    }
    plist::Value::from_reader(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())
}
fn read_bundle(path: &Path) -> Result<Application, String> {
    let path = engine::checked_path(path)?;
    let metadata_path = engine::checked_path(&path.join("Contents/Info.plist"))?;
    let value = read_plist(&metadata_path)?;
    let d = value.as_dictionary().ok_or("Invalid bundle metadata")?;
    let field = |key: &str| {
        d.get(key)
            .and_then(plist::Value::as_string)
            .unwrap_or("")
            .chars()
            .filter(|c| !c.is_control())
            .take(4096)
            .collect::<String>()
    };
    let name = field("CFBundleDisplayName");
    let alt = field("CFBundleName");
    Ok(Application {
        name: if !name.is_empty() {
            name
        } else if !alt.is_empty() {
            alt
        } else {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        },
        version: field("CFBundleShortVersionString"),
        bundle_id: field("CFBundleIdentifier"),
        publisher: String::new(),
        origin: if path.starts_with("/System") {
            "System".into()
        } else {
            "Mac app".into()
        },
        path: Some(path),
        estimated_bytes: None,
    })
}

pub fn startup(control: &Control) -> Result<Vec<StartupItem>, String> {
    #[cfg(windows)]
    {
        let script = r#"$items=@();foreach($r in @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Run','HKLM:\Software\Microsoft\Windows\CurrentVersion\Run','HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run')) {
          if(Test-Path -LiteralPath $r) { $key=Get-Item -LiteralPath $r; foreach($n in $key.GetValueNames()) { $items += [PSCustomObject]@{name=$n;location=$r;command=[string]$key.GetValue($n)}; } }
        }
        foreach($dir in @([Environment]::GetFolderPath('Startup'),[Environment]::GetFolderPath('CommonStartup'))){if(Test-Path -LiteralPath $dir){foreach($f in Get-ChildItem -LiteralPath $dir -File){$items += [PSCustomObject]@{name=$f.Name;location=$dir;command=$f.FullName};}}}
        ConvertTo-Json -InputObject @($items|Select-Object -First 2000) -Compress"#;
        Ok(json_rows(&command::powershell(script, control)?)?
            .into_iter()
            .take(MAX_APPS)
            .map(|v| StartupItem {
                name: text(&v, "name"),
                location: text(&v, "location"),
                command: text(&v, "command"),
            })
            .collect())
    }
    #[cfg(target_os = "macos")]
    {
        let mut dirs = vec![
            PathBuf::from("/Library/LaunchAgents"),
            PathBuf::from("/Library/LaunchDaemons"),
        ];
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Library/LaunchAgents"));
        }
        let mut items = Vec::new();
        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
                if control.cancelled() {
                    return Err("Startup scan cancelled.".into());
                }
                let p = entry.map_err(|e| e.to_string())?.path();
                if p.extension().is_none_or(|e| e != "plist") {
                    continue;
                }
                if let Ok(v) = read_plist(&p)
                    && let Some(d) = v.as_dictionary()
                {
                    let label = d
                        .get("Label")
                        .and_then(plist::Value::as_string)
                        .unwrap_or("Unnamed job")
                        .to_owned();
                    let cmd = d
                        .get("Program")
                        .and_then(plist::Value::as_string)
                        .map(str::to_owned)
                        .unwrap_or_else(|| {
                            d.get("ProgramArguments")
                                .and_then(plist::Value::as_array)
                                .map(|a| {
                                    a.iter()
                                        .filter_map(plist::Value::as_string)
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                })
                                .unwrap_or_default()
                        });
                    items.push(StartupItem {
                        name: label,
                        location: p.display().to_string(),
                        command: cmd,
                    });
                }
                if items.len() >= MAX_APPS {
                    return Ok(items);
                }
            }
        }
        items.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(items)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = control;
        Err("Startup discovery is available on Mac and Windows.".into())
    }
}

pub fn check_updates(control: &Control) -> Result<String, String> {
    #[cfg(windows)]
    {
        let path = dirs::data_local_dir()
            .ok_or("No local application folder")?
            .join("Microsoft/WindowsApps/winget.exe");
        let report = command::run(
            &path,
            ["upgrade", "--disable-interactivity"],
            control,
            Duration::from_secs(60),
        )?;
        Ok(format!(
            "WinGet report — no updates installed. Sources may need first-time setup in App Installer.\n\n{report}"
        ))
    }
    #[cfg(target_os = "macos")]
    {
        let brew = ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"]
            .into_iter()
            .map(PathBuf::from)
            .find(|p| p.is_file())
            .ok_or(
                "Homebrew is not installed. Use App Store or each app's Check for Updates menu.",
            )?;
        let output = command::run(
            &brew,
            ["outdated", "--cask", "--json=v2"],
            control,
            Duration::from_secs(60),
        )?;
        let value: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| format!("Homebrew report could not be read: {e}"))?;
        let casks = value["casks"]
            .as_array()
            .ok_or("Homebrew returned an unexpected format")?;
        let lines: Vec<_> = casks
            .iter()
            .take(2000)
            .map(|c| format!("{}  →  {}", text(c, "name"), text(c, "current_version")))
            .collect();
        Ok(format!(
            "Homebrew cask update check — {} update entries. No updates installed.\nApp Store, Sparkle and apps installed outside Homebrew are not covered.\n\n{}",
            casks.len(),
            lines.join("\n")
        ))
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = control;
        Err("Update checks are available on Mac and Windows.".into())
    }
}

pub fn related_paths(app: &Application) -> Vec<PathBuf> {
    if !valid_bundle_id(&app.bundle_id) {
        return Vec::new();
    }
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    [
        home.join("Library/Caches").join(&app.bundle_id),
        home.join("Library/Application Support")
            .join(&app.bundle_id),
        home.join("Library/Preferences")
            .join(format!("{}.plist", app.bundle_id)),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}
fn valid_bundle_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() < 256
        && s.contains('.')
        && !s.contains("..")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".-_".contains(&c))
}

#[derive(Clone, Debug)]
pub struct RemovalPlan {
    pub name: String,
    pub path: PathBuf,
    pub bytes: u64,
    bundle_id: String,
    modified: SystemTime,
    created: Instant,
    manifest: Arc<Vec<BundleStamp>>,
    #[cfg(unix)]
    inode: u64,
}
// Revalidate the complete bundle metadata, not only its top directory. This
// catches updates that replace a file under Contents without touching the root.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BundleStamp {
    path: PathBuf,
    bytes: u64,
    modified: SystemTime,
    directory: bool,
    link: Option<PathBuf>,
    #[cfg(unix)]
    identity: (u64, u64),
}
fn bundle_manifest(path: &Path, control: &Control) -> Result<Vec<BundleStamp>, String> {
    let start = Instant::now();
    let mut entries = Vec::new();
    #[cfg(unix)]
    let device = {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(path).map_err(|e| e.to_string())?.dev()
    };
    for entry in walkdir::WalkDir::new(path)
        .follow_links(false)
        .same_file_system(true)
        .max_depth(129)
    {
        if control.cancelled() {
            return Err("App review cancelled. Nothing was moved.".into());
        }
        if entries.len() >= 100_000 || start.elapsed() > Duration::from_secs(120) {
            return Err("App review reached its limit. Use the vendor's uninstaller.".into());
        }
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.depth() > 128 {
            return Err("App bundle is too deeply nested to review.".into());
        }
        let meta = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if meta.dev() != device {
                return Err("App contains another mounted volume; removal was refused.".into());
            }
        }
        if !meta.is_file() && !meta.is_dir() && !meta.file_type().is_symlink() {
            return Err("App bundle contains an unsupported file type.".into());
        }
        entries.push(BundleStamp {
            path: entry
                .path()
                .strip_prefix(path)
                .map_err(|e| e.to_string())?
                .to_path_buf(),
            bytes: meta.len(),
            modified: meta.modified().map_err(|e| e.to_string())?,
            directory: meta.is_dir(),
            link: if meta.file_type().is_symlink() {
                Some(std::fs::read_link(entry.path()).map_err(|e| e.to_string())?)
            } else {
                None
            },
            #[cfg(unix)]
            identity: {
                use std::os::unix::fs::MetadataExt;
                (meta.dev(), meta.ino())
            },
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}
fn allowed_app(path: &Path, roots: &[PathBuf]) -> bool {
    path.extension().is_some_and(|e| e == "app")
        && roots.iter().any(|r| {
            path.parent() == Some(r.as_path())
                || path.parent() == Some(r.join("Utilities").as_path())
        })
}
fn app_roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from("/Applications")];
    if let Some(h) = dirs::home_dir() {
        roots.push(h.join("Applications"));
    }
    roots
}
fn not_running(path: &Path) -> Result<(), String> {
    if std::env::current_exe().is_ok_and(|p| p.starts_with(path)) {
        return Err("Burrow cannot remove itself while running.".into());
    }
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    if sys
        .processes()
        .values()
        .any(|p| p.exe().is_some_and(|exe| exe.starts_with(path)))
    {
        return Err("This app is running. Quit it, then review removal again.".into());
    }
    Ok(())
}
pub fn plan_removal(app: &Application, control: &Control) -> Result<RemovalPlan, String> {
    if !cfg!(target_os = "macos") {
        return Err("Use Windows Installed apps to run the vendor's uninstaller.".into());
    }
    let path = engine::checked_path(app.path.as_deref().ok_or("App path is unknown")?)?;
    if !allowed_app(&path, &app_roots())
        || app.bundle_id.starts_with("com.apple.")
        || !valid_bundle_id(&app.bundle_id)
    {
        return Err("This app is protected or its identity could not be verified. Use the system's app manager.".into());
    }
    not_running(&path)?;
    let fresh = read_bundle(&path)?;
    if fresh.bundle_id != app.bundle_id {
        return Err("The app changed. Refresh the list and try again.".into());
    }
    let analysis = engine::analyze(&path, control)?;
    if analysis.partial {
        return Err("App size scan was incomplete. No removal plan was created.".into());
    }
    let manifest = Arc::new(bundle_manifest(&path, control)?);
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    Ok(RemovalPlan {
        manifest,
        name: app.name.clone(),
        path,
        bytes: analysis.total_bytes,
        bundle_id: app.bundle_id.clone(),
        modified: meta.modified().map_err(|e| e.to_string())?,
        created: Instant::now(),
        #[cfg(unix)]
        inode: {
            use std::os::unix::fs::MetadataExt;
            meta.ino()
        },
    })
}
fn validate_plan(plan: &RemovalPlan, roots: &[PathBuf], control: &Control) -> Result<(), String> {
    if plan.created.elapsed() > Duration::from_secs(300) {
        return Err("This review expired. Review the app again.".into());
    }
    let path = engine::checked_path(&plan.path)?;
    if path != plan.path || !allowed_app(&path, roots) {
        return Err("App location changed or is protected.".into());
    }
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.modified().map_err(|e| e.to_string())? != plan.modified {
        return Err("The app changed since review.".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.ino() != plan.inode {
            return Err("The app was replaced since review.".into());
        }
    }
    if read_bundle(&path)?.bundle_id != plan.bundle_id {
        return Err("App identity changed since review.".into());
    }
    if bundle_manifest(&path, control)?.as_slice() != plan.manifest.as_slice() {
        return Err("Files inside the app changed since review. Review removal again.".into());
    }
    Ok(())
}
pub fn remove_app(plan: &RemovalPlan, control: &Control) -> Result<String, String> {
    if !cfg!(target_os = "macos") {
        return Err("Native bundle removal is Mac-only.".into());
    }
    if control.cancelled() {
        return Err("Cancelled. App was not moved.".into());
    }
    validate_plan(plan, &app_roots(), control)?;
    not_running(&plan.path)?;
    if control.cancelled() {
        return Err("Cancelled. App was not moved.".into());
    }
    trash::delete(&plan.path)
        .map_err(|e| format!("Move not confirmed: {e}. Inspect Trash before retrying."))?;
    Ok(format!(
        "Moved {} ({}) to Trash. Related app data was kept. Space is not freed until you empty Trash yourself.",
        plan.name,
        human_bytes(plan.bytes)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_values_never_become_commands() {
        let i=parse_windows_apps(r#"[{"name":"Something; Remove-Item C:\\*","path":"C:\\Apps","bytes":1024,"UninstallString":"cmd /c anything"}]"#).unwrap();
        assert_eq!(i.apps.len(), 1);
        assert_eq!(i.apps[0].estimated_bytes, Some(1024));
    }
    #[test]
    fn partial_inventory_keeps_available_apps_and_source_errors() {
        let i = parse_windows_apps(r#"{"apps":[{"name":"Local editor","origin":"Desktop"}],"notes":["Store source unavailable"]}"#).unwrap();
        assert_eq!(i.apps.len(), 1);
        assert!(
            i.notes
                .iter()
                .any(|n| n.contains("Store source unavailable"))
        );
        assert!(parse_windows_apps(r#"{"apps":false}"#).is_err());
        assert!(
            parse_windows_apps(r#"{"apps":[],"notes":["No readable sources"]}"#)
                .unwrap()
                .notes
                .len()
                > 1
        );
    }
    #[test]
    fn handles_empty_single_and_malformed_inventory() {
        assert!(parse_windows_apps("null").unwrap().apps.is_empty());
        assert_eq!(
            parse_windows_apps(r#"{"name":"App"}"#).unwrap().apps.len(),
            1
        );
        assert!(parse_windows_apps("not json").is_err());
    }
    #[test]
    fn bundle_ids_cannot_escape() {
        for s in ["../anything", "/tmp.app", "com.foo/evil", "com..foo", ""] {
            assert!(!valid_bundle_id(s));
        }
        assert!(valid_bundle_id("org.example.my-app"));
    }
    #[test]
    fn removal_does_not_accept_arbitrary_or_system_paths() {
        let roots = vec![PathBuf::from("/Applications")];
        assert!(!allowed_app(
            Path::new("/System/Applications/A.app"),
            &roots
        ));
        assert!(!allowed_app(
            Path::new("/Applications/A.app/child.app"),
            &roots
        ));
        assert!(!allowed_app(Path::new("/Applications"), &roots));
        assert!(allowed_app(Path::new("/Applications/A.app"), &roots));
    }
    #[test]
    fn refuses_large_and_symlink_metadata() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("large.plist");
        std::fs::File::create(&p)
            .unwrap()
            .set_len(513 * 1024)
            .unwrap();
        assert!(read_plist(&p).is_err());
    }
    fn reviewed_fixture() -> (tempfile::TempDir, RemovalPlan) {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = dir.path().join("Example.app");
        std::fs::create_dir_all(path.join("Contents")).unwrap();
        std::fs::write(path.join("Contents/Info.plist"), br#"<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.example.demo</string></dict></plist>"#).unwrap();
        std::fs::write(path.join("Contents/payload"), b"original").unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        let manifest = Arc::new(bundle_manifest(&path, &Control::default()).unwrap());
        let plan = RemovalPlan {
            name: "Example".into(),
            path,
            bytes: 8,
            bundle_id: "org.example.demo".into(),
            modified: meta.modified().unwrap(),
            created: Instant::now(),
            manifest,
            #[cfg(unix)]
            inode: {
                use std::os::unix::fs::MetadataExt;
                meta.ino()
            },
        };
        (dir, plan)
    }
    #[test]
    fn internal_updates_invalidate_removal_review() {
        let (d, p) = reviewed_fixture();
        let roots = vec![d.path().to_path_buf()];
        validate_plan(&p, &roots, &Control::default()).unwrap();
        std::fs::write(p.path.join("Contents/payload"), b"updated contents").unwrap();
        assert!(validate_plan(&p, &roots, &Control::default()).is_err());
        assert!(p.path.exists());
    }
    #[test]
    fn expired_and_cancelled_reviews_cannot_remove_apps() {
        let (d, mut p) = reviewed_fixture();
        let roots = vec![d.path().to_path_buf()];
        let c = Control::default();
        c.stop();
        assert!(validate_plan(&p, &roots, &c).is_err());
        p.created = Instant::now() - Duration::from_secs(301);
        assert!(
            validate_plan(&p, &roots, &Control::default())
                .unwrap_err()
                .contains("expired")
        );
        assert!(p.path.exists());
    }
    #[cfg(unix)]
    #[test]
    fn parent_swap_to_a_link_invalidates_app_review() {
        use std::os::unix::fs::symlink;
        let (d, p) = reviewed_fixture();
        let moved = d.path().join("Moved.app");
        std::fs::rename(&p.path, &moved).unwrap();
        symlink(&moved, &p.path).unwrap();
        assert!(validate_plan(&p, &[d.path().to_path_buf()], &Control::default()).is_err());
        assert!(moved.join("Contents/payload").exists());
    }
    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    #[ignore = "Read-only integration test for disposable native runners"]
    fn native_inventory_is_readable() {
        let i = inventory(&Control::default()).unwrap();
        assert!(!i.apps.is_empty());
        assert!(i.apps.len() <= MAX_APPS);
        startup(&Control::default()).unwrap();
    }
}
