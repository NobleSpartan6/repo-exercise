# Install and use Burrow — no coding required

Burrow is a free desktop app. You do not need an account, payment card, Terminal, PowerShell, Homebrew, Python, Rust, or Node.js to install a **prebuilt release**.

**This is preview software, not an independently audited or certified cleaner.** Keep a backup and begin with the read-only Disk explorer. Preview packages have no verified publisher signature. Choosing not to run an unsigned app is a reasonable, safer option.

## 1. Get the right file

Open https://github.com/NobleSpartan6/burrow/releases in your browser. Choose the newest Burrow preview and expand **Assets**.

If there are no Burrow installer files, no prebuilt app has been published yet. The **Source code** downloads are for developers and are not installers. The repository owner can check the **Actions → Burrow — build, test and package** run; a failed or unfinished run is not a successful release.

### Mac

Click the Apple symbol at the top left of your screen, then **About This Mac**.

- If it says **Chip: Apple M1, M2, M3, M4, M5**, or another Apple M-series chip, download the file ending in **macOS-AppleSilicon.dmg**.
- If it says **Processor: Intel**, download the file ending in **macOS-Intel.dmg**.

The package is configured for macOS 12 or newer. Each release's Actions run shows which macOS versions were actually used to build and test it. Older supported deployment versions are not all separately hardware-tested.

### Windows

For a normal Intel/AMD 64-bit Windows 11 computer, download the file ending in **Windows-x64-Setup.exe**. This version is not a native ARM64 or 32-bit Windows build. Windows 10 22H2 compatibility is intended but not separately certified.

To check, open **Settings → System → About** and look for **System type**. A Windows portable ZIP is available for users who prefer not to install; it is not the recommended first choice for a beginner.

## 2. Install

### Mac: drag, then open

1. Find the downloaded `.dmg` file in **Downloads** and double-click it.
2. In the window that opens, drag **Burrow** onto the **Applications** shortcut.
3. Open your **Applications** folder and double-click **Burrow**. Do not keep running it from the disk image.
4. Eject the Burrow disk image from Finder after copying the app.

There is no terminal command to paste. Burrow itself must not be run using `sudo`. Copying an app into a shared Applications folder may require your Mac's normal installation permission; a personal `~/Applications` folder is an alternative on restricted computers.

### Windows: normal installer

1. Find the downloaded `...Setup.exe` in **Downloads** and double-click it.
2. Read any security warning before proceeding; see the next section.
3. Choose **Install**, then **Finish**.
4. Open **Start**, type **Burrow**, and open it.

The installer uses your own user account's Programs folder and does not request administrator privileges. Do not choose **Run as administrator**. The optional desktop shortcut is off by default.

For the portable version, right-click the downloaded ZIP, choose **Extract All**, then open `burrow.exe` inside the extracted folder. Do not run the app from inside the ZIP preview.

## 3. Understand security warnings

### Mac: “developer cannot be verified” or “Apple cannot check…”

The preview is **not notarized** and has no trusted Developer ID signature. Its ad-hoc signature checks bundle integrity locally; it does **not** identify a trusted publisher or certify safety.

Only proceed after verifying that the download came from **NobleSpartan6/burrow** on GitHub and that you understand the preview risk. After attempting to open Burrow, macOS may offer **System Settings → Privacy & Security → Open Anyway** for that app. Read the confirmation and choose **Open** only when you trust this exact download.

**Do not** disable Gatekeeper, remove quarantine attributes with Terminal, or weaken your computer's global security settings. If the warning says the app **will damage your computer**, is **malware**, or is **damaged**, stop rather than overriding it. On a managed computer, ask your IT administrator; do not bypass organizational policy.

Apple's explanation: https://support.apple.com/102445

### Windows: “Windows protected your PC”

The preview has no Authenticode publisher signature. A reputation warning is possible and is not proof that the file is safe. Verify the repository and filename first. On personal devices, the warning may expose **More info → Run anyway**; use that per-file choice only when you trust the download and accept the unsigned-preview risk.

If the message reports a specific virus or malware finding, stop. **Do not** turn off Defender, SmartScreen, Smart App Control, antivirus protection, or company policies. Some managed PCs do not allow unsigned applications at all.

Microsoft's explanation: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation

Checksums are available as `SHA256SUMS.txt` for a technical helper to compare. A matching checksum only confirms the downloaded bytes match the published artifact; it does not establish a trusted publisher or prove absence of malware.

## 4. Start with a read-only look

Open **Disk explorer**, choose a local folder, and click **Analyze folder**. The app shows the largest files it encountered. This screen cannot delete anything. **Copy path** copies a file's location so you can find it yourself.

Sizes are logical file lengths, not guaranteed physical disk usage. Shared, sparse, compressed and hard-linked files can be different. A result marked **Partial** covers only what was scanned; unreadable folders, links and cloud placeholders may be omitted. Stop a long scan with **Cancel**. Network drives can take longer because cancellation waits for the current operating-system call.

## 5. Review a cleanup

1. Open **Clean up**. Choose **7**, **30**, or **90 days**, then click **Scan caches**. The scan removes nothing.
2. Read the paths and select only files you recognize. Nothing is preselected. **Select visible** selects the currently filtered rows; **Clear selection** clears everything, including hidden selections.
3. Close the apps whose caches you selected. Their next launch may be slower while caches rebuild, and some cached downloads may be needed offline.
4. Click **Review & move to Trash**. Check the total number of selected files, including selections hidden by a filter. Confirm that the apps are closed, then approve the move.
5. Read the result. **Session log → Copy full report** preserves original paths and per-file outcomes. Paste the report into a text document before quitting if you may need it later.

Burrow rechecks files before moving them. Changed, missing, linked, out-of-scope or inaccessible files are refused. A failed OS Trash operation is not silently retried using permanent deletion. After an error, inspect Trash before trying again because the OS can leave an operation's outcome uncertain.

**Cancel stops future work; it does not undo moves already completed.** A close request during a job requests cancellation and keeps the app open until the operation can stop.

## 6. Recover files or reclaim space

Burrow requests a move to the operating system's **Trash / Recycle Bin**. **This does not immediately free disk space.** The app never empties Trash.

To recover on Windows, open **Recycle Bin**, find the file, right-click it and choose **Restore** when available. On Mac, open **Trash** and use **Put Back** when available; otherwise move the file back using the original path you copied from the session log. Restore behavior depends on the OS and trash provider. Do not assume a recovery option is guaranteed for every filesystem or system policy.

Leave files in Trash while checking that your apps work normally. Manually emptying Trash later permanently deletes its contents and is your decision. Trash is not a substitute for a backup.

## Read the Overview

**Memory in use** shows the amount used, the total memory reported by your OS,
and the percentage used. This is not a diagnosis of memory pressure.

Drive bars represent **used space**. The amount available is printed separately,
not on top of the bar. A warning says **Low free space** at 10% available or less,
and **Very low free space** at 5% or less. Those warnings do not trigger cleanup.
A volume with an unusable capacity reading is labeled **Capacity unavailable**,
not shown as a full drive. Virtual volumes can report shared storage or quotas.

Scroll inside Overview to reach additional drives. CPU and memory are sampled
separately from drive information, so a slow drive query does not hold up those
readings. Older drive information is labeled when it is at least 30 seconds old.

## Update an existing installation — no uninstall needed

Only install a newer version after its installer appears under the GitHub
release's **Assets**. A source ZIP, draft release, or unfinished build is not
an installable update.

### Already installed on Windows

1. Open **About & help → Download a newer release**, or return to the GitHub
   Releases page in your browser. Download the newer **Windows-x64-Setup.exe**.
2. In Burrow, copy any cleanup session log you need to keep. Wait for active work
   to finish, then close Burrow normally.
3. Open the installer under the same Windows user account. Keep the existing
   install location, select **Install**, and then **Finish**. You do not need to
   uninstall 0.1.0 first or choose **Run as administrator**.
4. Open Burrow from Start. **About & help** and the footer show the installed
   version. For this update, look for **0.2.0**.

The installer keeps the same application identity and per-user location as
0.1.0. In-place upgrade behavior still needs release-specific testing on a real
Windows computer; do not infer that it was tested from the old installation report.

### Already installed on Mac

Quit Burrow after any active work finishes, download the correct newer DMG,
and drag Burrow into the same Applications folder. Choose **Replace** when
Finder asks, then open the new copy. Keep following the security-warning advice
above. Test the new app before emptying Trash.

For the portable Windows build, extract the newer portable ZIP to a new folder
and open that copy after closing the old one. A portable copy is not updated by
the normal installer.

There is no automatic updater, background service, or update check. The release
link opens only when you click it.

## Uninstall

To uninstall on Mac, quit Burrow and move **Burrow.app** from Applications to Trash. On Windows, use **Settings → Apps → Installed apps → Burrow → Uninstall**. For the Windows portable version, quit the app and delete its extracted folder. Uninstalling Burrow does not restore or empty previously trashed cache files.

## Troubleshooting

**Nothing eligible:** this means no old files were found in the narrow cache allowlist, not that the whole computer is empty. Try Disk explorer. Do not grant full-disk access simply to increase the cleanup number.

**Permission denied:** skip that item. Do not run Burrow as administrator or turn off protections. macOS may legitimately restrict an app's access to a chosen folder.

**Blank window or graphics error:** the app uses OpenGL. Remote desktop sessions, outdated graphics drivers, or restricted virtual machines may not expose a compatible graphics context. Report the OS version and the exact error; do not download drivers from an unknown website.

**Need help:** open https://github.com/NobleSpartan6/burrow/issues and include the app version, Mac/Windows version, and steps that reproduce the issue. Remove private usernames, folder paths and filenames from screenshots or copied logs before posting them publicly.

Native rendering in 0.2.0 uses Metal on Mac and DirectX 12 on Windows, with a low-power adapter preference. Linux remains a separate OpenGL QA target. No browser runtime is introduced. The pinned eframe diagnostic environment variable `EFRAME_SCREENSHOT_TO` captures the native window and exits when explicitly set by a maintainer; normal launches do not capture or save screenshots.
