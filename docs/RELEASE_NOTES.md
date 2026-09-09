# Burrow 0.2.0 — modern native interface preview

A charcoal-and-mint redesign inspired by the restraint of Mole's desktop UI.
Still free, MIT-licensed, local-only, and native Rust/egui on Mac and Windows.

## What changed

- Replaced the large sidebar with compact segmented top navigation.
- Added a clear visual hierarchy, locally loaded system typography, quieter
  panels, rounded controls, static orbital accents and cleaner filename/path rows.
- Made compact and enlarged-text layouts scrollable; added explicit filtered-empty
  and unavailable-reading states. Confirmation remains separate and required.
- Kept the performance budget: virtualized lists, cached totals, metadata-only
  scans, bounded workers, independent system/drive sampling, no decorative
  animation loop, browser runtime, remote assets or bundled OS fonts.
- Updated links for the renamed `NobleSpartan6/burrow` repository.
- Added native GUI startup checks, headless click/selection/confirmation tests,
  Windows upgrade/install/uninstall checks, Mac packaged-app launch checks, and a
  native Trash/recovery test that touches only one uniquely named disposable file.
- Added a readable startup-error dialog instead of silent graphics-start failures.

The production cache allowlist and deletion engine are not broadened by this
redesign. Disk explorer remains read-only. Nothing is selected or removed
without your choice and confirmation. No health scores or fake performance
numbers; no promise to make your computer faster.

## Install or update

**Windows:** download `Burrow-0.2.0-Windows-x64-Setup.exe`, close the old app after
active work finishes, and run the installer using your normal user account.
Keep the installation location. No uninstall or administrator mode is needed.
Open About & help and check **0.2.0**.

**Mac:** choose the AppleSilicon DMG for an Apple M-series chip or the Intel DMG
for an Intel Mac. Quit the old app, open the DMG, drag Burrow into Applications,
and choose Replace. Source ZIPs are not installers. Portable Windows copies
are updated separately by extracting and opening the newer portable ZIP.

[Plain-language installation guide](https://github.com/NobleSpartan6/burrow/blob/main/docs/INSTALL.md).
Copy any cleanup session log before quitting; it is not saved automatically.

## Verification and limits

Publication is gated on the checks in the linked build run. The Windows upgrade
check runs the verified 0.1.1 installer and this release's installer in an isolated
CI folder, checks the installed executable, launches it, and uninstalls it. The
Mac check mounts the DMG, copies the app, verifies its version and ad-hoc signature,
and launches the copied executable. Neither simulates Gatekeeper or SmartScreen.
The disposable Trash test verifies one native move and recovery, not every volume,
OS setting, concurrent modification, or user recovery workflow.

This is preview software, not flawless or independently security-audited software.
Physical-device installations and unusual graphics, accessibility, network and
cloud configurations still need separate testing. Windows builds remain unsigned;
Mac builds are ad-hoc signed but not notarized. Do not disable security protections.

Keep backups. Moving files to Trash does not immediately free disk space. Burrow
never empties Trash. Inspect it after errors; cancellation does not undo completed
moves. Close apps whose caches you select, as regenerating caches may require
connectivity or temporarily slow apps down.

Native rendering in 0.2.0 uses Metal on Mac and DirectX 12 on Windows, with a low-power adapter preference. Linux remains a separate OpenGL QA target. No browser runtime is introduced. Native QA uses an explicit `--smoke-test` launch with `BURROW_SMOKE_OUTPUT` pointing to an empty evidence directory. It captures the app's GPU surface using egui screenshot events, not the unsupported eframe screenshot environment variable. Normal launches never capture or save screenshots.
