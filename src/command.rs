//! Bounded subprocesses for a small set of OS tools. No shell-built user input.
use crate::engine::Control;
use std::ffi::OsStr;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const OUTPUT_LIMIT: u64 = 8 * 1024 * 1024;

pub fn system_tool(name: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
        PathBuf::from(root).join("System32").join(name)
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/usr/bin").join(name)
    }
}

pub fn run<I, S>(
    program: &Path,
    args: I,
    control: &Control,
    timeout: Duration,
) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if control.cancelled() {
        return Err("Cancelled before starting.".into());
    }
    if !program.is_absolute() || !program.is_file() {
        return Err(format!(
            "{} is not available on this computer.",
            program.display()
        ));
    }
    // Anonymous, private files avoid pipe deadlocks and unbounded output buffers.
    let mut stdout = tempfile::tempfile().map_err(|e| e.to_string())?;
    let mut stderr = tempfile::tempfile().map_err(|e| e.to_string())?;
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(stdout.try_clone().map_err(|e| e.to_string())?)
        .stderr(stderr.try_clone().map_err(|e| e.to_string())?)
        .env("HOMEBREW_NO_AUTO_UPDATE", "1")
        .env("HOMEBREW_NO_ANALYTICS", "1");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let started = Instant::now();
    let status = loop {
        let too_large = [&stdout, &stderr]
            .iter()
            .any(|f| f.metadata().map_or(true, |m| m.len() > OUTPUT_LIMIT));
        let reason = if control.cancelled() {
            Some("Stopped. Changes already made by the OS tool are not undone.")
        } else if started.elapsed() > timeout {
            Some("The OS tool took too long and was stopped. Its result is not confirmed.")
        } else if too_large {
            Some("The OS tool returned too much output and was stopped.")
        } else {
            None
        };
        if let Some(reason) = reason {
            let _ = child.kill();
            let _ = child.wait();
            return Err(reason.into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
    };
    fn read(file: &mut std::fs::File) -> Result<String, String> {
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.take(OUTPUT_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > OUTPUT_LIMIT {
            return Err("Output exceeds the safety limit.".into());
        }
        Ok(String::from_utf8_lossy(&bytes)
            .trim_start_matches('\u{feff}')
            .trim()
            .to_owned())
    }
    let out = read(&mut stdout)?;
    let err = read(&mut stderr)?;
    if !status.success() {
        return Err(format!("The OS tool returned {}. {} {}", status, out, err));
    }
    Ok(if err.is_empty() {
        out
    } else {
        format!("{out}\n{err}")
    })
}

pub fn powershell(script: &str, control: &Control) -> Result<String, String> {
    let program = system_tool("WindowsPowerShell/v1.0/powershell.exe");
    let script = format!(
        "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); {script}"
    );
    run(
        &program,
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ],
        control,
        Duration::from_secs(30),
    )
}

#[derive(Clone, Copy, Debug)]
pub enum SettingsPage {
    Apps,
    Startup,
    Updates,
    Store,
    Storage,
    Activity,
    Power,
}

/// Only fixed OS URLs are accepted. This never evaluates registry uninstall strings.
pub fn open_settings(page: SettingsPage, control: &Control) -> Result<String, String> {
    #[cfg(windows)]
    {
        let url = match page {
            SettingsPage::Apps => "ms-settings:appsfeatures",
            SettingsPage::Startup => "ms-settings:startupapps",
            SettingsPage::Updates => "ms-settings:windowsupdate",
            SettingsPage::Store => "ms-windows-store:updates",
            SettingsPage::Storage => "ms-settings:storagesense",
            SettingsPage::Power => "ms-settings:powersleep",
            SettingsPage::Activity => {
                powershell(
                    "Start-Process -FilePath (Join-Path $env:SystemRoot 'System32\\Taskmgr.exe')",
                    control,
                )?;
                return Ok("Opened Task Manager. Complete any process action there.".into());
            }
        };
        powershell(&format!("Start-Process '{url}'"), control)?;
    }
    #[cfg(target_os = "macos")]
    {
        let args = match page {
            SettingsPage::Apps => vec!["/Applications"],
            SettingsPage::Startup => {
                vec!["x-apple.systempreferences:com.apple.LoginItems-Settings.extension"]
            }
            SettingsPage::Updates => {
                vec!["x-apple.systempreferences:com.apple.Software-Update-Settings.extension"]
            }
            SettingsPage::Store => vec!["macappstore://showUpdatesPage"],
            SettingsPage::Storage => vec!["x-apple.systempreferences:com.apple.settings.Storage"],
            SettingsPage::Activity => vec!["/System/Applications/Utilities/Activity Monitor.app"],
            SettingsPage::Power => vec!["x-apple.systempreferences:com.apple.preference.battery"],
        };
        run(&system_tool("open"), args, control, Duration::from_secs(10))?;
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (page, control);
        Err("This shortcut is available on Mac and Windows.".into())
    }
    #[cfg(any(windows, target_os = "macos"))]
    Ok(
        "Opened the system tool. Complete the action there; Burrow has not changed anything."
            .into(),
    )
}

/// Reveal a local path in the file manager; never execute the selected file.
pub fn reveal(path: &Path, control: &Control) -> Result<String, String> {
    let path = crate::engine::checked_path(path)?;
    #[cfg(target_os = "macos")]
    {
        run(
            &system_tool("open"),
            [OsStr::new("-R"), path.as_os_str()],
            control,
            Duration::from_secs(10),
        )?;
    }
    #[cfg(windows)]
    {
        // Pass a folder as one OS argument; never build a shell command.
        let mut cmd = Command::new(system_tool("explorer.exe"));
        let folder = if path.is_dir() {
            &path
        } else {
            path.parent().ok_or("No containing folder")?
        };
        cmd.arg(folder)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = cmd.spawn().map_err(|e| e.to_string())?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        let _ = control;
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (path, control);
        Err("Use Copy path with your file manager on this OS.".into())
    }
    #[cfg(any(windows, target_os = "macos"))]
    Ok("Opened the containing folder. No file was executed or removed.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_relative_executables() {
        assert!(
            run(
                Path::new("echo"),
                ["test"],
                &Control::default(),
                Duration::from_secs(1)
            )
            .is_err()
        );
    }
    #[test]
    fn cancellation_never_starts_a_command() {
        let c = Control::default();
        c.stop();
        assert!(
            run(Path::new("/missing"), [""], &c, Duration::from_secs(1))
                .unwrap_err()
                .contains("Cancelled")
        );
    }
    #[cfg(unix)]
    #[test]
    fn timeout_is_not_success() {
        assert!(
            run(
                Path::new("/bin/sleep"),
                ["5"],
                &Control::default(),
                Duration::from_millis(70)
            )
            .unwrap_err()
            .contains("too long")
        );
    }
    #[cfg(unix)]
    #[test]
    fn captures_output_and_nonzero_status() {
        assert_eq!(
            run(
                Path::new("/bin/echo"),
                ["hello"],
                &Control::default(),
                Duration::from_secs(1)
            )
            .unwrap(),
            "hello"
        );
        assert!(
            run(
                Path::new("/usr/bin/false"),
                [] as [&str; 0],
                &Control::default(),
                Duration::from_secs(1)
            )
            .is_err()
        );
    }
}
