# Install, update, or remove Burrow

No terminal or GitHub account is needed.

## Download and install

Use the [published 0.2.0 preview](https://github.com/NobleSpartan6/burrow/releases/tag/v0.2.0-preview-2e38c4c).
Expand **Assets** on that page if the files are hidden. The 0.3 workspace branch
is development source; its new features are not in that older installer.

| Computer | File | Install |
| --- | --- | --- |
| Mac, Apple M-series | `Burrow-0.2.0-macOS-AppleSilicon.dmg` | Open it; drag Burrow into Applications. |
| Mac, Intel | `Burrow-0.2.0-macOS-Intel.dmg` | Open it; drag Burrow into Applications. |
| Windows, Intel/AMD 64-bit | `Burrow-0.2.0-Windows-x64-Setup.exe` | Open it; choose Install, then Finish. |

On Mac, **Apple menu → About This Mac** identifies your chip. Open Burrow from
Applications and eject the DMG afterward. On Windows, use the Start menu and your
normal account, not administrator mode. Source-code ZIPs are not installers.

Targets: macOS 12+ with Metal graphics; Windows 10/11 x64 with DirectX 12 graphics.
Windows ARM and 32-bit systems are not tested targets. Linux is a source/QA target,
not an installer download. Hosted tests do not cover every device or driver.

## A security warning appeared

Windows previews are unsigned; Mac previews are ad-hoc signed but not notarized.
Only download from this repository and follow your device policy. **Do not disable
Gatekeeper, SmartScreen, antivirus, or quarantine protections.** A managed device
may need its administrator's approval. Waiting for a signed release is valid.

The release's `SHA256SUMS.txt` can detect changed or damaged downloads. It is not
publisher verification or proof that software is safe.

## First launch

Start with a small local folder in **Analyze**. Running the scan only reads sizes;
it does not remove anything. In 0.3 development, removing a file requires a separate
**Review…** action and confirmation. [Using the workspaces →](TRY_BURROW.md)

## Update later

Finish active work, copy any session report you need, and quit Burrow. Install a
newer *published* release: run its Windows installer in the same folder, or drag
the Mac app into Applications and choose Replace. Uninstalling first is unnecessary.
Check the version in **? → About & help**. Windows portable copies are updated
separately and do not replace an existing Start-menu installation.

## Troubleshoot or uninstall

For a startup/graphics error, try a local desktop session and your manufacturer's
graphics updates. Do not bypass security settings. Missing sensors are not zero
readings. Missing update providers do not mean all apps are current; use the app's
own updater or store.

[Report a problem](https://github.com/NobleSpartan6/burrow/issues/new/choose) with
the version, OS, and steps. Remove private paths, app lists, and other personal data.

To uninstall, quit first. Use **Settings → Apps** on Windows, or move Burrow from
Applications to Trash on Mac. There is no background service. Local preferences
and cleanup totals are kept; reinstalling does not silently forget protected folders.
