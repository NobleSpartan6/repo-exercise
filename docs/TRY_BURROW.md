# First steps

Start with a small local folder. Keep backups, and leave system folders alone.

## Analyze: see where the space goes

Choose **Analyze → Choose folder → Analyze folder**. Larger rectangles represent
more bytes. Click a folder to look inside it; the path buttons take you back up.
Switch to **Largest files** for individual file sizes. Copy a path or show a folder
in your file manager to investigate it.

The map uses logical file sizes, not physical space that can be recovered. Links,
cloud placeholders, other volumes, and unreadable items may be excluded. A partial
scan says so. Analyze never deletes files.

## Clean: review before moving anything

Open **Clean**, expand the cache choices, and turn off any group you do not want
scanned. **Protect a folder** excludes that folder and its children from cache
results. These choices are saved on your computer; they never expand where Clean
is allowed to look.

Scan old caches, inspect the paths, and select only files you understand. Close
the apps that use them. Click **Review selection**, read the warning, and confirm.
Nothing is preselected. A filter can hide a selected row without deselecting it;
the selected total remains visible. **Clear selection** clears everything.

Burrow asks the OS to move files to Trash or Recycle Bin. Check the result and
copy the session log before quitting. Space is not freed until you empty Trash
yourself; Burrow never does that for you.

To recover a file, open Trash or Recycle Bin and use the OS recovery action.
After an error, inspect both the original location and Trash before retrying.
Stopping a run does not restore files that have already moved.

## Apps: inspect, update, or manage startup

**Installed:** click Find installed apps. Search by name or publisher, select an
app, and measure its folder or show it in your file manager. Reported Windows
sizes are estimates supplied by publishers. Mac app sizes are measured on demand.

**Mac removal:** review a selected app, quit it, then confirm moving the bundle to
Trash. The review expires after five minutes and is checked again for changes.
Related settings and data are kept. Burrow does not remove Apple system apps or
itself. Apps with drivers, extensions, or background services may require their
vendor's uninstaller instead.

**Windows removal:** click Uninstall in Windows, find the app there, and use its
uninstaller. Burrow does not execute registry uninstall command strings.

**Updates:** enable the internet check, then check WinGet on Windows or Homebrew
casks on Mac. This does not install updates. Use the original updater or store to
install them. Apps outside those sources need their own update check.

**Startup:** read registered startup items, then open system settings to change
startup behavior. A registration is not proof that the item is enabled or running.
This list does not cover every modern Mac login item or Windows scheduled task.

## Optimize: choose a specific task

Select a supported task and review it before running it. The result says whether
the OS tool completed, failed, or was skipped. Cache rebuilding can briefly slow
an app while it creates new files. Burrow does not promise a faster computer.

The screen-on button starts a 30-minute session. Stop it at any time; quitting
Burrow releases the request. It may use more battery. Explicit Sleep, lid behavior,
and device policies still belong to the OS.

## Status: find what is busy

CPU and memory readings refresh every two seconds. Process and network readings
refresh every three seconds while Status or the mini monitor is visible. Battery
readings refresh every 30 seconds. Missing sensors show as unavailable.

Search processes by name or PID, sort by CPU or memory, and pin a process near the
top. A process can use more than 100% CPU because one full core counts as 100%.
Open the system monitor for process actions; Burrow does not force-quit apps.

**Mini monitor** opens a small floating, always-on-top window. It is not a menu-bar
or system-tray app. Closing it leaves the main app open; quitting Burrow closes both.

## Saved settings could not be read or saved

Clean pauses instead of silently forgetting a protected folder. The error remains
visible on the Clean page. **Reset cleanup settings** asks before replacing the
saved choices with defaults; add your protected folders again before cleaning.
Analyze and the other workspaces still work while Clean is paused.
