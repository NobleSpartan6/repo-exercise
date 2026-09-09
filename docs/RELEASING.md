# Build, verify and release Burrow 0.3.0

## Automated preview path

`.github/workflows/burrow.yml` runs on relevant pushes, pull requests and manual dispatch. It requires the committed Cargo.lock, checks Rust formatting and packaging regression tests, generates third-party notices and an exact locked source ZIP, then shares those inputs across the build matrix. It does not silently refresh dependency versions.

The matrix builds on `macos-14` (Apple Silicon), `macos-15-intel` (Intel), `windows-2022` (Windows x64), and `ubuntu-24.04` (Linux QA only). Each runs unit/UI tests, strict Clippy and an optimized build. Mac and Windows additionally run a uniquely named disposable native Trash/recovery test. Windows verifies the published 0.1.1 installer's digest, installs it into an isolated directory, upgrades to this release, checks executable identity, renders the installed app and uninstalls. Mac mounts the DMG read-only, copies the app, checks version, hash and ad-hoc signature, then renders the copy. Native screenshot checks exercise all six screens at desktop, compact and enlarged-text settings. Linux also exercises native navigation/resize and a synthetic metadata-scan benchmark.

Only a successful main push or explicit main dispatch with `publish_preview` selected may publish. The release job depends on every build and has the only release-write permission. It checks all expected assets, creates SHA-256 checksums, uploads to a draft, then publishes a prerelease. Failed builds cannot publish. Existing published assets are never silently overwritten; an unfinished draft must be inspected before retrying.

Tags follow `v<version>-preview-<short SHA>`. Cargo.toml is the version source for the executable, chrome, package names and release title. Update Burrow's Cargo.lock entry together with its manifest; never discard the lock to solve a build failure. Renderer linkage tests guard the DirectX allocator/renderer Windows-type mismatch found during 0.2 preparation.

## Expected assets

- `Burrow-0.3.0-macOS-AppleSilicon.dmg`
- `Burrow-0.3.0-macOS-Intel.dmg`
- `Burrow-0.3.0-Windows-x64-Setup.exe`
- `Burrow-0.3.0-Windows-x64-portable.zip`
- `Burrow-0.3.0-source.zip`
- `Cargo.lock`, `THIRD_PARTY_NOTICES.txt`, `SHA256SUMS.txt`

The `native-QA-<platform>` Actions artifacts contain actual native captures and launch/install reports. Inspect screenshots and logs, not just job names. Release preflight rejects missing, empty, unexpected or mixed-version assets. Each published release links its exact source commit and build run.

## Linux bootstrap isolation

A release attempt exposed a hash mismatch in the runner's unrelated Chrome package index. Burrow's Linux setup now scopes both apt commands to `/etc/apt/sources.list.d/ubuntu.sources` and excludes other source parts for those commands only. It does not alter `/etc/apt`, disable package signatures or checksum checks, or ignore update failures. The commands have four-minute timeouts and three acquisition retries. `test_ci_bootstrap.py` guards source scoping, required packages, valid Bash and failure handling.

References: [APT configuration](https://manpages.ubuntu.com/manpages/noble/man5/apt.conf.5.html), [runner image setup](https://github.com/actions/runner-images/blob/main/images/ubuntu/scripts/build/configure-apt.sh).

DMG creation retries only the observed transient Resource busy error, at most four attempts with 2/4/8-second delays. Other failures remain failures. No retry detaches other mounted drives; signature and image verification are mandatory.

## Manual packaging

Install developer prerequisites from docs/DEVELOPMENT.md; end users only need the prebuilt installer. Use the pinned Rust compiler and committed lock. Native packaging requires Python 3.11+; Windows also requires Inno Setup 6.

```sh
cargo fmt --all -- --check
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
# On the matching platform:
python3 scripts/package.py macos apple-silicon
python3 scripts/package.py macos intel
python scripts/package.py windows
```

Use the CI pipeline to collect all platform packages, run `verify-release`, generate checksums and publish. The Windows installer is per-user with static C runtime configuration; portable ZIPs are separate copies. Mac DMGs contain a real app, Applications shortcut and installation instructions. No proprietary Mole branding or bundled operating-system font files are distributed.

## Signing and remaining validation

Windows previews are unsigned. Mac previews are ad-hoc signed, not Developer ID signed or notarized. Do not buy certificates, invent credentials, remove quarantine attributes or disable OS protections as a release shortcut. Provision production signing under the maintainer's control in a protected secret store; sign before checksums, notarize/staple Mac packages, and test real browser-downloaded files.

A green hosted-runner run is not a physical-device, Gatekeeper, SmartScreen or independent security certification. Keep prerelease status until representative hardware, display scaling, assistive technology, network/cloud volumes and real recovery workflows have been checked. Keep backups and begin with read-only Analyze. Never claim zero flaws or guaranteed recovery. New failures should become reproducible regression tests; do not weaken a check merely to make a build pass.
