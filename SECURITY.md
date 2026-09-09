# Safety, privacy, and reporting

Burrow is preview software. Keep backups and start with a read-only Analyze
scan. A successful build or recovery test does not prove that every machine,
filesystem, concurrent edit, or third-party app is safe.

## What can change

**Clean** moves only explicitly selected old regular files from known cache
folders to the OS Trash/Recycle Bin. It checks their identity, age, metadata, and
allowed root again before each move. Saved preferences are reloaded at the start
of cleanup and combined with the restrictions already shown in the current session.
Disabled groups and protected folders only reduce eligibility; they never add
cleanup locations. Changes made concurrently during a run are not a locked transaction.

**Mac app removal** has a separate review. Only eligible app bundles directly in
Applications or its Utilities folder can be considered. Apple bundle IDs, unknown
identities, changed bundles, expired reviews, running apps, and Burrow itself are
refused. Metadata for the whole bundle is compared again before moving it. Related
settings, shared services, and support files are not removed. Apps with services,
drivers, or extensions should use their vendor's uninstaller.

**Optimize** runs only a reviewed list of fixed OS tasks. Tool failures remain
failures. It never uses elevated privileges, force-purges memory, repairs registry
entries, or changes thermal limits. The screen-on request is opt-in, timed, and
released on stop or quit; normal OS sleep policies still apply.

**Analyze scans** and app/startup inventories are read-only. A separate single-file
Trash action requires review of the full path and current size, explicit confirmation,
and revalidation. It is restricted to regular local files inside the user's home
and the analyzed root. Folders, app-bundle contents, protected paths, existing Trash,
and Burrow's own settings/executable are excluded. Reviews expire after five minutes.
Windows app removal opens Installed apps; registry command strings are never executed.

**Package updates** require internet consent, a structured provider catalog, explicit
selection, and a separate acknowledged review. No empty selection can become an
all-packages command. Each entry is revalidated; stale versions, duplicate identifiers,
unknown formats, and failed providers are refused. Only named Homebrew casks or
current-user upgrades from the configured WinGet source are attempted. Existing
PowerShell 7 and Microsoft.WinGet.Client are required for structured Windows discovery;
no prerequisite is installed implicitly. Localized text reports never create plans.

Updates execute the installed package manager and vendor installers, not a Trash
operation. They can modify dependencies and app data and have no Burrow rollback.
Stop/Close prevents the next package from starting but does not kill the current
installer; the window stays open until the worker returns. Time/output limits can
still interrupt the manager and leave an uncertain result or vendor child process.
Inspect the manager and app before retrying; there are no automatic retries.
Agreement acceptance, security-hash bypass, reboot, and arbitrary installer overrides
are not supplied. Homebrew cleanup and dependent auto-upgrades are disabled.
Do not run concurrent package-manager operations; metadata rechecks are not locks
on another program's database or a guarantee about vendor installer behavior.

## Recovery and limits

Burrow never empties Trash and never falls back to permanent deletion. After an
error, inspect both Trash and the original location before retrying. Cancellation
stops remaining work where possible; it does not undo completed changes.

Filesystem validation rejects unsafe ancestors, links, Windows reparse points,
and cloud placeholders for cleanup. App bundles may contain internal symlinks;
review records those links without following their targets. The OS Trash API is
path-based, so a small same-user time-of-check/time-of-use race still exists.
Metadata checks cannot detect an adversary who perfectly forges file metadata.
Do not run Burrow as an administrator or against files another program is changing.

OS tools run with fixed executable paths and separately passed arguments. Output
and waiting time are bounded. Cancellation does not promise to roll back a tool's
changes or immediately interrupt a blocked OS/filesystem call. Third-party update
sources and vendor uninstallers have their own trust and permission boundaries.

## Local data and network use

There are no accounts, ads, or telemetry. Burrow reads metadata rather than user
file contents for storage scans. It reads small app manifests and startup
registrations for the Apps workspace. It samples process names, IDs, CPU, and
memory; it does not read process environments or command lines for Status.

Cleanup settings are saved in a small local `Burrow/preferences.json` file under
the OS local application-data folder. It contains cache-group choices and
protected paths. Writes use a temporary file and atomic replacement. Invalid
preferences are reported, not silently rewritten. Path-free totals are stored in
`Burrow/cleanup-totals.json` with atomic replacement and a separate writer lock.
They count only confirmed file moves; corruption or a competing writer is reported
without repeating a successful cleanup. They do not measure reclaimed space or
record app-bundle removal. Inventories and task logs remain in memory unless copied.
Update checks and reviewed installs contact configured sources only after consent;
package managers may keep their own downloads/logs. Opening a system tool hands
control to that tool.

Normal launches never take screenshots. Explicit QA modes can capture Burrow's
own GPU surface or emit control positions. CI uses temporary fixtures and
hosted-runner data, not a user's computer.

## Downloads and reports

Windows previews are unsigned; Mac previews are ad-hoc signed but not notarized.
Do not disable OS security protections. Release checksums detect changed downloads
but are not verified publisher identity or an independent security audit.

For a reproducible non-sensitive bug, use the repository issue form. Redact
personal paths, usernames, app lists, and private data. Do not put credentials or
an exploitable private-data exposure into a public issue. Use GitHub's private
vulnerability reporting when it is available; otherwise contact the repository
owner privately before sharing details.
