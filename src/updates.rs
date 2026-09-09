//! Opt-in, selected-package updates. Human-readable command output is never an install plan.
use crate::{command, engine::Control};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const MAX_ENTRIES: usize = 2000;
const MAX_SELECTED: usize = 20;
const REVIEW_LIFETIME: Duration = Duration::from_secs(300);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Homebrew,
    WinGet,
}
impl Provider {
    pub fn label(self) -> &'static str {
        match self {
            Self::Homebrew => "Homebrew casks",
            Self::WinGet => "WinGet · current user only",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    id: String,
    name: String,
    installed: String,
    available: String,
}
impl Update {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn installed(&self) -> &str {
        &self.installed
    }
    pub fn available(&self) -> &str {
        &self.available
    }
}

#[derive(Clone, Debug)]
pub struct Catalog {
    provider: Provider,
    entries: Vec<Update>,
    checked: Instant,
}
impl Catalog {
    pub fn provider(&self) -> Provider {
        self.provider
    }
    pub fn entries(&self) -> &[Update] {
        &self.entries
    }
    /// Indices refer to this exact catalog. Empty, stale and out-of-range selections fail closed.
    pub fn review(&self, selected: &BTreeSet<usize>) -> Result<Plan, String> {
        if self.checked.elapsed() > REVIEW_LIFETIME {
            return Err("The update check expired. Check for updates again.".into());
        }
        if selected.is_empty() || selected.len() > MAX_SELECTED {
            return Err(format!(
                "Select between 1 and {MAX_SELECTED} updates to review."
            ));
        }
        let entries = selected
            .iter()
            .map(|index| {
                self.entries
                    .get(*index)
                    .cloned()
                    .ok_or_else(|| "The selection no longer matches this update check.".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Plan {
            provider: self.provider,
            entries,
            checked: self.checked,
        })
    }
}

/// No public fields or deserialization: only a successful structured catalog can create a plan.
#[derive(Clone, Debug)]
pub struct Plan {
    provider: Provider,
    entries: Vec<Update>,
    checked: Instant,
}
impl Plan {
    pub fn provider(&self) -> Provider {
        self.provider
    }
    pub fn entries(&self) -> &[Update] {
        &self.entries
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    ProviderCompleted,
    Refused,
    Failed,
    NotStarted,
}
impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::ProviderCompleted => "Provider completed",
            Self::Refused => "Not run",
            Self::Failed => "Result not confirmed",
            Self::NotStarted => "Not started",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Outcome {
    pub name: String,
    pub id: String,
    pub state: State,
    pub detail: String,
}

fn text(value: &Value, key: &str) -> Result<String, String> {
    let value = value[key]
        .as_str()
        .ok_or_else(|| format!("Missing update field: {key}"))?;
    if value.trim().is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        return Err(format!("Invalid update field: {key}"));
    }
    Ok(value.to_owned())
}
fn valid_id(provider: Provider, id: &str) -> bool {
    if id.len() > 256 {
        return false;
    }
    let parts = id.split('/').collect::<Vec<_>>();
    if parts.len() != 1 && !(provider == Provider::Homebrew && parts.len() == 3) {
        return false;
    }
    parts.iter().all(|part| {
        part.as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
            && part
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_.+@".contains(&c))
            && !part.contains("..")
    })
}
fn parse(provider: Provider, output: &str) -> Result<Catalog, String> {
    if output.len() > 2 * 1024 * 1024 {
        return Err("The update catalog exceeded its size limit.".into());
    }
    let value: Value = serde_json::from_str(output)
        .map_err(|e| format!("The provider did not return a structured update catalog: {e}"))?;
    let rows = match provider {
        Provider::Homebrew => value["casks"].as_array(),
        Provider::WinGet => value["packages"].as_array(),
    }
    .ok_or("Unexpected update catalog format. Use the provider report or the app's own updater.")?;
    if rows.len() > MAX_ENTRIES {
        return Err("The update catalog exceeded its entry limit.".into());
    }
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        let entry = match provider {
            Provider::Homebrew => {
                let id = text(row, "name")?;
                let versions = row["installed_versions"]
                    .as_array()
                    .ok_or("Missing installed cask versions")?;
                // Updating an ambiguously installed cask is not a useful first-run default.
                if versions.len() != 1 {
                    return Err(format!(
                        "{id} has multiple or unknown installed versions. Use Homebrew to review it."
                    ));
                }
                let installed = versions[0]
                    .as_str()
                    .ok_or("Invalid installed cask version")?
                    .to_owned();
                Update {
                    name: id.clone(),
                    id,
                    installed,
                    available: text(row, "current_version")?,
                }
            }
            Provider::WinGet => {
                if text(row, "source")? != "winget" {
                    return Err(
                        "Only packages from the configured winget source are installable here."
                            .into(),
                    );
                }
                Update {
                    id: text(row, "id")?,
                    name: text(row, "name")?,
                    installed: text(row, "installed")?,
                    available: text(row, "available")?,
                }
            }
        };
        if !valid_id(provider, &entry.id)
            || [&entry.installed, &entry.available].iter().any(|v| {
                v.is_empty()
                    || v.len() > 256
                    || v.starts_with('-')
                    || v.chars().any(char::is_control)
                    || v.eq_ignore_ascii_case("unknown")
                    || v.eq_ignore_ascii_case("latest")
            })
            || entry.installed == entry.available
        {
            return Err(
                "A package identifier or version is ambiguous. No install controls were created."
                    .into(),
            );
        }
        if !seen.insert(entry.id.to_lowercase()) {
            return Err("The provider returned duplicate package identifiers. No install controls were created.".into());
        }
        entries.push(entry);
    }
    entries.sort_by_cached_key(|e| (e.name.to_lowercase(), e.id.to_lowercase()));
    Ok(Catalog {
        provider,
        entries,
        checked: Instant::now(),
    })
}
fn program(provider: Provider) -> Result<PathBuf, String> {
    match provider {
        Provider::Homebrew => ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"]
            .into_iter()
            .map(PathBuf::from)
            .find(|p| p.is_file())
            .ok_or_else(|| "Homebrew is not installed. Use the app's updater or App Store.".into()),
        Provider::WinGet => dirs::data_local_dir()
            .map(|p| p.join("Microsoft/WindowsApps/winget.exe"))
            .filter(|p| p.is_file())
            .ok_or_else(|| {
                "WinGet is not available. Use the app's updater or Microsoft Store.".into()
            }),
    }
}

// Fixed script, never interpolated with app names, paths, identifiers or versions.
const WINGET_CATALOG: &str = r#"
$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue';
[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false);
Import-Module Microsoft.WinGet.Client -ErrorAction Stop;
$rows=@(Microsoft.WinGet.Client\Get-WinGetPackage -Source winget -ErrorAction Stop |
    Where-Object { $_.IsUpdateAvailable } | ForEach-Object {
        [PSCustomObject]@{ id=$_.Id; name=$_.Name; source=$_.Source;
            installed=$_.InstalledVersion; available=$_.AvailableVersions[0] }
    });
[PSCustomObject]@{packages=$rows} | ConvertTo-Json -Depth 4 -Compress
"#;
fn check_provider(provider: Provider, control: &Control) -> Result<Catalog, String> {
    let manager = program(provider)?;
    let output = match provider {
        Provider::Homebrew => command::run(
            &manager,
            ["outdated", "--cask", "--json=v2"],
            control,
            Duration::from_secs(60),
        )?,
        Provider::WinGet => {
            // The object-based package API needs PowerShell 7, not Windows PowerShell 5.1.
            // Never bootstrap a shell or module just because the user checked for updates.
            let pwsh = std::env::var_os("ProgramFiles").map(PathBuf::from).map(|p| p.join("PowerShell/7/pwsh.exe"))
                .filter(|p| p.is_file())
                .or_else(|| dirs::data_local_dir().map(|p| p.join("Microsoft/PowerShell/7/pwsh.exe")).filter(|p| p.is_file()))
                .ok_or("Reviewed WinGet installs need an existing PowerShell 7 and Microsoft.WinGet.Client module. Nothing was installed. You can still read the provider report or open Microsoft Store.")?;
            command::run(
                &pwsh,
                [
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-MTA",
                    "-Command",
                    WINGET_CATALOG,
                ],
                control,
                Duration::from_secs(60),
            )?
        }
    };
    parse(provider, &output)
}
pub fn check(control: &Control) -> Result<Catalog, String> {
    let provider = if cfg!(target_os = "macos") {
        Provider::Homebrew
    } else if cfg!(windows) {
        Provider::WinGet
    } else {
        return Err("Reviewed package updates are available on Mac and Windows.".into());
    };
    check_provider(provider, control)
}
fn install_args(provider: Provider, entry: &Update) -> Vec<String> {
    let args = match provider {
        Provider::Homebrew => vec![
            "upgrade",
            "--cask",
            "--skip-cask-deps",
            "--no-quit",
            "--no-ask",
            "--require-sha",
            "--",
            &entry.id,
        ],
        Provider::WinGet => vec![
            "upgrade",
            "--id",
            &entry.id,
            "--exact",
            "--version",
            &entry.available,
            "--source",
            "winget",
            "--scope",
            "user",
            "--silent",
            "--disable-interactivity",
        ],
    };
    args.into_iter().map(str::to_owned).collect()
}
fn execute_with(
    plan: &Plan,
    control: &Control,
    mut check: impl FnMut(Provider, &Control) -> Result<Catalog, String>,
    mut install: impl FnMut(Provider, &Update) -> Result<String, String>,
) -> Vec<Outcome> {
    let mut stopped = false;
    let mut outcomes = Vec::new();
    for entry in &plan.entries {
        let (state, detail) = if stopped || control.cancelled() {
            (
                State::NotStarted,
                "Queue stopped. Previously completed operations were not undone.".into(),
            )
        } else if plan.checked.elapsed() > REVIEW_LIFETIME {
            (
                State::Refused,
                "The five-minute review expired. Check and review again.".into(),
            )
        } else {
            match check(plan.provider, control) {
                Ok(fresh) if fresh.provider == plan.provider && fresh.entries.iter().any(|e| e == entry) => {
                    if control.cancelled() {
                        (State::NotStarted, "Stopped before starting this installer.".into())
                    } else if plan.checked.elapsed() > REVIEW_LIFETIME {
                        (State::Refused, "The review expired during revalidation. Check and review again.".into())
                    } else {
                        match install(plan.provider, entry) {
                            Ok(output) => (State::ProviderCompleted, output),
                            Err(error) => (State::Failed, error),
                        }
                    }
                }
                Ok(_) => (State::Refused, "The package or its installed/available version changed. Nothing was started for this entry.".into()),
                Err(error) => (State::Refused, format!("Could not revalidate this package: {error}")),
            }
        };
        if state != State::ProviderCompleted {
            stopped = true;
        }
        let detail = if detail.chars().count() > 16_384 {
            format!(
                "{}\n[Report shortened; inspect the package manager's own log.]",
                detail.chars().take(16_384).collect::<String>()
            )
        } else {
            detail
        };
        outcomes.push(Outcome {
            name: entry.name.clone(),
            id: entry.id.clone(),
            state,
            detail,
        });
    }
    outcomes
}
/// Stop requests prevent the next package from starting; they never kill an in-flight
/// installer. The UI keeps its root window open while this worker owns the queue.
/// A timeout/output-limit failure is ambiguous; no automatic retry or rollback follows.
pub fn install(plan: &Plan, control: &Control) -> Vec<Outcome> {
    execute_with(plan, control, check_provider, |provider, entry| {
        let program = program(provider)?;
        command::run(
            &program,
            install_args(provider, entry),
            &Control::default(),
            Duration::from_secs(15 * 60),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn catalog() -> Catalog {
        parse(Provider::WinGet, r#"{"packages":[{"id":"Example.App","name":"Example App","source":"winget","installed":"1.0","available":"2.0"},{"id":"Other.App","name":"Other App","source":"winget","installed":"3.0","available":"4.0"}]}"#).unwrap()
    }
    #[test]
    fn both_structured_providers_have_explicit_versions() {
        let brew = parse(Provider::Homebrew, r#"{"casks":[{"name":"homebrew/cask/example","installed_versions":["1.0"],"current_version":"2.0"}]}"#).unwrap();
        assert_eq!(brew.entries()[0].installed(), "1.0");
        assert_eq!(catalog().entries()[0].id(), "Example.App");
        assert!(
            parse(
                Provider::WinGet,
                "Name Id Version Available\nExample App 1.0 2.0"
            )
            .is_err()
        );
    }
    #[test]
    fn malformed_or_ambiguous_catalogs_fail_closed() {
        for bad in [
            "--all",
            "../../x",
            "x;Remove-Item",
            "x\ny",
            "a b",
            "",
            "$(x)",
        ] {
            assert!(!valid_id(Provider::WinGet, bad));
            assert!(!valid_id(Provider::Homebrew, bad));
        }
        let report = r#"{"packages":[{"id":"Example.App","name":"Example","source":"winget","installed":"1","available":"2"}]}"#;
        for (old, new) in [
            ("winget", "msstore"),
            ("Example.App", "--all"),
            ("\"2\"", "\"unknown\""),
        ] {
            assert!(parse(Provider::WinGet, &report.replace(old, new)).is_err());
        }
        let row = r#"{"id":"Example.App","name":"Example","source":"winget","installed":"1","available":"2"}"#;
        assert!(parse(Provider::WinGet, &format!("{{\"packages\":[{row},{row}]}}")).is_err());
        assert!(
            parse(
                Provider::Homebrew,
                r#"{"casks":[{"name":"example","installed_versions":[],"current_version":"2"}]}"#
            )
            .is_err()
        );
    }
    #[test]
    fn empty_stale_and_invalid_selections_cannot_become_upgrade_all() {
        let mut c = catalog();
        assert!(c.review(&BTreeSet::new()).is_err());
        assert!(c.review(&BTreeSet::from([8])).is_err());
        assert!(c.review(&BTreeSet::from([0])).is_ok());
        c.checked = Instant::now() - Duration::from_secs(301);
        assert!(c.review(&BTreeSet::from([0])).is_err());
    }
    #[test]
    fn commands_name_one_package_and_do_not_override_security_or_agreements() {
        let c = catalog();
        for provider in [Provider::Homebrew, Provider::WinGet] {
            let args = install_args(provider, &c.entries()[0]);
            assert_eq!(
                args.iter().filter(|s| s.as_str() == "Example.App").count(),
                1
            );
            for forbidden in [
                "--all",
                "--force",
                "--accept-source-agreements",
                "--accept-package-agreements",
                "--ignore-security-hash",
                "--allow-reboot",
                "--override",
                "--zap",
            ] {
                assert!(!args.iter().any(|s| s == forbidden));
            }
        }
        let args = install_args(Provider::WinGet, &c.entries()[0]);
        assert!(args.windows(2).any(|w| w == ["--scope", "user"]));
    }
    #[test]
    fn changed_package_does_not_reach_installer_and_stops_queue() {
        let c = catalog();
        let plan = c.review(&BTreeSet::from([0, 1])).unwrap();
        let mut fresh = c;
        fresh.entries[0].available = "9.0".into();
        let result = execute_with(
            &plan,
            &Control::default(),
            |_, _| Ok(fresh.clone()),
            |_, _| panic!("must not install"),
        );
        assert_eq!(result[0].state, State::Refused);
        assert_eq!(result[1].state, State::NotStarted);
    }
    #[test]
    fn cancel_after_first_install_leaves_second_unstarted() {
        let c = catalog();
        let plan = c.review(&BTreeSet::from([0, 1])).unwrap();
        let control = Control::default();
        let result = execute_with(
            &plan,
            &control,
            |_, _| Ok(c.clone()),
            |_, _| {
                control.stop();
                Ok("provider completed".into())
            },
        );
        assert_eq!(result[0].state, State::ProviderCompleted);
        assert_eq!(result[1].state, State::NotStarted);
    }
    #[test]
    fn cancellation_expiration_and_provider_errors_never_claim_success() {
        let c = catalog();
        let mut plan = c.review(&BTreeSet::from([0])).unwrap();
        let control = Control::default();
        control.stop();
        assert_eq!(
            execute_with(
                &plan,
                &control,
                |_, _| panic!("must not query"),
                |_, _| panic!("must not install")
            )[0]
            .state,
            State::NotStarted
        );
        plan.checked = Instant::now() - Duration::from_secs(301);
        assert_eq!(
            execute_with(
                &plan,
                &Control::default(),
                |_, _| panic!("must not query"),
                |_, _| panic!("must not install")
            )[0]
            .state,
            State::Refused
        );
        plan.checked = Instant::now();
        assert_eq!(
            execute_with(
                &plan,
                &Control::default(),
                |_, _| Ok(c.clone()),
                |_, _| Err("installer failed".into())
            )[0]
            .state,
            State::Failed
        );
    }
    #[test]
    fn cancel_during_revalidation_is_checked_before_start() {
        let c = catalog();
        let plan = c.review(&BTreeSet::from([0])).unwrap();
        let control = Control::default();
        let result = execute_with(
            &plan,
            &control,
            |_, token| {
                token.stop();
                Ok(c.clone())
            },
            |_, _| panic!("must not install"),
        );
        assert_eq!(result[0].state, State::NotStarted);
    }
}
