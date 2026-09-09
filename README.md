# Burrow

**Find what is taking up space. Decide what to remove.**

Burrow is a free desktop utility for Mac and Windows. It helps you review old
caches, explore storage, inspect installed apps, and see what your computer is
doing. It is inspired by Mole, but is an independent project—not the official
Mole app.

**[Download Burrow](https://github.com/NobleSpartan6/burrow/releases)** ·
**[Install or update](docs/INSTALL.md)** · **[First steps](docs/TRY_BURROW.md)**

## Choose your download

| Computer | File to download |
| --- | --- |
| Mac with an Apple M-series chip | `Burrow-…-macOS-AppleSilicon.dmg` |
| Mac with an Intel processor | `Burrow-…-macOS-Intel.dmg` |
| Windows with an Intel or AMD 64-bit processor | `Burrow-…-Windows-x64-Setup.exe` |

On Mac, open the download and drag Burrow into Applications. On Windows, open
the installer and choose Install. No terminal, account, or subscription is needed.

**This is preview software.** Windows downloads are unsigned. Mac downloads are
ad-hoc signed but not notarized. Read the [installation guide](docs/INSTALL.md)
before opening a download; do not turn off your computer's security protections.

## Five workspaces

| Workspace | What you can do in 0.3 |
| --- | --- |
| **Clean** | Review old cache files, exclude cache groups, protect folders, and move selected files to Trash or Recycle Bin. |
| **Apps** | Find installed apps, inspect their files, read startup registrations, and check supported update sources. Review app-bundle removal on Mac; open the system uninstaller on Windows. |
| **Optimize** | Review specific maintenance tasks and their results, open system tools, or keep the screen on for a timed session. |
| **Analyze** | Explore a folder map, open a subfolder, follow breadcrumbs, and find the largest files. Nothing is deleted here. |
| **Status** | See CPU, memory, network, battery, available temperatures, storage, and processes. Filter or pin processes, or open a small floating monitor. |

Not every Mole feature is present. See [feature coverage](docs/FEATURES.md) for
what works directly, what opens a system tool, and what is not supported.

## Your files stay in your control

Nothing is selected or removed automatically. Clean only scans a short list of
known cache folders. It does not sweep your documents or whole system. Mac app
removal has a separate review and keeps related data; use the vendor's uninstaller
for apps with services or drivers.

Keep backups. Moving files to Trash **does not immediately free space**. Burrow
never empties Trash, and stopping a task does not undo completed changes.

Burrow has no telemetry, ads, account, or subscription. Scans stay on your
computer. App update checks contact your configured WinGet or Homebrew sources
only after you enable the internet check. Cache exclusions and protected paths
are saved locally; inventories and session logs are not saved automatically.

## Small by design

Burrow uses native Rust and egui, not Electron or an embedded browser. Scans run
away from the interface. Long lists draw only visible rows. There is no rotating
planet, animated backdrop, or background service after you quit.

[Performance notes](docs/PERFORMANCE.md) · [Build from source](docs/DEVELOPMENT.md) ·
[Security and recovery](SECURITY.md) · [Changes](CHANGELOG.md) · [MIT license](LICENSE)

Found a problem? [Report it here](https://github.com/NobleSpartan6/burrow/issues/new/choose).
Please remove personal paths and private information from screenshots and logs.
