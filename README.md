# Burrow

**See what takes up space. Choose what to remove.**

A free, native desktop utility for Mac and Windows. No account, subscription,
ads, Electron, or background service. Inspired by Mole; not the official Mole app.

## Install

**The installable baseline is [Burrow 0.2.0 preview](https://github.com/NobleSpartan6/burrow/releases/tag/v0.2.0-preview-2e38c4c).**
This `burrow/0.3.0-workspaces` branch is development source, not a published 0.3
release. Features below describe this branch, not the older download.

| Your computer | Download 0.2.0 |
| --- | --- |
| Mac with an Apple M-series chip | [Apple Silicon DMG](https://github.com/NobleSpartan6/burrow/releases/download/v0.2.0-preview-2e38c4c/Burrow-0.2.0-macOS-AppleSilicon.dmg) |
| Mac with an Intel processor | [Intel DMG](https://github.com/NobleSpartan6/burrow/releases/download/v0.2.0-preview-2e38c4c/Burrow-0.2.0-macOS-Intel.dmg) |
| Windows, Intel or AMD 64-bit processor | [Windows installer](https://github.com/NobleSpartan6/burrow/releases/download/v0.2.0-preview-2e38c4c/Burrow-0.2.0-Windows-x64-Setup.exe) |

**Mac:** open the DMG and drag Burrow into Applications. **Windows:** open the
installer and choose Install. No terminal or GitHub account is needed.

These are unsigned/not-notarized previews. Do not disable security protections.
[Installation, updates, and troubleshooting →](docs/INSTALL.md)

## What is in 0.3 development?

| Workspace | Use it to… |
| --- | --- |
| **Clean** | Review old caches, protect folders, and move chosen files to Trash. See locally saved cleanup totals. |
| **Apps** | Inspect apps and startup entries, review supported package updates, remove a Mac app bundle, or open Windows' uninstaller. |
| **Optimize** | Run a reviewed maintenance task, open a system tool, or keep the screen on for 30 minutes. |
| **Analyze** | Explore a folder map and its largest files. Separately review an individual local file before moving it to Trash. |
| **Status** | Check live resource readings, filter or pin processes, and open a small floating monitor. |

Nothing is selected or removed automatically. Keep backups and close affected
apps first. **Moving files to Trash does not immediately free space.** Burrow
never empties Trash. Package updates run vendor installers and cannot be undone
by Burrow.

[How to use it](docs/TRY_BURROW.md) · [Supported features and remaining gaps](docs/FEATURES.md)

## For contributors

Rust + egui, bounded workers, virtualized lists, and no decorative animation loop.
The 0.2 release and its upgrade-test baseline remain unchanged. A new release
requires all existing native build, installer, recovery, and UI checks to pass.

[Build and test](docs/DEVELOPMENT.md) · [Performance](docs/PERFORMANCE.md) ·
[Safety and privacy](SECURITY.md) · [Changes](CHANGELOG.md) · [MIT license](LICENSE)

[Report a problem](https://github.com/NobleSpartan6/burrow/issues/new/choose).
Include the version and OS; remove private paths and details from screenshots.
