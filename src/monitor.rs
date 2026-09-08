use burrow::latest::Latest;
use eframe::egui;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};

#[derive(Clone, Default)]
pub struct Disk {
    pub name: String,
    pub mount: String,
    pub total: u64,
    pub available: u64,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub ready: bool,
    pub cpu: f32,
    pub used_memory: u64,
    pub total_memory: u64,
}

#[derive(Clone, Default)]
pub struct DriveSnapshot {
    pub ready: bool,
    pub disks: Vec<Disk>,
    pub sampled_at: Option<Instant>,
}

pub struct Monitor {
    pub system: Latest<Snapshot>,
    pub drives: Latest<DriveSnapshot>,
    stop: Arc<AtomicBool>,
    overview_visible: Arc<AtomicBool>,
}

/// Wait cooperatively without a busy loop. OS measurement calls themselves may block.
fn wait(stop: &AtomicBool, duration: Duration) -> bool {
    let until = Instant::now() + duration;
    while !stop.load(Ordering::Relaxed) {
        let remaining = until.saturating_duration_since(Instant::now());
        if remaining.is_zero() { return true; }
        std::thread::sleep(remaining.min(Duration::from_millis(100)));
    }
    false
}

impl Monitor {
    pub fn start(ctx: egui::Context) -> Result<Self, String> {
        let monitor = Self {
            system: Latest::default(), drives: Latest::default(),
            stop: Arc::new(AtomicBool::new(false)),
            overview_visible: Arc::new(AtomicBool::new(true)),
        };
        {
            let tx = monitor.system.clone();
            let stop = monitor.stop.clone();
            let visible = monitor.overview_visible.clone();
            let ctx = ctx.clone();
            std::thread::Builder::new().name("burrow-cpu-memory".into()).spawn(move || {
                // Do not enumerate processes, network activity, or temperatures.
                let mut system = System::new();
                let mut ready = false;
                while !stop.load(Ordering::Relaxed) {
                    system.refresh_cpu_usage();
                    system.refresh_memory();
                    tx.publish(Snapshot {
                        ready, cpu: system.global_cpu_usage(),
                        used_memory: system.used_memory(), total_memory: system.total_memory(),
                    });
                    if visible.load(Ordering::Relaxed) { ctx.request_repaint(); }
                    ready = true;
                    if !wait(&stop, Duration::from_secs(2)) { break; }
                }
            }).map_err(|e| e.to_string())?;
        }
        {
            let tx = monitor.drives.clone();
            let stop = monitor.stop.clone();
            let visible = monitor.overview_visible.clone();
            std::thread::Builder::new().name("burrow-drive-capacity".into()).spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    // A slow mapped/virtual volume must not block CPU/RAM samples.
                    // Exactly one drive worker: a slow OS call cannot spawn a backlog.
                    let mut disks: Vec<_> = Disks::new_with_refreshed_list().iter().map(|d| Disk {
                        name: d.name().to_string_lossy().into_owned(),
                        mount: d.mount_point().to_string_lossy().into_owned(),
                        total: d.total_space(), available: d.available_space(),
                    }).collect();
                    disks.sort_by(|a, b| a.mount.cmp(&b.mount));
                    if stop.load(Ordering::Relaxed) { break; }
                    tx.publish(DriveSnapshot { ready: true, disks, sampled_at: Some(Instant::now()) });
                    if visible.load(Ordering::Relaxed) { ctx.request_repaint(); }
                    if !wait(&stop, Duration::from_secs(10)) { break; }
                }
            }).map_err(|e| e.to_string())?;
        }
        // An early error drops this guard and tells any already-started worker to stop.
        Ok(monitor)
    }

    pub fn set_overview_visible(&self, visible: bool) {
        self.overview_visible.store(visible, Ordering::Relaxed);
    }
}

impl Drop for Monitor {
    fn drop(&mut self) { self.stop.store(true, Ordering::Relaxed); }
}
