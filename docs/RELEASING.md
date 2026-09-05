# Build, package and release

## Automated preview path

`.github/workflows/burrow.yml` runs on relevant pushes, pull requests and manual dispatch. It resolves one dependency lock, generates third-party notices and a locked source ZIP, then shares that lock with all matrix jobs. Each platform runs unit tests and Clippy and builds the optimized executable. Linux additionally opens the real native GUI under Xvfb and tests all four navigation shortcuts, saves screenshots/logs, and records the synthetic scan benchmark.

Targets are `macos-14` (Apple Silicon), `macos-15-intel` (Intel), and `windows-2022` (Windows x64). Linux is QA only. Hosted labels can change; review the official [runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) when maintaining this workflow. A build's runner OS is not a test of every deployment OS version.

Only a successful `main` push, or an explicit manual dispatch on `main` with `publish_preview` selected, may publish a preview. Pull requests do not publish. The release job alone has `contents: write`; checkout credentials are not persisted. It creates a draft release, uploads completed artifacts and checksums, and then publishes it as a prerelease. Existing releases for the same commit are not overwritten.

The tag includes the source commit's short SHA. The version in `Cargo.toml`, the Inno Setup default, app help text, README examples, and workflow release prefix must be updated together for a version change.

## Expected artifacts

- `Burrow-0.1.0-macOS-AppleSilicon.dmg`
- `Burrow-0.1.0-macOS-Intel.dmg`
- `Burrow-0.1.0-Windows-x64-Setup.exe`
- `Burrow-0.1.0-Windows-x64-portable.zip`
- `Burrow-0.1.0-source.zip` containing the exact release `Cargo.lock`
- `Cargo.lock`, `THIRD_PARTY_NOTICES.txt`, `SHA256SUMS.txt`

The `native-gui-QA` Actions artifact contains actual native-window screenshots and a smoke-test log. Screenshots are QA evidence, not a static UI implementation. Inspect them before release promotion. CI records build/test results; installers still need real-device install and recovery testing.

The first source commit need not contain the generated lockfile because the development environment may be unable to resolve it. Every published release must include its resolved lock and locked source bundle, and all targets in a run consume that same lock. Commit a reviewed `Cargo.lock` for future development and update it deliberately. Do not present builds from separately resolved locks as byte-reproducible.

## Manual packaging

Install the development prerequisites in the README, then use a release source bundle containing `Cargo.lock` and third-party notices. Python 3.11+ is required for packaging, not for end users.

```sh
cargo test --locked --all-targets
cargo build --locked --release
# Run on an Apple Silicon Mac:
python3 scripts/package.py macos apple-silicon
# Run on an Intel Mac:
python3 scripts/package.py macos intel
# Run on Windows after installing Inno Setup 6:
python scripts/package.py windows
python scripts/package.py checksums
```

macOS packages include a real `.app` bundle, an Applications shortcut in the disk image, and an ad-hoc signature. The DMG is verified using `hdiutil verify`. Windows uses a per-user Inno Setup installer; the MSVC target is configured for the static C runtime. A portable ZIP is also built. Generated app icons and source code contain no bundled proprietary Mole assets.

## Trusted signing is not configured

An ad-hoc Apple signature is **not** a Developer ID identity, notarization, or evidence of malware review. Windows previews are unsigned. The app is free to use, but a frictionless trusted-publisher installation experience is a separate distribution/signing problem. This repository does not purchase certificates, enroll in paid programs, invent credentials, or globally disable OS protections.

For production, provision a real signing identity under the maintainer's control. Keep certificates, passwords and keys in a protected secret store; restrict release environments and credentials to approved branches. Apply verified Windows signing before computing checksums. On Mac, use an appropriate Developer ID signing configuration, submit to Apple's notarization service, staple the ticket, and test a browser-downloaded/quarantined package on a clean machine. Treat signing as an additional integrity and identity control, not a substitute for reviewing code and dependencies.

## Promotion checklist

Verify native launch, high-DPI layout and keyboard access on physical Apple Silicon, Intel and Windows machines. In disposable test caches, exercise no-selection defaults, age filters, exact selections including hidden rows, confirmations, changed/locked files, junction/symlink rejection, cancellation after partial progress, actual Trash requests and manual restoration. Check that logical-size claims are not confused with free disk space. Confirm the app does not ask for admin rights or full-disk access simply to increase cleanup coverage.

Run performance tests on representative hardware; review dependencies, licenses and generated third-party notices; verify installer and portable uninstall behavior; verify artifact checksums against the published files; inspect GUI screenshots and all CI job conclusions. A green automated build is necessary but does not itself complete this checklist. Keep prerelease status until these checks are complete.
