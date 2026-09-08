# Burrow

**A free, native desktop cleaner and disk explorer for Mac and Windows.**

Review old caches. Find large files. See CPU, memory and drive usage. No account, subscription, ads, telemetry, Electron, browser runtime, or command line required for end users.

Burrow is an **independent, Mole-inspired implementation**, not the official Mole GUI, not a wrapper around its shell scripts, and not a full replacement for every Mole command. No Mole source or proprietary GUI assets are included. This project's code is MIT licensed.

> **Preview software:** the first builds are unsigned/unnotarized. Back up important data, review every selection, and read the [safety boundaries](SECURITY.md). A passing build is not a substitute for testing the installer and recovery on your own OS version. Burrow does not promise a speed increase from clearing caches.

## 0.1.1 maintenance update

The existing white-and-green native interface is retained. Memory readings now
show the OS-reported total and percentage used. Drive labels sit outside uniform
usage bars, low-space warnings include text, and the entire Overview scrolls on
short windows or at larger display scales. Drive polling is isolated from CPU/RAM
sampling, and monitoring no longer requests redraws while another page is open.
Choosing a different explorer folder clears the previous folder's result.

A user reported successful **0.1.0 Windows installation and Overview launch**.
That report does not validate 0.1.1, Mac installation, or native cleanup/restoration.
See [changes](CHANGELOG.md) and [release notes](docs/RELEASE_NOTES.md).
Download installers from Releases; source archives are for developers, not
ready-to-run updates.

## Install without using a terminal

Open **[Releases](https://github.com/NobleSpartan6/repo-exercise/releases)** and select the newest Burrow preview. Expand **Assets**. Download an installer, **not** GitHub's “Source code” archive.

| Your computer | Download | Install |
| --- | --- | --- |
| Mac with an Apple M-series chip | `Burrow-0.1.1-macOS-AppleSilicon.dmg` | Open it, drag **Burrow** into **Applications**, then open Burrow there. |
| Mac with an Intel processor | `Burrow-0.1.1-macOS-Intel.dmg` | Open it, drag **Burrow** into **Applications**, then open Burrow there. |
| Windows PC with an Intel or AMD 64-bit processor | `Burrow-0.1.1-Windows-x64-Setup.exe` | Open it, choose **Install**, then launch **Burrow** from Start. No administrator install is required. |

On a Mac, **Apple menu → About This Mac** shows either **Chip: Apple M…** or **Processor: Intel…**. A Windows portable ZIP is also produced; extract the entire ZIP before opening `burrow.exe`.

**[Read the step-by-step installation, warning and recovery guide →](docs/INSTALL.md)**

If Releases has no Burrow installers yet, there is **not yet a ready-made download**. Check the [build workflow](https://github.com/NobleSpartan6/repo-exercise/actions/workflows/burrow.yml). Installers are published only after every platform job succeeds. Nontechnical users should not try to open the source ZIP as an application.

### First use

Open **Clean up → Scan caches**. The scan is read-only and starts with **nothing selected**. Review paths, close the relevant apps, select files you recognize, and choose **Review & move to Trash**. Confirm only after reviewing the file count and size. **Disk explorer** is always read-only.

**Moving files to Trash does not immediately free disk space.** Burrow never empties Trash. Keep files there while checking that everything works; manually emptying it later is permanent and is your decision. A cancelled cleanup does not undo moves already completed.

## What is implemented

| Feature | macOS | Windows |
| --- | --- | --- |
| Review older allowlisted caches | Homebrew downloads, pip, uv, Go, Chrome, Edge, npm | pip, uv, Go, npm, Chrome/Edge Default-profile caches |
| Explicit per-file selection and confirmation | Yes | Yes |
| Native OS Trash / Recycle Bin request | Yes | Yes |
| Read-only largest-file explorer | Yes | Yes |
| Live CPU, RAM and drive capacity | Yes | Yes |
| Session log with original paths | Yes; copy before quitting | Yes; copy before quitting |
| App uninstall, registry edits, system “optimization,” Docker purge or startup management | **Not implemented** | **Not implemented** |

Target packaging: **macOS 12+**, Apple Silicon and Intel; **Windows 11 x64**, with Windows 10 22H2 compatibility intended but not a separate tested target. Native Windows ARM64 and 32-bit builds are not provided. CI runs on the specific OS versions recorded in each workflow run; do not interpret minimum deployment targets as a full OS compatibility test matrix. Linux is used for development QA only and has **no cleanup allowlist**.

## Performance by design

The Rust executable draws through egui/OpenGL. There is no embedded Chromium engine and no JavaScript bridge. CPU/memory sampling happens on one background worker every two seconds; a separate worker refreshes drive capacity every ten seconds after each completed reading. A blocked volume query does not block CPU/memory sampling. Each stream retains only its latest sample. Monitoring requests redraws only on Overview; other pages still redraw for user input and active jobs. There is no continuous animation loop at idle.

Cleanup scans use at most **four worker threads** and metadata only. Result lists render only visible rows; filtering runs only when the filter or scan changes. Cleanup actions run serially off the UI thread so each outcome is recorded. Disk exploration uses a **single pass and a bounded top-200 heap**, rather than retaining an entire directory tree.

Per scan, the engine caps traversal at **500,000 entries**, **128 directory levels**, and approximately **120 seconds**, and caps cleanup candidates at **20,000**. Limit and cancellation results are labelled partial. Cancellation is cooperative: a stalled OS filesystem call cannot be interrupted immediately. A capped preview is not a claim about total reclaimable space.

There are **no fabricated startup, memory or speed benchmarks** here. `cargo run --release --example scan_bench` measures a documented synthetic workload. The Linux QA workflow records its result; that is not a benchmark of a user's Mac or Windows PC. See [performance measurement](docs/PERFORMANCE.md).

## Safety and privacy

Only the known OS cache roots in [`src/platform.rs`](src/platform.rs) are eligible. A folder selected in Disk explorer cannot become a cleanup target. Before each Trash operation the engine checks the current allowlist, every path component, containment, regular-file type, file size and timestamps, and, on Unix, inode/device identity and hard links. Symlinks and Windows reparse points/junctions are refused. Changed and inaccessible files are reported, not silently counted as cleaned.

No permanent-delete fallback, privilege escalation, scheduled cleaner, process killer, downloaded shell script, or auto-empty operation is implemented. All results stay in memory; file paths are not sent anywhere or persisted by Burrow. Clicking a help/source link explicitly opens your browser. The operating system and build infrastructure have their own networking behavior.

These are safeguards, **not a claim of perfect safety**. The remaining path-based race window, cache availability tradeoffs, Windows identity limits, Trash recovery limitations and non-adversarial threat model are documented in [SECURITY.md](SECURITY.md).

## Build from source — developers only

Install Rust using [rustup](https://rustup.rs). Windows developers also need Visual Studio Build Tools with the **Desktop development with C++** workload and a Windows SDK; Mac developers need Xcode Command Line Tools. The repository pins Rust 1.90.0. End users installing a release do not need any of these.

```sh
git clone https://github.com/NobleSpartan6/repo-exercise.git
cd repo-exercise
cargo run --release
```

For a **locked release build**, download the source archive attached to the release and use `cargo build --release --locked`. The 0.1.1 source retains the 0.1.0 release's transitive dependency versions and changes only Burrow's own version. `Cargo.lock` is required and shared across all platform jobs; CI refuses to silently resolve new versions when it is missing. Update dependencies deliberately in a separate reviewed change. Locked inputs alone do not guarantee byte-identical binaries across build environments.

```sh
cargo test --all-targets
cargo clippy --all-targets
cargo fmt --all
cargo run --release --example scan_bench
# Core-only tests without compiling the desktop GUI:
cargo test --no-default-features --lib
```

Packaging and signing details are in [RELEASING.md](docs/RELEASING.md). The workflow generates Apple Silicon and Intel DMGs, a per-user Windows EXE installer, a Windows portable ZIP, a locked source ZIP, third-party notices and SHA-256 checksums. Standard hosted Actions runners are [free for public repositories](https://docs.github.com/en/actions/reference/runners/github-hosted-runners); private repositories have different billing limits. No paid signing service is configured.

## Project layout

```text
src/engine.rs       Read-only discovery, safety guards, cleanup transactions, tests
src/platform.rs     Explicit per-OS cache allowlist
src/ui.rs           Native GUI, selection, confirmation, background-job lifecycle
src/monitor.rs      Separate CPU/RAM and drive workers
src/latest.rs       Bounded latest-value measurement mailbox
src/metrics.rs      Capacity validation and display-only warning thresholds
scripts/           Packaging, license notices, GUI smoke tests
packaging/         Per-user Windows installer definition
docs/              Installation, recovery, performance and release guidance
```

## Inspiration and upstream projects

[Mole by Tw93](https://github.com/tw93/mole) inspired the maintenance workflow. Its [product page](https://faberon.io/projects/mole) describes a GPL-3.0 Mac CLI and a separate paid Mac GUI. Burrow does not reuse that code or imply affiliation. Core libraries include [egui/eframe](https://github.com/emilk/egui), [sysinfo](https://github.com/GuillaumeGomez/sysinfo), [trash-rs](https://github.com/Byron/trash-rs), [rfd](https://github.com/PolyMeilex/rfd), [Rayon](https://github.com/rayon-rs/rayon), and [WalkDir](https://github.com/BurntSushi/walkdir). Release packages include generated third-party notices.
