# Changelog

## 0.3.0 — in development (not published)

Continues from the unchanged 0.2.0 preview `v0.2.0-preview-2e38c4c`.

New in this continuation: explicit single-file Trash reviews in Analyze;
path-free atomic cleanup totals; selected, revalidated Homebrew/WinGet updates
with stop-after-current behavior; Windows Clippy and Linux native-window discovery
fixes; and shorter install/use/coverage guides that separate releases from source.
No new runtime dependencies or expanded automatic cache roots.

Adds app/startup inventories, opt-in update checks, reviewed Mac bundle removal,
maintenance tasks, a timed screen-on session, a drill-down folder map, richer
status readings, process filters/pins, and a floating mini monitor. Cleanup now
saves cache choices and protected folders. The README and guides are rewritten
for everyday use. [Feature coverage](docs/FEATURES.md) lists platform differences
and features still missing from the Mole reference.

Expanded tests cover parsing failures, cancellation, changed app bundles,
preferences, proportional maps, filters, virtual rows, native discovery, and
six-screen rendering. Preview signing limitations remain unchanged.

## 0.2.0

Modern native dark interface, top navigation, system typography, scrollable compact layouts, virtualized rows and cached totals. Expanded native startup, confirmation, packaging, installer and disposable Trash recovery tests. Updated repository links. No broader cleanup permissions, browser runtime, or decorative animation loop.


## 0.1.1 — maintenance preview

Retains the existing visual design and cleanup allowlist. Adds memory totals and
percentages, drive labels outside uniform usage bars, text low-space warnings,
unknown-capacity handling, a scrollable Overview with narrow-width stacking,
separate CPU/RAM and drive workers, bounded latest-value sample delivery, and
suppression of monitoring-triggered repaints outside Overview. Clears stale
explorer results after folder changes. App versions, installer names and release
tags share the Cargo package version. Includes the existing transitive lock,
packaging regression tests, release-asset checks and transient DMG retry. Removes
the obsolete staged-tree publisher so it cannot reattach old bootstrap source.

See the version's release and CI run for build status. Publication is gated on
all platform builds, unit tests, packaging checks and the Linux native-window
smoke test. Real-device upgrades and restoration remain separate checks.

## 0.1.0 — initial unsigned preview

Introduced the native Rust desktop GUI, allowlisted cache review, explicit
selection/confirmation, OS Trash requests with revalidation, read-only largest
file inspection, CPU/RAM/drive overview and per-user/native installers.

A user later reported successful Windows installation and Overview launch.
This is user-reported old-version evidence only; no private screenshot or drive
information is included in the repository.
