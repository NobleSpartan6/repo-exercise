# Using Burrow

This guide describes **0.3 development**, not the published 0.2 preview. Start with
a small local folder and keep backups. [Need to install?](INSTALL.md)

## Analyze: find large files

Choose **Analyze → Choose folder → Analyze folder**. Larger rectangles represent
more bytes. Click a folder to look inside; breadcrumbs go back up. **Largest files**
shows individual sizes. Copy a path or reveal it in your file manager to investigate.

Scanning is read-only. To remove one file, click **Review…** in Largest files or
right-click a file rectangle and choose its review action. Check the full path and
size, acknowledge the warning, then choose **Move reviewed file to Trash**. Closing
the review or pressing Escape does nothing to the file. Reviews expire after five
minutes and changed files are refused.

Only individual regular local files inside your home folder are eligible. Folders,
app-bundle contents, protected paths, links, and cloud placeholders are excluded.
Maps use logical sizes, not recoverable physical space; partial scans say so.

## Clean: remove only what you choose

Choose cache groups and **Protect a folder** as needed. Scan old caches, inspect
the paths, and select only files you understand. Close affected apps. Choose
**Review selection**, read the warning, and confirm. Nothing is preselected;
**Clear selection** also clears selected rows hidden by a filter.

Check the result before retrying a failed move. Recover files through Trash or
Recycle Bin. Stopping does not undo completed moves. Burrow never empties Trash.
Saved totals count confirmed file moves since this feature was enabled, **not free
space**. App bundles and older-version history are not included. Counts contain no
filenames; detailed session reports disappear on quit unless you copy them.

An unreadable settings file pauses cache cleanup and manual file removal instead
of ignoring protection. Read the error on Clean. Resetting settings requires a
confirmation; add your protected folders again afterward. Read-only scans still work.

## Apps: inspect, uninstall, or update

**Installed:** find apps, search by name or publisher, and select one to inspect its
folder. Windows publisher sizes are estimates. Mac sizes are measured on demand.
Mac removal has a separate, expiring review and moves the bundle only; related data
is kept. Apps with services or drivers need the vendor's uninstaller. Windows
removal opens Installed apps and never runs registry command strings itself.

**Updates:** enable internet access, then choose **Find installable updates**.
Select apps, choose **Review selected updates**, read the warning, and confirm.
Homebrew casks are supported on Mac. Windows install controls require already
installed PowerShell 7 and `Microsoft.WinGet.Client`; only current-user packages
from the configured `winget` source are attempted. No provider is installed for you.
**Read provider report** remains a non-installing alternative. Other sources,
including App Store and Sparkle, use their own updaters.

Save work, close selected apps, and do not run another package manager operation
at the same time. Vendor installers can change app data and dependencies. Each
entry is checked again; a changed entry, expired review, or failure stops the queue.
**Stop after current update** lets the current installer return and skips the rest.
Closing Burrow follows the same rule. Completed changes cannot be undone by Burrow.
Read each outcome, reopen the app to check its version, and inspect the package
manager after an error. No automatic retries, reboots, or agreement acceptance.

**Startup:** inspect registrations, then open OS settings to make changes. Being
listed does not mean an item is enabled or running; coverage is not exhaustive.

## Optimize and Status

**Optimize:** review a supported task before running it. Failures remain failures;
Burrow does not promise a faster computer. **Keep screen on** lasts 30 minutes;
Stop or Quit releases it. OS sleep policies and lid behavior still apply.

**Status:** inspect CPU, memory, drives, processes, and available power/sensor data.
Search or pin a process; use the system monitor for process actions. One fully used
core counts as 100% process CPU. Missing sensors show unavailable, not zero.
**Mini monitor** opens a separate floating window, not a tray/menu-bar popover.
Closing it leaves Burrow open; quitting closes both.

**Keyboard:** Ctrl/Command+1–5 changes workspace; +6 opens Help. Tab/Space operates
controls; Escape closes a review or requests Stop. Plus/minus adjusts text size.
