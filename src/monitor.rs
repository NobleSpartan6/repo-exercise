use eframe::egui;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, TrySendError}};
use std::time::Duration;
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
    pub disks: Vec<Disk>,
}

pub struct Monitor {
    pub rx: Receiver<Snapshot>,
    stop: Arc<AtomicBool>,
}

impl Monitor {
    pub fn start(ctx: egui::Context) -> Result<Self, String> {
        let (tx, rx) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        std::thread::Builder::new().name("burrow-monitor".into()).spawn(move || {
            // Do not enumerate processes, network activity or temperatures.
            let mut system = System::new();
            let mut disks = Vec::new();
            let mut tick = 0_u64;
            while !stopped.load(Ordering::Relaxed) {
                system.refresh_cpu_usage();
                system.refresh_memory();
                if tick % 5 == 0 {
                    disks = Disks::new_with_refreshed_list().iter().map(|d| Disk {
                        name: d.name().to_string_lossy().into_owned(),
                        mount: d.mount_point().to_string_lossy().into_owned(),
                        total: d.total_space(), available: d.available_space(),
                    }).collect();
                }
                let sample = Snapshot {
                    ready: tick > 0, cpu: system.global_cpu_usage(),
                    used_memory: system.used_memory(), total_memory: system.total_memory(),
                    disks: disks.clone(),
                };
                match tx.try_send(sample) {
                    Ok(()) => ctx.request_repaint(),
                    Err(TrySendError::Disconnected(_)) => break,
                    Err(TrySendError::Full(_)) => {},
                }
                tick += 1;
                // Short sleeps make shutdown cooperative without a busy loop.
                for _ in 0..20 {
                    if stopped.load(Ordering::Relaxed) { return; }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }).map_err(|e| e.to_string())?;
        Ok(Self { rx, stop })
    }
}

impl Drop for Monitor {
    fn drop(&mut self) { self.stop.store(true, Ordering::Relaxed); }
}
