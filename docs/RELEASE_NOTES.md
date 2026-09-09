# Burrow 0.3.0 — release draft

This is unreleased development work. The published installer baseline remains
[0.2.0 preview](https://github.com/NobleSpartan6/burrow/releases/tag/v0.2.0-preview-2e38c4c).
Do not offer 0.3 installers until the full native release gates pass and a release
is explicitly published.

## What changes

Five native workspaces: Clean, Apps, Optimize, Analyze, and Status. They add saved
cache choices and folder protection, app/startup inspection, reviewed Mac app-bundle
removal, maintenance tasks, timed keep-awake, a drill-down folder map, process
filtering/pinning, richer OS readings, and a floating mini monitor.

This continuation adds separate single-file Trash reviews in Analyze, path-free
saved cleanup totals, and reviewed selected-package updates through existing
Homebrew or WinGet providers. Clean reloads saved protections before acting and
retains restrictions already shown in the current session. The automatic cache
allowlist and the published 0.2 upgrade baseline are unchanged.

The README now links the exact published download. The install/use guides separate
what is released, what is implemented in source, and what remains unsupported.

## Limits and recovery

This is not full Mole parity. [Feature coverage](FEATURES.md) lists the remaining
gaps and update prerequisites. Windows structured updates require preinstalled
PowerShell 7 and Microsoft.WinGet.Client and attempt current-user packages only.
Package updates run vendor installers, can change dependencies/data, and have no
Burrow rollback. Stop lets the current installer return before skipping the rest.

Windows previews are unsigned; Mac previews are ad-hoc signed, not notarized. Do
not disable security protections. Keep backups. Trash moves do not immediately
free space, and Burrow never empties Trash or permanently deletes after a failure.
Inspect both locations after an error. Hosted checks are not physical-device tests
or a guarantee of zero bugs.

[Installation](INSTALL.md) · [Using Burrow](TRY_BURROW.md) · [Safety and privacy](../SECURITY.md)
