# Safety, privacy, and reporting

Burrow is preview software. Keep backups and start with the read-only Analyze
workspace. A successful build or recovery test does not prove that every machine,
filesystem, concurrent edit, or third-party app is safe.

## What can change

**Clean** moves only explicitly selected old regular files from known cache
folders to the OS Trash/Recycle Bin. It checks their identity, age, metadata, and
allowed root again before each move. Disabled groups and protected folders reduce
the preview; they never add cleanup locations.

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

**Analyze** and app/startup inventories are read-only. Windows app removal opens
Installed apps; Burrow never runs command strings obtained from the registry.
Update checks do not install updates and require explicit internet permission.

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
preferences are reported, not silently rewritten. Inventories and task logs remain
in memory unless you copy them. Update sources are contacted only for an opted-in
update check; opening a system tool hands control to that tool.

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
