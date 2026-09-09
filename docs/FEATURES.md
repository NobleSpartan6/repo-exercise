# Feature coverage

**0.3 development** continues from the [published 0.2.0 preview](https://github.com/NobleSpartan6/burrow/releases/tag/v0.2.0-preview-2e38c4c).
It is not a complete port of [Mole](https://mole.fit/). “Built in” below describes
source on this branch, not a claim that an unreleased installer has passed QA.

| Capability | Mac | Windows |
| --- | --- | --- |
| Cache scan, explicit selection, Trash results | Built in | Built in, Recycle Bin |
| Saved cache choices and protected folders | Built in | Built in |
| Saved cleanup totals | Confirmed file-move counts only; no filenames or freed-space claim | Same |
| Folder map, drill-down, breadcrumbs, largest files | Built in | Built in |
| Analyze file removal | Separate reviewed, expiring single-file action inside home; no folders or app contents | Same; links/reparse/cloud placeholders excluded |
| App inventory, search, size inspection | Application bundles; sizes on demand | Desktop/Store registrations; publisher size estimates |
| App removal | Reviewed app bundle to Trash; related data kept | Opens OS/vendor uninstaller |
| Related app files | Exact bundle-ID paths can be inspected | No automatic leftover sweep |
| Update discovery | Opt-in Homebrew cask check | Opt-in WinGet report |
| Selected package updates | Reviewed named casks via existing Homebrew | Reviewed current-user upgrades via existing WinGet, PowerShell 7 and Microsoft.WinGet.Client |
| Startup inventory/changes | Read registrations; OS settings for changes | Read registrations; OS settings for changes |
| Maintenance | Reviewed Quick Look/local lookup tasks and OS shortcuts | Reviewed local lookup task and OS shortcuts |
| Keep screen on | Opt-in timed native request | Opt-in timed native request |
| CPU, RAM, swap, disks, network, uptime | OS readings | OS readings |
| Battery charge/power and temperatures | Only when the provider exposes them | Only when the provider exposes them |
| Process search, sort, pin | Built in; OS monitor for actions | Same |
| Mini monitor | Separate floating window | Separate floating window |

## Update boundaries

No automatic selection or `upgrade all`. Plans expire five minutes after the
check. Each selected entry is revalidated before execution; failures stop the
remaining queue. Stop does not kill the current installer. Empty, duplicate,
malformed, unsupported, or stale catalogs never become install commands.

The installer reports are provider results, not independent software audits.
Windows installs are user-scope only; missing PowerShell/module prerequisites leave
read-only reporting available. Homebrew metadata can change, and its selected cask
version is ultimately chosen at execution. Homebrew automatic cleanup, dependent
upgrades, and app-quitting are disabled; vendor installers still have their own
side effects. App Store/Sparkle aggregation and OS-update installation are absent.

## Still missing

True tray/menu-bar integration; GPU utilization and fan RPM/control; battery health
or charge limits; accessory batteries; camera/microphone activity; app icons and
last-used dates; in-app startup toggles; related-data/bulk uninstall; folder removal
in Analyze; process termination; and a complete cleanup history with recovery.
Cleanup totals are not a retroactive history. Linux remains a source/QA target.

## Deliberate differences

Burrow does not automatically choose files, empty Trash, permanently delete after
a Trash failure, force-purge RAM, repair registry entries, change thermal limits,
or silently elevate itself. Those actions are not prerequisites for useful storage
management. Unsupported readings and OS handoffs are labeled rather than presented
as working native features. [Safety and privacy](../SECURITY.md)
