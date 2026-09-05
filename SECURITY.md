# Safety boundaries and security policy

## Intended use

Burrow is preview, user-level software for reviewing old files in a deliberately narrow cache allowlist on a **non-adversarial local machine**. It is not an antivirus, an OS optimizer, a privileged administration agent, a secure erasure tool, or a filesystem sandbox. Do not run it elevated. Keep backups and close the relevant apps before cleanup.

## Destructive capability

The only production cleanup sink is `trash::delete`, invoked serially for explicitly selected preview candidates. There is no `remove_file`, recursive permanent-delete fallback, Trash-empty operation, downloaded command, arbitrary shell execution, registry editing or sudo path in the app. Build scripts and isolated test fixtures are a separate context.

The UI sends no arbitrary filesystem path to a deletion endpoint. Candidate fingerprints are private engine state, not deserialized UI-supplied metadata. The cleanup worker independently rediscovers the OS cache allowlist. Disk explorer cannot issue cleanup candidates.

Every selected candidate is revalidated immediately before the OS Trash request: current allowlist membership; absolute path and component-aware containment; no parent traversal; no symlinks in any ancestor; no Windows reparse points, junctions, offline or recall-on-access placeholders; a regular file rather than a directory; matching size, modification time and available creation time; and the original minimum age. Unix also checks inode/device identity and excludes multiply hard-linked files. Duplicate selections are processed once. Changed/missing files and errors are reported individually.

These guards reduce accidental scope expansion. They do not make cleanup intrinsically harmless: old caches may still be open, useful offline, expensive to regenerate or needed by a running app. The age check uses modification time, not last access time. macOS/Unix can allow a move while a process still has a file open. The user must close affected apps and review the selection.

## Residual risks: do not overstate the guarantees

**TOCTOU:** validation and the native path-based Trash request are separate operations. Another process with the same user privileges could swap a path or ancestor after validation. Inode checks on Unix do not make the subsequent path-based request atomic. Windows validation compares size and timestamps, not a stable NTFS file ID or held file handle; a deliberately replaced file with identical metadata may evade that fingerprint. This app is not hardened against a malicious or concurrently mutating same-user filesystem. An adversarial or privileged deployment requires handle-relative, platform-specific atomic operations and a separate security review.

**Filesystem traversal:** no-follow and same-filesystem traversal checks are best-effort snapshots. Permission errors, concurrent renames, mount changes and link races may make scan totals incomplete. Limits and cancellations are exposed as partial results. OS filesystem calls can block beyond the cooperative time budget. Root-discovery errors are surfaced; some inaccessible paths can be skipped.

**Trash semantics:** the app requests the OS Trash service and never substitutes permanent deletion itself. This does not guarantee recoverability under every OS setting, quota, volume, filesystem or policy. Native operation errors can leave outcomes uncertain. Inspect Trash before retrying, and use a backup for important data. Moving to Trash does not necessarily free any physical disk space. Burrow never empties it.

**Metrics:** disk explorer reports logical lengths, not uniquely allocated bytes. Compression, clones, sparse files and hard links affect physical storage. Volumes can share a backing disk or APFS container; do not add their capacities together as independent physical space.

## Privacy

No account, telemetry, analytics, remote scanning, update checker or background service is implemented. Scan paths, selections and reports are held in memory and are lost on exit. A user can explicitly copy a report to the clipboard or open a fixed documentation/source link in their browser. The clipboard, browser, crash reporting and OS signature checks have their own behavior outside Burrow.

## Verification and release status

Unit tests cover age limits, future and recent files, changed and missing candidates, allowlist rejection, duplicate selections, overlapping roots, cancellation, Trash failure without fallback, top-K bounded read-only analysis, parent traversal, Unix links/ancestor swaps/hard links and Windows junctions. Unit tests inject a mock Trash sink and do not prove native restoration behavior. GUI smoke tests exercise native window creation and navigation in Linux/Xvfb, not every end-to-end workflow on physical Mac/Windows devices.

Preview packages are not signed with a verified Windows publisher identity or Apple Developer ID and are not notarized. A green CI run proves the recorded jobs passed; it is not a security certification, malware verdict, or blanket compatibility guarantee. SHA-256 checksums detect changed artifact bytes, not malicious source or a compromised release account.

Before declaring a stable release, test installation, preview/selection/confirmation, cancellation, denied permissions, actual native Trash behavior and manual restoration on disposable data on physical Apple Silicon, Intel Mac and Windows hardware. Check accessibility, scaling, keyboard navigation, multiple profiles, cache rebuild behavior and performance measurements. Review transitive dependencies and generated notices. Add trusted platform signing and macOS notarization through securely managed credentials; do not paste private certificates into the repository.

## Reporting

Do not post private file paths, credentials, exploit details or personal data in a public issue. Use GitHub's private vulnerability reporting when it is enabled for this repository, or first ask the maintainer for a private reporting channel without disclosing the vulnerability. Ordinary non-sensitive bugs can use the issue tracker. No private reporting address is fabricated here.
