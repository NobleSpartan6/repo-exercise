use burrow::latest::Latest;
use eframe::egui;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use sysinfo::{Components, Disks, Networks, ProcessRefreshKind, ProcessesToUpdate, System};

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
    pub sampled_at: Option<Instant>,
}

#[derive(Clone, Default)]
pub struct DriveSnapshot {
    pub ready: bool,
    pub disks: Vec<Disk>,
    pub sampled_at: Option<Instant>,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessRow {
    pub pid: u32,
    pub started: u64,
    pub name: String,
    pub cpu: f32,
    pub memory: u64,
}
#[derive(Clone, Default)]
pub struct DetailSnapshot {
    pub ready: bool,
    pub sampled_at: Option<Instant>,
    pub uptime: u64,
    pub swap: u64,
    pub down: Option<f64>,
    pub up: Option<f64>,
    pub processes: Vec<ProcessRow>,
    pub process_count: usize,
    pub temperatures: Vec<(String, f32)>,
}
#[derive(Clone, Default)]
pub struct PowerSnapshot {
    pub ready: bool,
    pub percent: Option<f32>,
    pub state: String,
    pub sampled_at: Option<Instant>,
}

/// Small shared readings for secondary windows; no process lists are cloned here.
#[derive(Clone, Default)]
pub struct MiniSnapshot {
    pub system: Snapshot,
    pub sampled_at: Option<Instant>,
    pub down: Option<f64>,
    pub up: Option<f64>,
    pub detail_at: Option<Instant>,
    pub power: PowerSnapshot,
}
pub fn mini_id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("burrow-mini")
}

pub struct Monitor {
    pub system: Latest<Snapshot>,
    pub drives: Latest<DriveSnapshot>,
    pub details: Latest<DetailSnapshot>,
    pub power: Latest<PowerSnapshot>,
    pub mini: Arc<Mutex<MiniSnapshot>>,
    stop: Arc<AtomicBool>,
    overview_visible: Arc<AtomicBool>,
}

/// Wait cooperatively without a busy loop. OS measurement calls themselves may block.
fn wait(stop: &AtomicBool, duration: Duration) -> bool {
    let until = Instant::now() + duration;
    while !stop.load(Ordering::Relaxed) {
        let remaining = until.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return true;
        }
        std::thread::sleep(remaining.min(Duration::from_millis(100)));
    }
    false
}

impl Monitor {
    pub fn start(ctx: egui::Context) -> Result<Self, String> {
        let monitor = Self {
            system: Latest::default(),
            drives: Latest::default(),
            details: Latest::default(),
            power: Latest::default(),
            mini: Arc::new(Mutex::new(MiniSnapshot::default())),
            stop: Arc::new(AtomicBool::new(false)),
            overview_visible: Arc::new(AtomicBool::new(true)),
        };
        {
            let tx = monitor.system.clone();
            let mini = monitor.mini.clone();
            let stop = monitor.stop.clone();
            let visible = monitor.overview_visible.clone();
            let ctx = ctx.clone();
            std::thread::Builder::new()
                .name("burrow-cpu-memory".into())
                .spawn(move || {
                    // CPU/RAM stays independent of process, sensor and volume calls.
                    let mut system = System::new();
                    let mut ready = false;
                    while !stop.load(Ordering::Relaxed) {
                        system.refresh_cpu_usage();
                        system.refresh_memory();
                        let sample = Snapshot {
                            ready,
                            cpu: system.global_cpu_usage(),
                            used_memory: system.used_memory(),
                            total_memory: system.total_memory(),
                            sampled_at: Some(Instant::now()),
                        };
                        {
                            let mut value = mini.lock().unwrap_or_else(|e| e.into_inner());
                            value.system = sample.clone();
                            value.sampled_at = Some(Instant::now());
                        }
                        tx.publish(sample);
                        if visible.load(Ordering::Relaxed) {
                            ctx.request_repaint();
                        }
                        ready = true;
                        if !wait(&stop, Duration::from_secs(2)) {
                            break;
                        }
                    }
                })
                .map_err(|e| e.to_string())?;
        }
        {
            let tx = monitor.drives.clone();
            let stop = monitor.stop.clone();
            let visible = monitor.overview_visible.clone();
            let ctx = ctx.clone();
            std::thread::Builder::new()
                .name("burrow-drive-capacity".into())
                .spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        // A slow mapped/virtual volume must not block CPU/RAM samples.
                        // Exactly one drive worker: a slow OS call cannot spawn a backlog.
                        let mut disks: Vec<_> = Disks::new_with_refreshed_list()
                            .iter()
                            .map(|d| Disk {
                                name: d.name().to_string_lossy().into_owned(),
                                mount: d.mount_point().to_string_lossy().into_owned(),
                                total: d.total_space(),
                                available: d.available_space(),
                            })
                            .collect();
                        disks.sort_by(|a, b| a.mount.cmp(&b.mount));
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                        tx.publish(DriveSnapshot {
                            ready: true,
                            disks,
                            sampled_at: Some(Instant::now()),
                        });
                        if visible.load(Ordering::Relaxed) {
                            ctx.request_repaint();
                        }
                        if !wait(&stop, Duration::from_secs(10)) {
                            break;
                        }
                    }
                })
                .map_err(|e| e.to_string())?;
        }
        {
            let tx = monitor.details.clone();
            let stop = monitor.stop.clone();
            let mini = monitor.mini.clone();
            let visible = monitor.overview_visible.clone();
            let ctx = ctx.clone();
            std::thread::Builder::new()
                .name("burrow-status-details".into())
                .spawn(move || {
                    let mut system = System::new();
                    let mut networks = Networks::new();
                    let mut components = Components::new();
                    let mut previous = None;
                    let mut ready = false;
                    while !stop.load(Ordering::Relaxed) {
                        if !visible.load(Ordering::Relaxed) {
                            previous = None;
                            ready = false;
                            if !wait(&stop, Duration::from_millis(500)) {
                                break;
                            }
                            continue;
                        }
                        let now = Instant::now();
                        system.refresh_processes_specifics(
                            ProcessesToUpdate::All,
                            true,
                            ProcessRefreshKind::nothing().with_cpu().with_memory(),
                        );
                        system.refresh_memory();
                        networks.refresh(true);
                        components.refresh(true);
                        let elapsed =
                            previous.map(|p: Instant| now.duration_since(p).as_secs_f64());
                        previous = Some(now);
                        let received = networks
                            .iter()
                            .filter(|(n, _)| !n.starts_with("lo"))
                            .fold(0u64, |sum, (_, n)| sum.saturating_add(n.received()));
                        let sent = networks
                            .iter()
                            .filter(|(n, _)| !n.starts_with("lo"))
                            .fold(0u64, |sum, (_, n)| sum.saturating_add(n.transmitted()));
                        let mut processes: Vec<_> = system
                            .processes()
                            .iter()
                            .map(|(pid, p)| ProcessRow {
                                pid: pid.as_u32(),
                                started: p.start_time(),
                                name: p.name().to_string_lossy().chars().take(256).collect(),
                                cpu: p.cpu_usage(),
                                memory: p.memory(),
                            })
                            .collect();
                        let count = processes.len();
                        processes
                            .sort_by(|a, b| b.cpu.total_cmp(&a.cpu).then(b.memory.cmp(&a.memory)));
                        processes.truncate(4096);
                        let temperatures = components
                            .iter()
                            .filter_map(|c| {
                                c.temperature()
                                    .filter(|t| t.is_finite() && *t > -40.0 && *t < 200.0)
                                    .map(|t| (c.label().chars().take(120).collect(), t))
                            })
                            .take(24)
                            .collect();
                        let sample = DetailSnapshot {
                            ready,
                            sampled_at: Some(Instant::now()),
                            uptime: System::uptime(),
                            swap: system.used_swap(),
                            down: elapsed.filter(|t| *t > 0.0).map(|t| received as f64 / t),
                            up: elapsed.filter(|t| *t > 0.0).map(|t| sent as f64 / t),
                            processes,
                            process_count: count,
                            temperatures,
                        };
                        {
                            let mut value = mini.lock().unwrap_or_else(|e| e.into_inner());
                            value.down = sample.down;
                            value.up = sample.up;
                            value.detail_at = sample.sampled_at;
                        }
                        tx.publish(sample);
                        ready = true;
                        if visible.load(Ordering::Relaxed) {
                            ctx.request_repaint();
                        }
                        if !wait(&stop, Duration::from_secs(3)) {
                            break;
                        }
                    }
                })
                .map_err(|e| e.to_string())?;
        }
        {
            let tx = monitor.power.clone();
            let stop = monitor.stop.clone();
            let visible = monitor.overview_visible.clone();
            let mini = monitor.mini.clone();
            std::thread::Builder::new()
                .name("burrow-power-status".into())
                .spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        if !visible.load(Ordering::Relaxed) {
                            if !wait(&stop, Duration::from_millis(500)) {
                                break;
                            }
                            continue;
                        }
                        let control = burrow::engine::Control {
                            cancel: stop.clone(),
                            ..Default::default()
                        };
                        let mut sample = read_power(&control);
                        sample.ready = true;
                        sample.sampled_at = Some(Instant::now());
                        {
                            mini.lock().unwrap_or_else(|e| e.into_inner()).power = sample.clone();
                        }
                        tx.publish(sample);
                        if visible.load(Ordering::Relaxed) {
                            ctx.request_repaint();
                        }
                        if !wait(&stop, Duration::from_secs(30)) {
                            break;
                        }
                    }
                })
                .map_err(|e| e.to_string())?;
        }
        // An early error drops this guard and tells any already-started worker to stop.
        Ok(monitor)
    }

    pub fn set_overview_visible(&self, visible: bool) {
        self.overview_visible.store(visible, Ordering::Relaxed);
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn read_power(control: &burrow::engine::Control) -> PowerSnapshot {
    #[cfg(target_os = "macos")]
    {
        let result = burrow::command::run(
            &burrow::command::system_tool("pmset"),
            ["-g", "batt"],
            control,
            Duration::from_secs(5),
        );
        match result {
            Ok(s) => parse_mac_power(&s),
            Err(e) => PowerSnapshot {
                state: format!("Battery reading unavailable: {e}"),
                ..Default::default()
            },
        }
    }
    #[cfg(windows)]
    {
        let result = burrow::command::powershell(
            "$b=Get-CimInstance Win32_Battery | Select-Object -First 1; if($null -eq $b){'null'}else{[PSCustomObject]@{percent=$b.EstimatedChargeRemaining;status=$b.BatteryStatus}|ConvertTo-Json -Compress}",
            control,
        );
        match result {
            Ok(s) => parse_windows_power(&s),
            Err(e) => PowerSnapshot {
                state: format!("Battery reading unavailable: {e}"),
                ..Default::default()
            },
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = control;
        PowerSnapshot {
            state: "Battery provider is available on Mac and Windows.".into(),
            ..Default::default()
        }
    }
}
#[cfg(any(target_os = "macos", test))]
fn parse_mac_power(s: &str) -> PowerSnapshot {
    let percent = s
        .split_whitespace()
        .find_map(|w| w.split_once('%').and_then(|(n, _)| n.parse::<f32>().ok()))
        .filter(|p| (0.0..=100.0).contains(p));
    PowerSnapshot {
        percent,
        state: if percent.is_none() {
            "No battery reported"
        } else if s.contains("discharging") {
            "On battery"
        } else if s.contains("not charging") {
            "Plugged in · not charging"
        } else if s.contains("charging") {
            "Charging"
        } else {
            "External power"
        }
        .into(),
        ..Default::default()
    }
}
#[cfg(any(windows, test))]
fn parse_windows_power(s: &str) -> PowerSnapshot {
    let v: serde_json::Value = serde_json::from_str(s).unwrap_or_default();
    let percent = v["percent"]
        .as_f64()
        .filter(|p| p.is_finite() && (0.0..=100.0).contains(p))
        .map(|p| p as f32);
    let state = match v["status"].as_u64() {
        Some(1) => "On battery",
        Some(2) => "External power",
        Some(3) => "Fully charged",
        Some(6..=9) => "Charging",
        _ => "Power state unavailable",
    };
    PowerSnapshot {
        percent,
        state: if v.is_null() {
            "No battery reported"
        } else {
            state
        }
        .into(),
        ..Default::default()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plugged_in_is_not_always_charging() {
        assert_eq!(
            parse_mac_power("-InternalBattery-0 80%; not charging; 0:00 remaining").state,
            "Plugged in · not charging"
        );
    }
    #[test]
    fn missing_battery_is_not_zero_percent() {
        assert!(parse_windows_power("null").percent.is_none());
        assert!(
            parse_mac_power("Now drawing from 'AC Power'")
                .percent
                .is_none()
        );
    }
    #[test]
    fn battery_parser_validates_and_labels() {
        let p = parse_mac_power("-InternalBattery-0 73%; discharging; 3:20 remaining");
        assert_eq!(p.percent, Some(73.0));
        assert_eq!(p.state, "On battery");
        assert_eq!(
            parse_windows_power(r#"{"percent":60,"status":2}"#).percent,
            Some(60.0)
        );
        assert!(parse_windows_power(r#"{"percent":255}"#).percent.is_none());
    }
}
