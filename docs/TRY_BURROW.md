# Try Burrow — no terminal needed

Burrow is a free, independent, open-source preview for **Windows Intel/AMD 64-bit** and **Apple Silicon / Intel Macs**. It is not the official Mole app. Start with the read-only features.

## Download

Open [Burrow releases](https://github.com/NobleSpartan6/burrow/releases). Choose the newest published preview and expand **Assets**. On Windows, download the file ending in **Windows-x64-Setup.exe**. On a Mac, open **Apple menu → About This Mac**: an Apple M-series chip needs **macOS-AppleSilicon.dmg**; an Intel processor needs **macOS-Intel.dmg**. Do not download the source-code ZIP to install.

## Install or update

**Windows:** wait for work in an older Burrow to finish, then close it. Double-click the downloaded installer, keep the existing installation location, and choose **Install → Finish**. Open Burrow from Start. No developer tools, uninstall, or administrator mode are required. Portable ZIP copies are separate and must be replaced separately.

**Mac:** quit an older Burrow, open the downloaded DMG, and drag Burrow to **Applications**. Choose **Replace** for an update. Open Burrow from Applications, not from inside the disk image.

**Security notice:** Windows packages are unsigned. Mac packages are ad-hoc signed but are not notarized. Security software may warn or block installation. Do not disable antivirus, Gatekeeper, SmartScreen, or quarantine protections. Stop when your device or organization blocks it. A successful automated build is not security certification. Read [the full installation guide](INSTALL.md) for details.

## First five minutes

Open **Overview** to see CPU, memory, and drives. Open **Disk explorer**, choose a small ordinary local folder, and run a read-only scan. It cannot remove files. Try the navigation, resizing, and **Ctrl / Command + plus or minus** for text size. Check the version in **About & help**.

For cleanup, keep a backup and close affected apps first. Open **Clean up → Scan caches**. Nothing is selected by default. Review the full file paths, select only files you understand, and use **Review selection**. The separate confirmation must be acknowledged before any move to Trash / Recycle Bin. Skip cleanup entirely when unsure. Do not test with important, synchronized, network, or shared data.

Moving files to Trash does not immediately free disk space, and Trash is not a backup. Burrow never empties it. Stopping does not undo completed moves. Inspect Trash after an error before retrying. Rebuilding caches can require connectivity or temporarily slow other apps down.

## Help us make the next release stronger

[Report a problem](https://github.com/NobleSpartan6/burrow/issues/new?template=bug_report.yml) with the Burrow version, OS version, Mac chip or Windows graphics adapter, display scale, what you clicked, and what happened. A redacted screenshot helps. Do not post personal file paths, account details, or unredacted cleanup logs in a public issue. Maintainers should add a regression check for every confirmed defect before closing it.

This preview has automated checks, not a guarantee of no flaws on every device. Keep the previous installer available. To roll back, quit Burrow and reinstall the older version from the same releases page; no cleanup move is reversed by rolling back the app.
