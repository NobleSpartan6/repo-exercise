# Performance measurement

Burrow is engineered to keep filesystem work off the GUI thread, not advertised with unmeasured numbers.

## Architecture and bounds

The native Rust/egui desktop app uses the host graphics stack rather than shipping Chromium. CPU/RAM and drive capacity have independent workers and one-value mailboxes. New measurements overwrite older unread measurements instead of queuing. CPU/RAM is sampled every two seconds; the drive worker waits ten seconds after each completed poll, so a slow volume does not spawn additional workers or delay CPU/RAM samples. Monitoring requests repaints only while Overview is selected; it continues taking bounded samples on other pages. The UI requests extra refreshes approximately every 150 ms only during a job. The OS/graphics driver can still affect idle power and memory.

Cleanup discovery traverses known roots with four workers maximum, no content reads and no symlink following. Candidate count is capped at 20,000. Cleanup moves are intentionally serial for understandable per-file outcomes; the native Trash service, especially when handling many individual files, may dominate cleanup time. This app does not claim that moving thousands of cache files to Trash is instant.

Disk explorer makes one traversal and maintains the largest 200 paths with a min-heap: O(n log 200) comparison work, O(200) retained largest-file records, plus the directory walk stack. The engine does not build an in-memory copy of the whole filesystem. Cleanup preview necessarily retains up to 20,000 fingerprints and paths.

Traversal caps are 500,000 visited entries, depth 128 and a cooperative 120-second budget per scan. Each cap can produce partial results. Filesystem calls can block; cancellation is not a hard real-time guarantee. Long paths and filesystem metadata errors also affect memory and throughput.

Virtualized result lists draw only visible rows. Filter indexes are rebuilt on filter or result changes rather than every frame. Selection uses a set and incrementally maintained byte total. Displayed sizes are binary KiB/MiB/GiB and logical file lengths, not guaranteed physical reclaimable space.

## Synthetic reproducible benchmark

From a locked release source bundle:

```sh
cargo run --release --locked --example scan_bench
```

This creates 10,000 one-KiB sparse fixture files in 100 temporary folders, then times only the metadata scan, verifies its result, and removes the fixture on scope exit. Creation is excluded. Metadata is likely warm in the filesystem cache. The test records file count, elapsed seconds and retained result count. It is **not** a cold-disk, native-Trash, user-cache or real-world OS comparison.

The Linux QA workflow runs this example and writes its output to the run summary. Do not copy that number into a Mac/Windows marketing claim.

## Measure real targets before making claims

Record app version, exact source commit and Cargo.lock; CPU, memory and graphics hardware; OS build; filesystem and drive model; display scaling; dataset composition; denied paths; warm/cold-cache conditions; and whether antivirus, indexing or cloud sync is active.

Measure cold launch to interactive window, steady-state idle CPU and process memory, scan throughput for both lots of small files and fewer large files, responsiveness while scrolling/cancelling, time spent in native Trash, and the effects of cache rebuilding on the other application. Use Activity Monitor and Instruments on macOS, and Task Manager/Windows Performance Recorder on Windows. Compare release builds, not debug builds.

Publish median and tail values across repeated runs, along with limits and errors. Keep GUI responsiveness, metadata scan speed, Trash speed and subsequent application performance separate. Clearing a cache can temporarily make an app slower, not faster.

## 0.1.1 regression checks

The core Rust tests include unknown/contradictory capacities, near-full drives,
exact 5%/10% warning boundaries, maximum u64 sizes, non-finite CPU values,
latest-sample replacement, bounded retention, and independent stream locks.
They are intended to run with `cargo test --locked --all-targets` in CI.
These are correctness checks, not measured performance results.

For a real-device check, open Overview while a virtual or mapped drive is slow.
Verify CPU/RAM readings continue independently. Switch to About and compare idle
rendering activity, then return to Overview. Also test 1060×760 and 860×620 windows,
increased UI scale, long drive names, and scrolling to the final drive.
A user report of the old version launching is not a benchmark or validation of
these new behaviors.

## 0.2 interface budget

No continuously animated decorations, blur, web view or remote assets. Static orbital geometry. System fonts are validated and loaded once, not distributed. Preview byte totals are cached on receipt. Both file lists are virtualized inside finite-height scroll regions. Scan progress repaints occur at about 7 Hz; CPU/RAM sampling at 0.5 Hz; monitoring does not request repaints on other pages. CI idle measurements are observations on a single machine, not Mac/Windows hardware guarantees.

Native rendering in 0.2.0 uses Metal on Mac and DirectX 12 on Windows, with a low-power adapter preference. Linux remains a separate OpenGL QA target. No browser runtime is introduced. Native QA uses an explicit `--smoke-test` launch with `BURROW_SMOKE_OUTPUT` pointing to an empty evidence directory. It captures the app's GPU surface using egui screenshot events, not the unsupported eframe screenshot environment variable. Normal launches never capture or save screenshots.
