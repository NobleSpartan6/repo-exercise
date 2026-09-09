# Install or update Burrow 0.2.0 — no coding required

Burrow is free. You do not need an account, a payment card, Terminal, PowerShell, Homebrew, Python or Rust to install a prebuilt release.

**This is preview software.** Windows downloads are unsigned. Mac downloads are ad-hoc signed but not notarized. Automated checks are not a guarantee of flawless operation or an independent security audit. Keep backups and start with read-only Disk explorer.

## Download the right installer

Open [Burrow releases](https://github.com/NobleSpartan6/burrow/releases), choose the newest 0.2.0 preview and expand **Assets**.

| Your computer | File to download |
|---|---|
| Windows with an Intel/AMD 64-bit processor | `Burrow-0.2.0-Windows-x64-Setup.exe` |
| Mac with an Apple M-series chip | `Burrow-0.2.0-macOS-AppleSilicon.dmg` |
| Mac with an Intel processor | `Burrow-0.2.0-macOS-Intel.dmg` |

On Mac, **Apple menu → About This Mac** shows Chip or Processor. On Windows, **Settings → System → About → System type** shows the processor type. This is not a native Windows ARM64 or 32-bit build. **Source code ZIPs are not installers.**

Mac packages target macOS 12 or newer. Windows 11 x64 is the intended desktop target; Windows 10 22H2 compatibility is intended but not separately certified. The linked release build records the actual hosted-runner versions tested, not every supported OS or physical device.

## Windows: install or update

1. Close Burrow normally after active work finishes. Copy any cleanup session log you need before quitting; it is not saved automatically.
2. Open the downloaded **Setup.exe** under your normal Windows account. Keep the existing installation location when updating.
3. Choose **Install**, then **Finish**, and open **Burrow** from Start.
4. Check **About & help** or the footer for **0.2.0**.

No uninstall or administrator mode is needed. The installer uses your own user account's Programs folder; do not choose **Run as administrator**. The optional desktop shortcut is off by default.

The separate portable ZIP is optional: choose **Extract All**, then open `burrow.exe` in the extracted folder, not inside the ZIP preview. Update portable copies by extracting the newer ZIP separately. The normal installer does not update a portable copy.

## Mac: install or update

1. Quit Burrow normally after active work finishes. Copy any cleanup session log you need first.
2. Open the downloaded **DMG** and drag **Burrow** onto **Applications**. Choose **Replace** when updating an existing copy.
3. Open Burrow from your **Applications** folder, then eject the disk image in Finder.
4. Check **About & help** or the footer for **0.2.0**.

Do not keep running from the disk image or use `sudo`. Copying into a shared Applications folder may require normal installation permission. A personal Applications folder is an alternative on restricted machines.

## Understand security warnings

**Mac:** this preview has no trusted Developer ID signature or notarization. Ad-hoc signing checks bundle integrity locally; it does not identify a trusted publisher. Verify the exact download came from **NobleSpartan6/burrow** on GitHub before proceeding. After an attempted launch, macOS may offer **System Settings → Privacy & Security → Open Anyway** for this app. Use that per-app choice only when you trust the download and accept the preview risk.

**Windows:** this preview has no Authenticode publisher signature. A SmartScreen reputation warning may offer **More info → Run anyway**. Use that per-file choice only when you trust this exact download and accept the unsigned-preview risk. A warning is not evidence that a file is safe.

**Do not disable Gatekeeper, Defender, SmartScreen, Smart App Control, antivirus or organizational policies. Do not remove quarantine attributes using Terminal.** Stop if a warning reports malware, says the app will damage your computer, or says it is damaged. On managed computers, ask your IT administrator. Choosing not to run unsigned software is reasonable.

Official explanations: [Apple](https://support.apple.com/102445) and [Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

`SHA256SUMS.txt` is included for a technical helper to check download integrity. A matching checksum does not establish a trusted publisher or prove that software is free of defects or malware.

## First use: look without deleting

Open **Disk explorer**, choose a local folder and click **Analyze folder**. This screen finds the largest 200 files encountered and cannot delete anything. **Copy path** copies a file's location.

Reported sizes are logical lengths, not guaranteed physical disk usage. Shared, sparse, compressed or hard-linked files can differ. **Partial** means only the scanned portion is represented; links, cloud placeholders or unreadable folders may be omitted. **Cancel** stops a long scan, but a network filesystem call may take time to return.

## Review old caches carefully

Open **Clean up**, choose **7**, **30** or **90 days**, then **Scan caches**. Scanning removes nothing and nothing is preselected. Review the paths and select only files you recognize. **Select visible** selects filtered rows; **Clear selection** also clears hidden selections.

Close the apps whose caches you selected, then choose **Review & move to Trash**. Review the full selection, including hidden rows, acknowledge that the apps are closed and confirm. Caches may be needed offline; rebuilding them can temporarily slow apps or require connectivity.

Burrow rechecks files before requesting a move. Changed, missing, linked, out-of-scope or inaccessible files are refused. Its own code does not retry failed Trash operations using permanent deletion. Read the result and use **Session log → Copy full report** to preserve original paths and outcomes before quitting. Inspect Trash after an error because an OS operation's outcome can be uncertain.

**Cancel stops future work; it does not undo completed moves.** A close request during a job requests cancellation and waits for the operation to stop. Do not grant extra privileges merely to increase the cleanup number.

## Recovery and disk space

Moving files to **Trash / Recycle Bin does not immediately free disk space**. Burrow never empties Trash. Keep backups; Trash is not a backup.

On Windows, open Recycle Bin and choose **Restore** for the file when available. On Mac, use **Put Back** when available or move the file back using the original path from your session log. Recovery depends on the OS, volume and policy; it is not guaranteed for every configuration. Leave files in Trash while checking your apps. Manually emptying Trash later permanently deletes its contents and is your decision.

## Read the Overview

Memory shows used, total and percentage used, not a diagnosis of memory pressure. Drive bars show **used space**, with free space labeled separately. Warnings begin at 10% available or less and become **Very low free space** at 5% or less; they never trigger cleanup. Unusable readings are labeled unavailable rather than shown as full drives. Virtual volumes can report shared storage or quotas.

Scroll to reach additional drives. CPU/RAM sampling is independent of drive queries; drive readings at least 30 seconds old are labeled. Use **Ctrl / Command + 1–4** to change pages and **Ctrl / Command + plus or minus** to adjust text scale.

## Troubleshooting and uninstalling

**No eligible files:** no old files were found in the narrow cache allowlist. That is not a whole-computer diagnosis. Try Disk explorer; do not grant full-disk access merely to find more caches.

**Permission denied:** skip the item. Never run elevated or turn off protections to bypass a restriction.

**Blank window or graphics error:** Mac uses Metal; Windows uses DirectX 12. Try a local desktop session and your manufacturer's graphics updates. Do not use untrusted driver downloads. Report the exact error and OS version.

**Uninstall:** quit Burrow, then move Burrow.app to Trash on Mac or use **Settings → Apps → Installed apps → Burrow → Uninstall** on Windows. For portable copies, delete the extracted app folder after quitting. Uninstalling does not restore or empty previously trashed files.

There is no automatic updater or background update check. Help/release links open the browser only when clicked. Normal launches do not capture screenshots; the explicit maintainer `--smoke-test` mode creates QA evidence only when requested.

[Report a problem](https://github.com/NobleSpartan6/burrow/issues) with the app version, OS, processor, graphics adapter and reproduction steps. Redact private paths, filenames and account information before posting screenshots or logs publicly.
