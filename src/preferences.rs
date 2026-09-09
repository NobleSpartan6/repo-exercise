//! Small, versioned local preferences. No inventories, file contents or telemetry.
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
const MAX_BYTES: u64 = 64 * 1024;
#[derive(Clone, Debug, Default)]
pub struct Preferences {
    pub disabled_caches: HashSet<String>,
    pub protected: Vec<PathBuf>,
}
impl Preferences {
    pub fn path() -> Option<PathBuf> {
        dirs::data_local_dir().map(|p| p.join("Burrow").join("preferences.json"))
    }
    pub fn excludes(&self, path: &Path) -> bool {
        self.protected.iter().any(|p| path.starts_with(p))
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() as u64 > MAX_BYTES {
            return Err("Preferences file is too large.".into());
        }
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|e| format!("Could not read preferences: {e}"))?;
        if value["version"] != 1 {
            return Err("Unsupported preferences version. Your file was left unchanged.".into());
        }
        let strings = |key: &str| -> Result<Vec<String>, String> {
            let values = value[key]
                .as_array()
                .ok_or_else(|| format!("Invalid {key} preferences"))?;
            if values.len() > 256 {
                return Err("Too many preferences entries.".into());
            }
            values
                .iter()
                .map(|v| {
                    v.as_str()
                        .filter(|s| s.len() <= 4096)
                        .map(str::to_owned)
                        .ok_or_else(|| "Invalid preferences entry".into())
                })
                .collect()
        };
        let disabled_caches = strings("disabled_caches")?.into_iter().collect();
        let protected: Vec<_> = strings("protected")?
            .into_iter()
            .map(PathBuf::from)
            .collect();
        if protected.iter().any(|p| {
            !p.is_absolute()
                || p.components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
        }) {
            return Err("Protected folders must use absolute paths.".into());
        }
        Ok(Self {
            disabled_caches,
            protected,
        })
    }
    pub fn load() -> Result<Self, String> {
        let Some(path) = Self::path() else {
            return Ok(Self::default());
        };
        match std::fs::File::open(path) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(MAX_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                Self::decode(&bytes)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn save(&self) -> Result<(), String> {
        self.save_at(&Self::path().ok_or("No local preferences folder is available")?)
    }
    fn save_at(&self, path: &Path) -> Result<(), String> {
        let parent = path.parent().ok_or("No preferences parent")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut disabled: Vec<_> = self.disabled_caches.iter().collect();
        disabled.sort();
        let protected: Result<Vec<_>, _> = self
            .protected
            .iter()
            .map(|p| p.to_str().ok_or("This path cannot be saved as text"))
            .collect();
        let bytes = serde_json::to_vec_pretty(
            &json!({"version":1,"disabled_caches":disabled,"protected":protected?}),
        )
        .map_err(|e| e.to_string())?;
        Self::decode(&bytes)?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_or_newer_preferences_are_rejected() {
        for s in ["bad", "{}", r#"{"version":2}"#] {
            assert!(Preferences::decode(s.as_bytes()).is_err());
        }
    }
    #[test]
    fn protections_match_components_not_string_prefixes() {
        let root = std::env::temp_dir().join("keep");
        let p = Preferences {
            protected: vec![root.clone()],
            ..Default::default()
        };
        assert!(p.excludes(&root.join("inside")));
        assert!(!p.excludes(&root.with_file_name("keeper")));
    }
    #[test]
    fn saves_and_replaces_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("prefs.json");
        let mut p = Preferences::default();
        p.save_at(&file).unwrap();
        p.disabled_caches.insert("pip".into());
        p.save_at(&file).unwrap();
        assert!(
            Preferences::decode(&std::fs::read(file).unwrap())
                .unwrap()
                .disabled_caches
                .contains("pip")
        );
    }
    #[test]
    fn rejects_relative_protections() {
        assert!(
            Preferences::decode(
                br#"{"version":1,"disabled_caches":[],"protected":["../somewhere"]}"#
            )
            .is_err()
        );
    }
}
