# Burrow

**Room to breathe.** A small, free, native desktop app for Mac and Windows.

[Download an installer](https://github.com/NobleSpartan6/burrow/releases) · [Install or update](docs/INSTALL.md) · [What's changed](CHANGELOG.md)

Burrow 0.2 brings a quiet charcoal-and-mint interface, centered navigation, clearer
system readings, and carefully spaced file lists. The design takes inspiration
from Mole's restrained desktop interface without copying its branding or adding
features Burrow does not implement.

## Install without a terminal

Open **Releases**, choose the latest Burrow preview, and expand **Assets**.

| Your computer | Download | Install |
|---|---|---|
| Windows Intel/AMD 64-bit | `Windows-x64-Setup.exe` | Close Burrow, open the installer, then choose Install. |
| Mac with an Apple M-series chip | `macOS-AppleSilicon.dmg` | Open the DMG and drag Burrow into Applications. |
| Intel Mac | `macOS-Intel.dmg` | Open the DMG and drag Burrow into Applications. |

To update, install over the existing copy; on Mac choose **Replace**. No developer
tools or uninstall are needed. Windows portable ZIPs are separate installations.
**Source ZIPs are not installers.** See the [plain-language guide](docs/INSTALL.md).

**Preview notice:** Windows packages are not publisher-signed. Mac packages are
ad-hoc signed but not notarized. Security warnings are possible. Do not disable
system protections, use quarantine-removal commands, or run the app elevated.
An automated test pass is not a guarantee of flawless operation on every machine.

## Four screens. Clear boundaries.

**Overview** shows real CPU, memory, and OS-reported drive capacity. There is no
invented health score or promise to speed up your computer.

**Clean up** reviews older files in a narrow cache allowlist. Nothing is
preselected. Review paths, close affected apps, select files, and confirm before
requesting a move to Trash / Recycle Bin. Files are revalidated before each move.
No automatic cleanup; no empty-Trash command; no permanent-delete fallback in
Burrow's own code. Trash behavior and recovery also depend on the operating system.

**Disk explorer** reads metadata to find the largest 200 files in a chosen folder.
It cannot delete files. **About & help** explains the limits, shortcuts and updates.

Moving to Trash does **not** immediately free disk space. Recovery is not a
backup. Cancellation does not undo completed moves. Keep backups and inspect
Trash after an error. [Safety details](SECURITY.md).

## Small by design

Compiled Rust + GPU-rendered egui, not Electron or Chromium. Bounded scan workers,
virtualized rows, cached preview totals, independent CPU/RAM and drive workers,
no continuous decorative animation, no remote assets or bundled system fonts.
Everything except user-clicked help links is local. No account, ads, telemetry,
subscription, automatic updater, or service left running after quitting.

[Performance and measurement](docs/PERFORMANCE.md) · [Design specification](docs/DESIGN.md)

## Verification

Releases are gated on native Mac/Windows builds, unit and interaction tests,
packaging tests, a disposable native Trash/recovery check, Windows installation /
in-place upgrade / launch / uninstall checks, copied Mac-app launch checks, and
Linux screenshot/navigation/resize checks. Each release links its exact source
commit and CI run. Read the actual run results rather than assuming a test passed.

Physical machines, high-DPI assistive technology, Gatekeeper, SmartScreen, network
volumes, and unusual OS configurations still need separate testing. See
[release notes](docs/RELEASE_NOTES.md) and [maintainer instructions](docs/RELEASING.md).

## Build from source (developers)

Install the pinned Rust toolchain with rustup. On Windows use the MSVC toolchain
and Visual Studio C++ build tools. On Mac install Xcode command-line tools.

```sh
cargo test --locked --all-targets
cargo run --locked --release
```

`--version` prints the app version. `--smoke-test` opens the actual native window,
renders all four screens without scanning or moving files, and closes it.
No GUI-free mode performs cleanup. `cargo test --no-default-features` runs the
engine without the desktop stack. Native packaging requires Python 3.11+;
Windows packaging additionally needs Inno Setup 6.

MIT licensed. Independent project; not affiliated with Mole, Tw93, or Faberon.

Native rendering in 0.2.0 uses Metal on Mac and DirectX 12 on Windows, with a low-power adapter preference. Linux remains a separate OpenGL QA target. No browser runtime is introduced. The pinned eframe diagnostic environment variable `EFRAME_SCREENSHOT_TO` captures the native window and exits when explicitly set by a maintainer; normal launches do not capture or save screenshots.
