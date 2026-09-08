# Changelog

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
