//! Path-free cumulative counts of confirmed file moves, not claims of freed space.
use crate::engine::{self, Cleanup};
use serde_json::{Value, json};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
const MAX_BYTES: u64 = 4096;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub runs: u64,
    pub files: u64,
    pub bytes: u64,
}
impl Totals {
    fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() as u64 > MAX_BYTES {
            return Err("Cleanup totals file is too large.".into());
        }
        let value: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if value["version"] != 1 {
            return Err("Unsupported cleanup totals version. The file was left unchanged.".into());
        }
        let number = |key: &str| {
            value[key]
                .as_u64()
                .ok_or_else(|| format!("Invalid cleanup total: {key}"))
        };
        Ok(Self {
            runs: number("runs")?,
            files: number("files")?,
            bytes: number("bytes")?,
        })
    }
    fn add(self, result: &Cleanup) -> Result<Self, String> {
        if result.moved == 0 {
            return Ok(self);
        }
        let add = |a: u64, b: u64| {
            a.checked_add(b)
                .ok_or_else(|| "Cleanup total exceeded its limit.".to_owned())
        };
        Ok(Self {
            runs: add(self.runs, 1)?,
            files: add(self.files, result.moved as u64)?,
            bytes: add(self.bytes, result.moved_bytes)?,
        })
    }
}
fn path() -> Result<PathBuf, String> {
    dirs::data_local_dir()
        .map(|p| p.join("Burrow/cleanup-totals.json"))
        .ok_or_else(|| "No local folder for cleanup totals.".into())
}
fn load_at(path: &Path) -> Result<Totals, String> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Totals::default()),
        Err(e) => return Err(e.to_string()),
        Ok(meta) if !meta.is_file() || engine::is_link_or_placeholder(&meta) => {
            return Err("Cleanup totals must be a regular local file.".into());
        }
        Ok(_) => {}
    }
    engine::checked_path(path)?;
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Totals::decode(&bytes)
}
pub fn load() -> Result<Totals, String> {
    load_at(&path()?)
}
fn record_at(path: &Path, result: &Cleanup) -> Result<Totals, String> {
    if result.moved == 0 {
        return load_at(path);
    }
    let parent = path.parent().ok_or("No cleanup totals parent folder")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    engine::checked_path(parent)?;
    let lock_path = path.with_extension("lock");
    if lock_path.symlink_metadata().is_ok() {
        engine::checked_path(&lock_path)?;
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)
        .map_err(|e| e.to_string())?;
    // Separate lock file survives atomic replacement; another Burrow instance
    // must not silently overwrite our counts. Never block the UI or deletion.
    lock.try_lock()
        .map_err(|e| format!("Cleanup totals are busy; this run was not recorded: {e}"))?;
    let total = load_at(path)?.add(result)?;
    let bytes = serde_json::to_vec(
        &json!({"version": 1, "runs": total.runs, "files": total.files, "bytes": total.bytes}),
    )
    .map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(total)
}
/// Called only by the worker after actual confirmed moves. Errors do not turn a
/// successful Trash action into a failure or cause the action to be retried.
pub fn record(result: &Cleanup) -> Result<Totals, String> {
    record_at(&path()?, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn result() -> Cleanup {
        Cleanup {
            moved: 2,
            moved_bytes: 30,
            refused: 9,
            cancelled: true,
            log: vec!["PRIVATE/PATH".into()],
        }
    }
    #[test]
    fn only_confirmed_moves_accumulate_and_paths_are_not_stored() {
        let d = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = d.path().join("totals.json");
        assert_eq!(
            record_at(&path, &result()).unwrap(),
            Totals {
                runs: 1,
                files: 2,
                bytes: 30
            }
        );
        assert_eq!(
            record_at(&path, &result()).unwrap(),
            Totals {
                runs: 2,
                files: 4,
                bytes: 60
            }
        );
        assert_eq!(load_at(&path).unwrap().bytes, 60);
        assert!(!fs::read_to_string(path).unwrap().contains("PRIVATE"));
    }
    #[test]
    fn empty_or_failed_run_creates_no_totals() {
        let d = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = d.path().join("totals.json");
        let empty = Cleanup {
            moved: 0,
            moved_bytes: 0,
            refused: 1,
            cancelled: false,
            log: vec![],
        };
        assert_eq!(record_at(&path, &empty).unwrap(), Totals::default());
        assert!(!path.exists());
    }
    #[test]
    fn corrupt_newer_or_oversized_totals_are_preserved() {
        let d = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = d.path().join("totals.json");
        for text in [
            "invalid".to_owned(),
            r#"{"version":2}"#.into(),
            " ".repeat(4097),
        ] {
            fs::write(&path, &text).unwrap();
            assert!(record_at(&path, &result()).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), text);
        }
        assert!(Totals::decode(br#"{"version":1,"runs":-1,"files":2,"bytes":4}"#).is_err());
    }
    #[test]
    fn counts_do_not_wrap() {
        let totals = Totals {
            runs: u64::MAX,
            files: 0,
            bytes: 0,
        };
        assert!(totals.add(&result()).is_err());
    }
    #[test]
    fn concurrent_writer_is_refused_not_overwritten() {
        let d = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = d.path().join("totals.json");
        let lock = File::create(path.with_extension("lock")).unwrap();
        lock.try_lock().unwrap();
        assert!(record_at(&path, &result()).is_err());
        assert!(!path.exists());
        drop(lock);
        assert!(record_at(&path, &result()).is_ok());
    }
}
