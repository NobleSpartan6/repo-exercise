//! Reviewed maintenance operations. No privilege escalation or arbitrary commands.
use crate::{
    command::{self, SettingsPage},
    engine::Control,
};
use std::time::Duration;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    QuickLookCache,
    QuickLookProviders,
    LookupCache,
    StorageSettings,
    SystemUpdates,
}
impl Action {
    pub const ALL: [Self; 5] = [
        Self::QuickLookCache,
        Self::QuickLookProviders,
        Self::LookupCache,
        Self::StorageSettings,
        Self::SystemUpdates,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::QuickLookCache => "Rebuild Quick Look thumbnails",
            Self::QuickLookProviders => "Reload Quick Look providers",
            Self::LookupCache => "Clear local name-lookup cache",
            Self::StorageSettings => "Open system storage tools",
            Self::SystemUpdates => "Open system updates",
        }
    }
    pub fn detail(self) -> &'static str {
        match self {
            Self::QuickLookCache => {
                "Mac only. Cached thumbnails will be regenerated as you browse files."
            }
            Self::QuickLookProviders => {
                "Mac only. Reloads thumbnail providers; does not restart your apps."
            }
            Self::LookupCache => {
                "Mac: directory-service cache. Windows: DNS cache. You may need a network connection for the next lookup. Some systems require permission."
            }
            Self::StorageSettings => {
                "Review the operating system's cleanup choices. Nothing is cleaned by opening it."
            }
            Self::SystemUpdates => {
                "Check for OS updates in System Settings. Burrow does not install them."
            }
        }
    }
    pub fn supported(self) -> bool {
        match self {
            Self::QuickLookCache | Self::QuickLookProviders => cfg!(target_os = "macos"),
            _ => cfg!(any(windows, target_os = "macos")),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Record {
    pub task: String,
    pub result: String,
    pub success: bool,
}
pub fn perform(actions: &[Action], control: &Control) -> Vec<Record> {
    let mut report = Vec::new();
    for action in actions {
        if control.cancelled() {
            report.push(Record {
                task: action.label().into(),
                result: "Skipped: you stopped the run.".into(),
                success: false,
            });
            continue;
        }
        if !action.supported() {
            report.push(Record {
                task: action.label().into(),
                result: "Not supported on this operating system.".into(),
                success: false,
            });
            continue;
        }
        let result = match action {
            Action::QuickLookCache => command::run(
                &command::system_tool("qlmanage"),
                ["-r", "cache"],
                control,
                Duration::from_secs(30),
            ),
            Action::QuickLookProviders => command::run(
                &command::system_tool("qlmanage"),
                ["-r"],
                control,
                Duration::from_secs(30),
            ),
            Action::LookupCache => {
                if cfg!(windows) {
                    command::run(
                        &command::system_tool("ipconfig.exe"),
                        ["/flushdns"],
                        control,
                        Duration::from_secs(30),
                    )
                } else {
                    command::run(
                        &command::system_tool("dscacheutil"),
                        ["-flushcache"],
                        control,
                        Duration::from_secs(30),
                    )
                }
            }
            Action::StorageSettings => command::open_settings(SettingsPage::Storage, control),
            Action::SystemUpdates => command::open_settings(SettingsPage::Updates, control),
        };
        report.push(match result {
            Ok(s) => Record {
                task: action.label().into(),
                result: if s.is_empty() {
                    "OS tool completed.".into()
                } else {
                    s
                },
                success: true,
            },
            Err(e) => Record {
                task: action.label().into(),
                result: e,
                success: false,
            },
        });
    }
    report
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stopped_runs_do_not_execute_tools() {
        let c = Control::default();
        c.stop();
        let r = perform(&Action::ALL, &c);
        assert_eq!(r.len(), 5);
        assert!(
            r.iter()
                .all(|r| !r.success && r.result.starts_with("Skipped"))
        );
    }
    #[test]
    fn unsupported_actions_explain_why() {
        if !cfg!(target_os = "macos") {
            let r = perform(&[Action::QuickLookCache], &Control::default());
            assert!(!r[0].success);
            assert!(r[0].result.contains("Not supported"));
        }
    }
}
