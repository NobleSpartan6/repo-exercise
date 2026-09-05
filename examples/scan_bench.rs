//! A synthetic metadata-only benchmark, not a real-world performance promise.
use burrow::engine::{self, Control};
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    for group in 0..100 {
        let folder = directory.path().join(group.to_string());
        fs::create_dir(&folder)?;
        for file in 0..100 {
            fs::File::create(folder.join(file.to_string()))?.set_len(1024)?;
        }
    }
    let root = directory.path().canonicalize()?;
    let started = Instant::now();
    let result = engine::analyze(&root, &Control::default()).map_err(std::io::Error::other)?;
    let elapsed = started.elapsed();
    assert_eq!(result.files, 10_000);
    assert_eq!(result.top.len(), engine::TOP_FILES);
    assert!(!result.partial);
    println!("Synthetic scan: 10,000 one-KiB files / 100 folders in {:.3}s ({:.0} files/s). Retained {} largest file records. File creation excluded; warm filesystem metadata; not a Mac/Windows benchmark.",
        elapsed.as_secs_f64(), result.files as f64 / elapsed.as_secs_f64(), result.top.len());
    Ok(())
}
