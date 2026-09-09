# Performance

The goal is a responsive utility that does little work when it is not needed.
These are design limits, not speed claims about every computer.

## Interface

Rust and egui draw a native GPU surface. There is no embedded browser, remote
asset download, blur pass, or decorative animation loop. System fonts are read
locally and validated; their files are not included in releases.

Long file, app, startup, and process lists render only visible rows. Filtered row
indexes are rebuilt when data, filters, or sort order change—not for every draw.
Cleanup totals are cached. The folder map draws at most 48 tiles. CPU/RAM history
retains 60 samples. The floating monitor reads a small shared snapshot rather than
copying the process list, including when the main window is minimized.

## Background work

The interface does not run recursive scans. One task at a time handles scanning,
app discovery, OS tools, or reviewed changes. Cache discovery uses at most four
workers. A slow volume query cannot block the separate CPU/RAM sampling worker.
Process/network/temperature sampling and battery sampling are separate too.

CPU/RAM refresh every 2 seconds; drives every 10 seconds. Detailed readings refresh
every 3 seconds only while Status or the mini monitor is requested. Battery checks
run at most once per 30 seconds while those views are requested. The mini window
redraws every 2 seconds while open. Closing Burrow stops the workers and releases
an active screen-on request; no service is installed.

## Limits

| Work | Limit |
| --- | --- |
| Cache preview | 20,000 candidates |
| Metadata scan | 500,000 entries, depth 128, about 120 seconds between OS calls |
| Analyze results | Largest 200 files; 2,048 direct children plus an Other total |
| App/startup inventory | Up to 2,000 entries |
| Visible process dataset | Top 4,096 sampled processes |
| Mac removal review | 100,000 bundle metadata entries; 5-minute review expiry |
| OS-tool output | 8 MiB per output stream; bounded wait and explicit failure |
| Saved preferences | 64 KiB |

Filesystem calls can block inside the OS. Cancellation is cooperative, not a
promise to interrupt every stalled disk immediately. Partial scans and stale
readings are labelled. Stopping a command or cleanup does not undo prior changes.

## Measure rather than guess

```sh
cargo run --locked --release --example scan_bench
```

The benchmark creates a synthetic temporary tree and reports its metadata scan.
CI also records a brief Linux idle sample. Neither measures real Mac/Windows disk
performance, energy use, SSD wear, or end-to-end cleanup speed. Use representative
hardware and workloads before making those claims.
