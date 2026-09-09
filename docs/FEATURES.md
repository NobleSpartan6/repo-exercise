# Feature coverage

Burrow 0.3 brings Mole's five-workspace structure to Mac and Windows. It is not a
complete port of Mole. This table separates working features from system handoffs
and features that are still missing.

| Feature | Burrow 0.3 |
| --- | --- |
| Old cache review and selected-file cleanup | Built in; narrow known cache folders only. |
| Cache group choices and protected folders | Built in; saved locally. |
| Cleanup confirmation and results | Built in; OS Trash/Recycle Bin, no permanent-delete fallback. |
| App inventory, search, size inspection | Built in; Mac Applications folders; Windows desktop/Store registrations. Portable apps may be absent. |
| App removal | Mac bundle-only review and Trash; Windows opens Installed apps for the vendor's uninstaller. |
| Related app files | Exact bundle-ID matches can be inspected on Mac; they are never removed automatically. |
| App update discovery | Opt-in WinGet or Homebrew cask check. Missing providers or source setup are reported. |
| Installing app updates | Use the app's updater, package manager, or store. No built-in bulk update installer. |
| Startup inventory | Windows Run keys and startup folders; Mac third-party LaunchAgents/LaunchDaemons. |
| Startup changes | Opens the OS settings. No in-app service/agent toggles. |
| Maintenance | Reviewed Quick Look tasks on Mac, local lookup-cache task on Mac/Windows, and system-tool shortcuts. |
| Keep screen on | Opt-in 30-minute native request; stop early or quit to release it. |
| Folder map and drill-down | Built in; breadcrumbs, direct-child totals, largest files, copy/reveal. Read-only. |
| CPU, RAM, swap, disk, network, uptime | Built in using OS readings. No invented health score. |
| Battery charge and power state | Built in where reported. Desktops may have no battery. |
| Temperature | Shown only for sensors the OS provider exposes. |
| Process list | Search, sort, and pin; opens the native system monitor for process actions. |
| Small monitor | Floating native window; not a tray or menu-bar popover. |
| Keyboard navigation and large text | Command/Ctrl+1–5 for workspaces; +6 for help; Command/Ctrl plus/minus for zoom. |

## Not implemented yet

GPU utilization, fan RPM/control, battery health or charge limiting, Bluetooth
accessory batteries, camera/microphone activity, tray/menu-bar integration, app
icons and last-used dates, Sparkle/App Store update aggregation, one-click bulk
uninstall with related data, arbitrary-file Trash actions in Analyze, historical
lifetime cleanup totals, and the reference's planetary artwork are not implemented.

These are not hidden behind an upgrade or subscription. They need additional
platform-specific providers, review, and tests. Unsupported readings are not
presented as zero, and a system-settings shortcut is not called a completed action.

## Deliberate safety differences

Burrow does not automatically choose files, empty Trash, permanently delete on
failure, force-purge RAM, repair registry entries, force-quit processes, override
thermal controls, or silently grant itself elevated permissions. These behaviors
are not prerequisites for a useful storage utility.

Changing fan speeds, killing processes, and deleting shared app data have different
risks from a read-only scan. They should not be added just to match a screenshot.
