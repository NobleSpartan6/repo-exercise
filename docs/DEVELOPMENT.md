# Build and contribute

Burrow is a Rust application with an egui interface. The desktop renderer uses
Metal on Mac, DirectX 12 on Windows, and OpenGL on the Linux QA target.

## Build locally

Install the platform's Rust build tools, then run these commands from the repo:

```sh
cargo build --locked --release
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all -- --check
python -m unittest discover -s scripts -p 'test_*.py'
```

`rust-toolchain.toml` selects Rust 1.90. Linux additionally needs the graphics
packages listed in `.github/workflows/burrow.yml`. Installer packaging uses
Python 3.13, macOS system tools, or Inno Setup on Windows; see
[Releasing](RELEASING.md).

Core scan tests can run without the desktop features:

```sh
cargo test --locked --no-default-features --lib
```

## Code map

| Area | Files |
| --- | --- |
| Cache review, validation, Trash, folder totals | `src/engine.rs`, `src/platform.rs` |
| App/startup discovery and Mac removal review | `src/software.rs` |
| Fixed OS tools, deadlines, maintenance | `src/command.rs`, `src/maintenance.rs` |
| Local cleanup choices | `src/preferences.rs` |
| Timed power request | `src/awake.rs` |
| Sampling and bounded UI messages | `src/monitor.rs`, `src/latest.rs` |
| Native workspaces and visual style | `src/ui.rs`, `src/workspaces.rs`, `src/design.rs` |
| Folder-map geometry | `src/treemap.rs` |
| UI, installed-binary and packaging checks | `src/ui_tests.rs`, `src/qa.rs`, `scripts/` |

## Test real behavior

Every bug fix should include a regression test where practical. Use temporary
fixtures, not personal caches or installed apps. The ignored native tests are for
disposable runners. One existing recovery test moves and restores a uniquely
named disposable file; the app inventory test is read-only.

`--smoke-test` visits all six screens at three sizes and saves app-surface captures
only when `BURROW_SMOKE_OUTPUT` points to a new evidence folder. It does not scan
or clean personal files. `--interaction-test` exposes control positions for the
Linux pointer test. Neither mode is active in a normal launch.

Do not weaken assertions or skip a failing platform to get a release through.
Record the failure, fix its cause, rerun the affected tests, and inspect the UI.
Do not claim full Mole parity: update [feature coverage](FEATURES.md) as work lands.
