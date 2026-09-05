# Burrow 0.1.0 — unsigned preview

A free, independent Mole-inspired native desktop app for macOS and Windows. This is an initial preview, not the official Mole GUI and not a full port of Mole's feature set.

Implemented: explicit review and selection of older allowlisted cache files; serial OS Trash requests with revalidation and per-file outcomes; a read-only top-200 largest-file explorer; CPU, RAM and drive monitoring; cancellation; a copyable in-memory session log.

## Choose your download

Mac M-series: `macOS-AppleSilicon.dmg`. Intel Mac: `macOS-Intel.dmg`. Windows Intel/AMD 64-bit: `Windows-x64-Setup.exe`. A Windows portable ZIP and a locked source ZIP are also attached. **Source code ZIPs are not installers.**

Installation and recovery: [docs/INSTALL.md](https://github.com/NobleSpartan6/repo-exercise/blob/main/docs/INSTALL.md).

## Important limitations

Packages have no verified publisher signature. Mac bundles are ad-hoc signed but not Developer ID signed or notarized; Windows packages are unsigned. Read OS warnings, verify the download source and do not disable system protections.

Moving to Trash does **not** immediately free disk space. Nothing is preselected; the app never empties Trash. Close relevant applications first. Cancellation does not undo completed moves. Keep backups; Trash recovery depends on OS behavior and policy.

Scans are capped and label partial results. Sizes are logical, not uniquely allocated bytes. App uninstall, system optimization, registry editing, Docker purge and startup management are not implemented. Native Windows ARM64/32-bit installers are not provided.

The release workflow requires all configured build/test jobs to pass. Automated tests and Linux GUI navigation checks are not a substitute for real-device installation, native cleanup/recovery, accessibility and performance validation. The project has not undergone an independent security audit.

The locked source archive, exact Cargo.lock, generated third-party notices and SHA-256 checksums accompany this release. Checksums verify artifact bytes, not publisher trust or absence of malware.
