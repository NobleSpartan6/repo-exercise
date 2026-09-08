# Burrow 0.1.1 — unsigned maintenance preview

This update keeps the white-and-green native interface and existing safety scope.
Burrow remains free and MIT-licensed, with no account, subscription, telemetry,
automatic updater, or background service after quitting.

## What changed

- **Clearer memory readings:** amount used, OS-reported total, and percentage used.
- **More readable drive usage:** full-width bars labeled as used space, with
  available capacity outside the bar. Text warnings appear at 10%/5% free.
  Invalid or missing capacities are shown as unavailable rather than full.
- **Better small-window behavior:** all of Overview scrolls, with a visible
  scrollbar; CPU/memory readings stack when there is insufficient width.
- **Independent monitoring:** slow volume queries no longer share a worker with
  CPU/RAM sampling. Each stream keeps only its latest sample; monitoring stops
  requesting repaints on other pages. Stale drive data is labeled.
- **Explorer correction:** selecting or scanning a different folder clears old
  results, rather than leaving the previous folder's results on screen.
- **Release reliability:** shared version metadata, required dependency lock,
  packaging regression tests, release-asset preflight and bounded retry for the
  observed transient macOS disk-image creation error.

The cache allowlist and deletion engine are unchanged. No new folders are
eligible for cleanup. There is still no automatic cleanup or empty-Trash action.
No faster-scan benchmark or lower memory-use figure is claimed for this update.

## Updating from 0.1.0

Copy any session log you need before quitting, wait for work to finish, then
close Burrow. On Windows, run the newer **Windows-x64-Setup.exe** under the same
user account and install over the existing copy. On Mac, replace the existing
app with the appropriate DMG's app. No terminal or uninstall is needed.
Check **About & help** for **0.1.1** after installation.

Choose **macOS-AppleSilicon.dmg** for an Apple M-series chip,
**macOS-Intel.dmg** for an Intel Mac, or **Windows-x64-Setup.exe** for a Windows
Intel/AMD 64-bit PC. **Source ZIPs are not installers.** The Windows portable
build is updated separately by extracting and opening the new portable copy.

Read the [installation and recovery guide](https://github.com/NobleSpartan6/repo-exercise/blob/main/docs/INSTALL.md).

## Verification and limitations

The user confirmed **0.1.0 installed and opened on Windows** and supplied an
Overview screenshot. That confirms only the reported old-version installation
and visible screen, not 0.1.1, Mac installation, full feature correctness,
performance, native Trash requests or restoration.

Publication is gated on all Mac/Windows build and test jobs, packaging regression
checks, and the Linux native-window launch/navigation/resize smoke test. The
release footer links its source commit and build run. These checks do not certify
physical-device installation, high-DPI accessibility, native Trash behavior or
restoration. Keep prerelease status until those separate checks are complete.

Packages remain unsigned on Windows and ad-hoc signed but **not notarized** on
Mac. Do not disable system protections to install them. Moving to Trash does
**not** immediately free disk space. Review selections, close relevant apps,
keep backups and inspect Trash after errors. Cancellation does not undo moves
already completed. The project has not undergone an independent security audit.
