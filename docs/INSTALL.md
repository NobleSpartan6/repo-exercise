# Install or update Burrow

You do not need a terminal or a GitHub account.

## 1. Download the right file

Open [Burrow releases](https://github.com/NobleSpartan6/burrow/releases) and choose
the newest published preview. Open **Assets** if the downloads are hidden.

| Your computer | Choose |
| --- | --- |
| Mac, Apple M-series chip | `macOS-AppleSilicon.dmg` |
| Mac, Intel processor | `macOS-Intel.dmg` |
| Windows, Intel or AMD 64-bit processor | `Windows-x64-Setup.exe` |

On a Mac, **Apple menu → About This Mac** shows the chip or processor.
The Source code ZIP and `source.zip` are for developers, not installation.

Burrow targets macOS 12 or later with Metal graphics, and Windows 10/11 x64
with DirectX 12 graphics. Windows ARM and 32-bit systems are not tested targets.
Hosted build checks do not cover every OS version, graphics driver, or device.

## 2. Install

**Mac:** open the DMG, drag Burrow into Applications, then open Burrow from
Applications. Eject the disk image afterward.

**Windows:** open the Setup EXE and choose Install, then Finish. Open Burrow from
the Start menu. Use your normal Windows account; administrator mode is not needed.
The portable ZIP is an alternative: extract it completely and open `burrow.exe`.

## Security warnings

These previews do not have verified publisher signing. Mac builds are ad-hoc
signed but not notarized; Windows builds are unsigned. Your computer may refuse
to open them. This is not proof that a file is safe or unsafe.

Only use the downloads from this repository. Follow your organization's device
policy. Do not disable Gatekeeper, SmartScreen, antivirus, or other protections,
and do not run a command that strips quarantine or bypasses a warning. A managed
computer may need approval from its administrator. Waiting for a signed release
is a valid choice.

Technical users can compare a download with the release's `SHA256SUMS.txt`.
Checksums detect a different or damaged download; they do not replace publisher
verification or a security review.

## 3. Try a read-only scan first

Open **Analyze**, choose a small local folder, and click **Analyze folder**.
Click a rectangle to look inside a subfolder. This workspace never deletes files.
See [First steps](TRY_BURROW.md) before using Clean or app removal.

## Update an existing installation

Wait for active work to finish. Copy any session log you need, then quit Burrow.

**Windows:** run the newer installer under the same account and keep the same
installation folder. There is no need to uninstall first.

**Mac:** open the newer DMG, drag Burrow into Applications, and choose Replace.

Open **? → About & help** and check the version. Portable Windows copies must be
updated separately; extracting a new ZIP does not replace an older shortcut.

## Something did not work

A startup error may mean your graphics driver or desktop session could not open
a native window. Try a local desktop session and the graphics updates provided
by your computer manufacturer. Do not change security settings to work around it.

An unavailable reading is not a zero reading. Some computers do not expose battery
or temperature information. App update checks need an already configured WinGet
or Homebrew installation; otherwise use the app's own updater or your app store.

[Report a problem](https://github.com/NobleSpartan6/burrow/issues/new/choose) with the
Burrow version, OS, and what happened. Remove personal paths and private details.

## Remove Burrow

On Windows, uninstall Burrow in **Settings → Apps**. On Mac, quit Burrow and move
it from Applications to Trash. A portable copy can be removed after quitting it.
There is no background service to stop. Saved cleanup preferences are kept so an
update or reinstall does not silently forget protected folders.
