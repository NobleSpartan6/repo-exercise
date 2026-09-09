# Burrow 0.3.0 preview

Five workspaces for reviewing storage and seeing what your computer is doing.
Free, MIT-licensed, and built with native Rust/egui for Mac and Windows.

## New in this preview

- **Clean:** saved cache-group choices and protected folders. The cache allowlist
  is not expanded; nothing is selected or removed automatically.
- **Apps:** installed-app search, size inspection, startup registrations, and
  opt-in WinGet/Homebrew update checks. Mac bundle removal has a separate review;
  Windows opens Installed apps for the vendor's uninstaller.
- **Optimize:** reviewed maintenance tasks, per-task results, system-tool shortcuts,
  and a timed screen-on session that ends when you stop it or quit.
- **Analyze:** a read-only folder map with drill-down, breadcrumbs, direct-child
  totals, and the largest files.
- **Status:** process search/sort/pinning, network rates, swap, uptime, battery,
  available temperatures, and a floating mini monitor.
- Shorter README, clearer first steps, and an explicit feature-coverage table.

[Install or update](https://github.com/NobleSpartan6/burrow/blob/main/docs/INSTALL.md)
· [First steps](https://github.com/NobleSpartan6/burrow/blob/main/docs/TRY_BURROW.md)
· [Feature coverage](https://github.com/NobleSpartan6/burrow/blob/main/docs/FEATURES.md)

## Install

On Windows, quit the old app and run `Burrow-0.3.0-Windows-x64-Setup.exe` under your
normal account, keeping the installation folder. On Mac, choose the DMG for your
chip, quit Burrow, and drag the new copy into Applications. Choose Replace.
Source ZIPs are for developers. Check **? → About & help** for version 0.3.0.

## Know before trying it

This is not full Mole feature parity. There is no built-in bulk app updater,
related-data deletion, GPU usage/fan control, battery-health management, or
menu-bar/system-tray integration. The feature table explains other limits.

Windows builds are unsigned. Mac builds are ad-hoc signed but not notarized. Do
not disable security protections. Automated hosted-runner checks are not a
physical-device test, independent security audit, or guarantee of zero bugs.

Keep backups and try Analyze first. Moving files to Trash does not immediately
free space. Burrow never empties Trash or permanently deletes when a move fails.
Inspect Trash after errors; stopping a task does not undo completed changes.
Use a vendor uninstaller for apps with drivers, extensions, or background services.

Inventories stay local. App update checks use the internet only after you enable
them. Cleanup preferences save protected paths locally; session logs and
inventories are not saved automatically. Copy any log you need before quitting.
