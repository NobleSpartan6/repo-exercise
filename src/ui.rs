use burrow::{engine::{self, Analysis, Candidate, Cleanup, Control, Preview}, human_bytes, platform};
use crate::monitor::{DriveSnapshot, Monitor, Snapshot};
use burrow::{VERSION, metrics::{DiskCapacity, SpaceLevel, Usage, cpu_fraction}};
use eframe::egui::{self, RichText};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{atomic::Ordering, mpsc::{self, Receiver, Sender}};
use std::time::{Duration, Instant};
use crate::design::{self, BG, PANEL, LINE, MUTED, ACCENT, AMBER, DANGER};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page { Overview, Cleanup, Explorer, About }
impl Page {
    fn title(self) -> &'static str {
        match self { Self::Overview => "Overview", Self::Cleanup => "Clean up", Self::Explorer => "Disk explorer", Self::About => "About & help" }
    }
}

enum Task { Preview(u32), Analyze(PathBuf), Clean(Vec<Candidate>) }
enum Event { Preview(Preview), Analyzed(Analysis), Cleaned(Cleanup), Error(String) }

pub struct Burrow {
    page: Page,
    monitor: Option<Monitor>,
    sample: Snapshot,
    drive_sample: DriveSnapshot,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    control: Control,
    busy: Option<&'static str>,
    preview: Option<Preview>,
    analysis: Option<Analysis>,
    report: Option<Cleanup>,
    selected: HashSet<usize>,
    selected_bytes: u64,
    preview_bytes: u64,
    visible: Vec<usize>,
    filter: String,
    days: u32,
    folder: Option<PathBuf>,
    message: String,
    confirm: bool,
    closed_apps: bool,
    show_log: bool,
    smoke: Option<(usize, Instant)>,
}

impl Burrow {
    pub fn new(cc: &eframe::CreationContext<'_>, smoke: bool) -> Self {
        Self::with_context(&cc.egui_ctx, !smoke, smoke)
    }
    fn with_context(ctx: &egui::Context, monitoring: bool, smoke: bool) -> Self {
        design::configure(ctx);
        let (tx, rx) = mpsc::channel();
        let (monitor, message) = match monitoring.then(|| Monitor::start(ctx.clone())) {
            Some(Ok(m)) => (Some(m), String::new()),
            Some(Err(e)) => (None, format!("System monitor unavailable: {e}")),
            None => (None, String::new()),
        };
        Self {
            page: Page::Overview, monitor, sample: Snapshot::default(), drive_sample: DriveSnapshot::default(), tx, rx,
            control: Control::default(), busy: None, preview: None, analysis: None,
            report: None, selected: HashSet::new(), selected_bytes: 0, preview_bytes: 0, visible: Vec::new(),
            filter: String::new(), days: 7, folder: None, message,
            confirm: false, closed_apps: false, show_log: false,
            smoke: smoke.then(|| (0, Instant::now())),
        }
    }
    fn navigate(&mut self, page: Page, ctx: &egui::Context) {
        self.page = page;
        if let Some(monitor) = &self.monitor { monitor.set_overview_visible(page == Page::Overview); }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("Burrow — {}", page.title())));
    }
    fn start(&mut self, task: Task, ctx: &egui::Context) {
        if self.busy.is_some() { return; }
        self.message.clear(); self.control = Control::default();
        self.busy = Some(match &task {
            Task::Preview(_) => "Scanning known caches", Task::Analyze(_) => "Reading folder sizes",
            Task::Clean(_) => "Moving selected files to Trash",
        });
        if matches!(&task, Task::Preview(_)) {
            self.preview = None; self.selected.clear(); self.selected_bytes = 0; self.visible.clear();
        }
        if matches!(&task, Task::Analyze(_)) { self.analysis = None; }
        let control = self.control.clone(); let tx = self.tx.clone(); let repaint = ctx.clone();
        let spawned = std::thread::Builder::new().name("burrow-job".into()).spawn(move || {
            let event = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match task {
                Task::Preview(days) => engine::preview(&platform::cache_roots(), days, &control).map(Event::Preview).unwrap_or_else(Event::Error),
                Task::Analyze(path) => engine::analyze(&path, &control).map(Event::Analyzed).unwrap_or_else(Event::Error),
                Task::Clean(files) => Event::Cleaned(engine::clean_selected(&files, &control)),
            })).unwrap_or_else(|_| Event::Error("The worker stopped unexpectedly. Inspect Trash before retrying a cleanup.".into()));
            let _ = tx.send(event); repaint.request_repaint();
        });
        if let Err(error) = spawned { self.busy = None; self.message = format!("Could not start the task: {error}"); }
    }
    fn receive(&mut self) {
        if let Some(monitor) = &self.monitor {
            if let Some(sample) = monitor.system.take() { self.sample = sample; }
            if let Some(sample) = monitor.drives.take() { self.drive_sample = sample; }
        }
        while let Ok(event) = self.rx.try_recv() {
            self.busy = None;
            match event {
                Event::Preview(result) => {
                    self.preview_bytes = result.files.iter().map(|f| f.bytes).fold(0, u64::saturating_add);
                    self.preview = Some(result); self.refilter();
                },
                Event::Analyzed(result) => self.analysis = Some(result),
                Event::Cleaned(result) => {
                    self.message = format!("{} files moved to Trash; {} not confirmed moved. {}", result.moved, result.refused,
                        if result.cancelled { "Stopped early. Completed moves were not undone." }
                        else { "Space is not reclaimed until you empty Trash yourself." });
                    self.report = Some(result); self.preview = None; self.visible.clear();
                    self.selected.clear(); self.selected_bytes = 0;
                },
                Event::Error(error) => self.message = error,
            }
        }
    }
    fn refilter(&mut self) {
        let query = self.filter.to_lowercase();
        self.visible = self.preview.as_ref().map(|p| p.files.iter().enumerate()
            .filter(|(_, f)| query.is_empty() || f.path.to_string_lossy().to_lowercase().contains(&query) || f.category.to_lowercase().contains(&query))
            .map(|(i, _)| i).collect()).unwrap_or_default();
    }
    fn overview(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical().id_salt("overview").auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0); design::orbit(ui, 70.0); ui.add_space(8.0);
                design::title(ui, "Room to breathe.", 36.0);
                design::muted(ui, "Less clutter. A clearer view of your computer.");
                ui.add_space(9.0);
                centered_actions(ui, 334.0, |ui| {
                    if primary(ui, "Review old caches", self.busy.is_none()).clicked() { self.navigate(Page::Cleanup, ctx); }
                    if secondary(ui, "Find large files", true).clicked() { self.navigate(Page::Explorer, ctx); }
                });
                ui.add_space(6.0);
                ui.label(RichText::new("Nothing is selected or removed automatically.").size(12.0).color(MUTED));
            });
            ui.add_space(26.0);
            let cpu = self.sample.ready.then(|| cpu_fraction(self.sample.cpu)).flatten();
            let memory = Usage::new(self.sample.used_memory, self.sample.total_memory);
            let cpu_value = cpu.map(|n| format!("{:.0}%", n * 100.0)).unwrap_or_else(|| "—".into());
            let memory_value = memory.map(|_| human_bytes(self.sample.used_memory)).unwrap_or_else(|| "—".into());
            let memory_detail = memory.map(|n| format!("of {}  ·  {:.0}% used", human_bytes(self.sample.total_memory), n.percent())).unwrap_or_else(|| "Waiting for an OS reading".into());
            if ui.available_width() >= 570.0 {
                ui.columns(2, |cols| {
                    design::metric(&mut cols[0], "CPU in use", &cpu_value, "Overall processor usage", cpu);
                    design::metric(&mut cols[1], "Memory in use", &memory_value, &memory_detail, memory.map(Usage::fraction));
                });
            } else {
                design::metric(ui, "CPU in use", &cpu_value, "Overall processor usage", cpu);
                design::metric(ui, "Memory in use", &memory_value, &memory_detail, memory.map(Usage::fraction));
            }
            ui.add_space(22.0);
            ui.horizontal(|ui| {
                design::title(ui, "Your drives", 19.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new("Bars show used space").size(12.0).color(MUTED)); });
            });
            ui.add_space(4.0);
            for disk in &self.drive_sample.disks {
                design::card().inner_margin(16).show(ui, |ui| {
                    ui.set_min_width((ui.available_width() - 1.0).max(0.0));
                    ui.horizontal(|ui| {
                        let name = if disk.name.is_empty() { &disk.mount } else { &disk.name };
                        ui.add(egui::Label::new(RichText::new(name).font(design::heading(14.0))).truncate()).on_hover_text(name);
                        if !disk.name.is_empty() { ui.add(egui::Label::new(RichText::new(&disk.mount).size(12.0).color(MUTED)).truncate()).on_hover_text(&disk.mount); }
                    });
                    if let Some(capacity) = DiskCapacity::new(disk.total, disk.available) {
                        let (color, warning) = match capacity.space_level() {
                            SpaceLevel::Normal => (ACCENT, None), SpaceLevel::Low => (AMBER, Some("Low free space")), SpaceLevel::VeryLow => (DANGER, Some("Very low free space")),
                        };
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(format!("{} free of {}", human_bytes(disk.available), human_bytes(disk.total))).size(12.0).color(MUTED));
                            ui.label(RichText::new(format!("{:.0}% used", capacity.used_percent())).size(12.0).color(color));
                            if let Some(warning) = warning { ui.colored_label(color, warning); }
                        });
                        design::bar(ui, capacity.used_fraction(), color);
                    } else { design::muted(ui, "Capacity unavailable — waiting for a usable OS reading."); }
                });
            }
            if self.drive_sample.disks.is_empty() {
                design::card().show(ui, |ui| design::muted(ui, if self.drive_sample.ready { "No readable drives reported by the operating system." }
                    else { "Reading drive information… CPU and memory are sampled separately." }));
            }
            if let Some(at) = self.drive_sample.sampled_at && at.elapsed().as_secs() >= 30 {
                ui.colored_label(AMBER, format!("Drive reading is {}s old. A volume may be slow to respond.", at.elapsed().as_secs()));
            }
            ui.add_space(6.0);
            ui.label(RichText::new("CPU / memory: every 2s. Drives: every 10s. Volumes can share physical storage.").size(12.0).color(MUTED));
        });
    }
    fn age_control(&mut self, ui: &mut egui::Ui) {
        let previous = self.days;
        ui.horizontal_wrapped(|ui| {
            design::muted(ui, "Only files older than");
            ui.add_enabled_ui(self.busy.is_none(), |ui| {
                egui::ComboBox::from_id_salt("minimum-age").selected_text(format!("{} days", self.days)).show_ui(ui, |ui| {
                    for days in [7, 30, 90] { ui.selectable_value(&mut self.days, days, format!("{days} days")); }
                });
            });
        });
        if previous != self.days { self.preview = None; self.selected.clear(); self.visible.clear(); self.selected_bytes = 0; }
    }
    fn cleanup(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.preview.is_none() {
            egui::ScrollArea::vertical().id_salt("cleanup-start").show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(26.0); design::orbit(ui, 118.0); ui.add_space(18.0);
                    design::title(ui, "A scan is just a look.", 32.0);
                    design::muted(ui, "Review old caches. Keep what matters.");
                    ui.add_space(15.0); centered_actions(ui, 252.0, |ui| self.age_control(ui)); ui.add_space(5.0);
                    if primary(ui, "Scan caches", self.busy.is_none()).clicked() { self.start(Task::Preview(self.days), ctx); }
                    ui.add_space(10.0);
                    ui.label(RichText::new("Personal folders and system files are not included.").size(12.0).color(MUTED));
                });
                ui.add_space(30.0);
                design::card().show(ui, |ui| {
                    ui.set_min_width((ui.available_width()-1.0).max(0.0));
                    design::title(ui, "You choose what goes.", 16.0);
                    design::muted(ui, "Scan first, review the paths, then select individual files. A separate confirmation is always required.");
                    egui::CollapsingHeader::new("Which caches are checked?").show(ui, |ui| {
                        for root in platform::cache_roots() {
                            ui.label(RichText::new(root.label).font(design::heading(13.0)));
                            ui.add(egui::Label::new(RichText::new(root.path.to_string_lossy()).size(12.0).color(MUTED)).wrap());
                        }
                        if !cfg!(any(target_os="windows", target_os="macos")) { design::muted(ui, "This Linux QA build has no cleanup allowlist."); }
                    });
                });
                self.last_report(ui);
            });
            return;
        }
        let height = (ui.available_height() - 340.0).clamp(140.0, 500.0);
        egui::ScrollArea::vertical().id_salt("cleanup-page").show(ui, |ui| self.cleanup_results(ui,ctx,height));
    }
    fn cleanup_results(&mut self, ui:&mut egui::Ui, ctx:&egui::Context, height:f32) {
        page_heading(ui, "Clean up", "Only older files in known caches. Nothing selected by default.");
        ui.horizontal_wrapped(|ui| {
            self.age_control(ui);
            if secondary(ui, "Scan again", self.busy.is_none()).clicked() { self.start(Task::Preview(self.days), ctx); }
        });
        let Some(preview) = &self.preview else { return; };
        let total = self.preview_bytes;
        ui.add_space(10.0); design::title(ui, &format!("{} to review", human_bytes(total)), 27.0);
        design::muted(ui, format!("{} eligible files  ·  {:.2}s  ·  {} entries visited  ·  {} excluded/unreadable", preview.files.len(), preview.elapsed.as_secs_f64(), preview.visited, preview.skipped));
        if preview.partial { ui.colored_label(AMBER, "Partial scan — stopped or reached a limit. These are not full-folder totals."); }
        for warning in preview.warnings.iter().take(2) { ui.colored_label(AMBER, warning); }
        if preview.files.is_empty() {
            ui.add_space(24.0); design::title(ui, "Nothing eligible in the caches checked.", 22.0);
            design::muted(ui, "This is not a whole-computer scan. Disk explorer can inspect a folder of your choice.");
            if secondary(ui, "Open disk explorer", true).clicked() { self.navigate(Page::Explorer, ctx); }
            self.last_report(ui); return;
        }
        ui.add_space(7.0);
        let filter = ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Filter by cache or file path…").desired_width(f32::INFINITY).margin(egui::vec2(12.0, 10.0)));
        design::record(&filter, "filter"); if filter.changed() { self.refilter(); }
        ui.horizontal_wrapped(|ui| {
            if secondary(ui, "Select visible", self.busy.is_none()).clicked() && let Some(preview) = &self.preview {
                for i in &self.visible { if self.selected.insert(*i) { self.selected_bytes = self.selected_bytes.saturating_add(preview.files[*i].bytes); } }
            }
            if secondary(ui, "Clear selection", !self.selected.is_empty()).clicked() { self.selected.clear(); self.selected_bytes = 0; }
            if primary(ui, "Review selection…", !self.selected.is_empty() && self.busy.is_none()).clicked() { self.closed_apps = false; self.confirm = true; }
        });
        ui.label(RichText::new(format!("{} selected · {} · includes selections hidden by the filter", self.selected.len(), human_bytes(self.selected_bytes))).size(12.0).color(MUTED));
        ui.separator();
        if self.visible.is_empty() { design::muted(ui, "No files match this filter. Hidden selections are still selected."); }
        if let Some(preview) = &self.preview {
            egui::ScrollArea::vertical().id_salt("cleanup-results").auto_shrink([false,false]).max_height(height).show_rows(ui, 50.0, self.visible.len(), |ui, range| {
                for row in range {
                    let i = self.visible[row]; let file = &preview.files[i];
                    ui.horizontal(|ui| {
                        ui.set_height(50.0); let mut checked = self.selected.contains(&i);
                        let response = ui.checkbox(&mut checked, "").on_hover_text("Select this file");
                        if response.changed() {
                            if checked { self.selected.insert(i); self.selected_bytes = self.selected_bytes.saturating_add(file.bytes); }
                            else { self.selected.remove(&i); self.selected_bytes = self.selected_bytes.saturating_sub(file.bytes); }
                        }
                        let width = (ui.available_width() - 104.0).max(40.0);
                        ui.allocate_ui(egui::vec2(width, 44.0), |ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            let name = file.path.file_name().unwrap_or_default().to_string_lossy();
                            ui.add(egui::Label::new(RichText::new(name).font(design::heading(13.0))).truncate());
                            ui.add(egui::Label::new(RichText::new(file.path.to_string_lossy()).size(11.0).color(MUTED)).truncate()).on_hover_text(format!("{}\n{}", file.category, file.path.display()));
                        });
                        ui.label(RichText::new(human_bytes(file.bytes)).size(13.0).color(ACCENT));
                    });
                }
            });
        }
    }
    fn last_report(&mut self, ui: &mut egui::Ui) {
        if let Some(report) = &self.report {
            ui.add_space(12.0);
            design::muted(ui, format!("Last cleanup: {} files / {} moved to Trash", report.moved, human_bytes(report.moved_bytes)));
            if secondary(ui, "Session log", true).clicked() { self.show_log = true; }
        }
    }
    fn choose_folder(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().set_title("Choose a folder to inspect").pick_folder() {
            if self.folder.as_ref() != Some(&folder) { self.analysis = None; }
            self.folder = Some(folder);
        }
    }
    fn explorer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.analysis.is_none() {
            egui::ScrollArea::vertical().id_salt("explorer-start").show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(26.0); design::orbit(ui, 118.0); ui.add_space(18.0);
                    design::title(ui, "Find the big things first.", 32.0);
                    design::muted(ui, "A clearer picture of where your space goes."); ui.add_space(20.0);
                    centered_actions(ui, 320.0, |ui| {
                        if secondary(ui, "Choose folder…", self.busy.is_none()).clicked() { self.choose_folder(); }
                        if primary(ui, "Analyze folder", self.folder.is_some() && self.busy.is_none()).clicked() && let Some(folder) = &self.folder { self.start(Task::Analyze(folder.clone()), ctx); }
                    });
                    if let Some(folder) = &self.folder { ui.add(egui::Label::new(RichText::new(folder.to_string_lossy()).size(12.0).color(MUTED)).truncate()).on_hover_text(folder.display().to_string()); }
                    ui.add_space(10.0);
                    ui.label(RichText::new("Read-only. No delete controls on this screen.").size(12.0).color(ACCENT));
                });
                ui.add_space(30.0);
                design::card().show(ui, |ui| {
                    ui.set_min_width((ui.available_width()-1.0).max(0.0));
                    design::title(ui, "Size, not contents.", 16.0);
                    design::muted(ui, "Burrow reads file metadata and keeps only the largest 200 results. Your files stay exactly where they are.");
                    ui.label(RichText::new("Links and cloud placeholders are excluded. Network drives may respond slowly.").size(12.0).color(MUTED));
                });
            });
            return;
        }
        let height = (ui.available_height() - 220.0).clamp(140.0, 500.0);
        egui::ScrollArea::vertical().id_salt("explorer-page").show(ui, |ui| self.explorer_results(ui,ctx,height));
    }
    fn explorer_results(&mut self, ui:&mut egui::Ui, ctx:&egui::Context, height:f32) {
        page_heading(ui, "Disk explorer", "Largest files first. Inspect freely; this screen never deletes.");
        ui.horizontal_wrapped(|ui| {
            if secondary(ui, "Choose folder…", self.busy.is_none()).clicked() { self.choose_folder(); }
            if primary(ui, "Analyze again", self.folder.is_some() && self.busy.is_none()).clicked() && let Some(folder) = &self.folder { self.start(Task::Analyze(folder.clone()), ctx); }
        });
        let Some(result) = &self.analysis else { return; };
        ui.add_space(14.0);
        design::title(ui, &format!("{} across {} files", human_bytes(result.total_bytes), result.files), 25.0);
        ui.add(egui::Label::new(RichText::new(result.root.to_string_lossy()).color(MUTED)).truncate()).on_hover_text(result.root.display().to_string());
        ui.label(RichText::new(format!("{:.2}s · {} excluded/unreadable · logical sizes, not allocated space", result.elapsed.as_secs_f64(), result.skipped)).size(12.0).color(MUTED));
        if result.partial { ui.colored_label(AMBER, "Partial result — stopped or reached a limit. Totals cover visited files only."); }
        ui.add_space(5.0); ui.separator();
        if result.top.is_empty() { design::muted(ui, "No readable files found in the selected folder."); }
        egui::ScrollArea::vertical().id_salt("largest-files").auto_shrink([false,false]).max_height(height).show_rows(ui, 55.0, result.top.len(), |ui, range| {
            for i in range {
                let file = &result.top[i];
                ui.horizontal(|ui| {
                    ui.set_height(55.0);
                    ui.add_sized([29.0, 36.0], egui::Label::new(RichText::new(format!("{:02}", i+1)).size(12.0).color(MUTED)));
                    let width = (ui.available_width() - 190.0).max(40.0);
                    ui.allocate_ui(egui::vec2(width, 48.0), |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.add(egui::Label::new(RichText::new(file.path.file_name().unwrap_or_default().to_string_lossy()).font(design::heading(13.0))).truncate());
                        ui.add(egui::Label::new(RichText::new(file.path.to_string_lossy()).size(11.0).color(MUTED)).truncate()).on_hover_text(file.path.display().to_string());
                    });
                    ui.add_sized([84.0, 36.0], egui::Label::new(RichText::new(human_bytes(file.bytes)).color(ACCENT).size(13.0)));
                    if secondary(ui, "Copy path", true).clicked() { ctx.copy_text(file.path.to_string_lossy().into_owned()); }
                });
            }
        });
    }
    fn about(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().id_salt("about").show(ui, |ui| {
            page_heading(ui, "Small app. Clear boundaries.", &format!("Burrow {VERSION} · Preview · Free and open source (MIT)"));
            design::card().show(ui, |ui| {
                ui.set_min_width((ui.available_width()-1.0).max(0.0));
                design::title(ui, "Native. Local. Deliberate.", 20.0);
                design::muted(ui, "Rust and a GPU-rendered interface. No Electron, browser runtime, account, ads, telemetry, or background service after quitting.");
                ui.add_space(8.0);
                ui.label("Review allowlisted caches, inspect large files, and see CPU, memory and drive usage. No app uninstaller, registry changes, system cleanup, or promised speed boost.");
            });
            ui.add_space(15.0); design::title(ui, "Before moving files", 18.0);
            design::muted(ui, "Close affected apps and keep a backup. Caches may be needed offline; rebuilding them can temporarily slow apps down. Never run Burrow as administrator or with sudo.");
            design::muted(ui, "Trash is not a backup and does not immediately free disk space. Burrow never empties it. Inspect Trash after an error before retrying. Stopping does not undo completed moves.");
            ui.add_space(10.0); design::title(ui, "Make yourself comfortable", 18.0);
            design::muted(ui, "Ctrl / Command + 1–4 switches pages. Tab and Space navigate controls. Escape closes a confirmation or requests cancellation. Ctrl / Command + plus or minus changes text scale.");
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.hyperlink_to("Download a newer release", "https://github.com/NobleSpartan6/burrow/releases");
                ui.hyperlink_to("Installation guide", "https://github.com/NobleSpartan6/burrow/blob/main/docs/INSTALL.md");
                ui.hyperlink_to("Source & report an issue", "https://github.com/NobleSpartan6/burrow");
            });
            design::muted(ui, "To update, quit Burrow and run the newer Windows installer or replace the Mac app. Copy any session log before quitting; paths and history are not saved.");
            ui.add_space(10.0);
            ui.label(RichText::new("Only the links above open your browser. No automatic updates or update checks. Independent project inspired by Mole; not affiliated with its authors.").size(12.0).color(MUTED));
            ui.add_space(6.0);
            ui.colored_label(AMBER, "Preview builds are unsigned on Windows and not notarized on Mac. Do not disable system protections.");
        });
    }
    fn confirmation(&mut self, ctx: &egui::Context) {
        if !self.confirm { return; }
        let mut open = true; let mut accept = false; let mut cancel = false;
        let screen = ctx.content_rect(); let width = (screen.width() - 64.0).clamp(240.0, 480.0);
        egui::Window::new("Review before moving files").open(&mut open).collapsible(false).resizable(false)
            .default_width(width).max_width(width).max_height((screen.height()-72.0).max(140.0)).vscroll(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.set_max_width(width); ui.add_space(5.0);
                design::title(ui, &format!("Move {} files to Trash?", self.selected.len()), 23.0);
                ui.label(RichText::new(human_bytes(self.selected_bytes)).size(21.0).color(ACCENT));
                design::muted(ui, "Includes selected files hidden by the filter. Changed, moved or inaccessible files are refused.");
                design::muted(ui, "Space is not freed until you empty Trash yourself. Rebuilding caches may need an internet connection.");
                let check = ui.checkbox(&mut self.closed_apps, "I have closed the apps whose caches I selected.");
                design::record(&check, "acknowledge"); ui.add_space(5.0);
                ui.horizontal_wrapped(|ui| {
                    cancel = secondary(ui, "Go back", true).clicked();
                    accept = primary(ui, "Move selected files to Trash", self.closed_apps && !self.selected.is_empty()).clicked();
                });
            });
        self.confirm = open && !cancel && !accept;
        if accept && let Some(preview) = &self.preview {
            let mut indices: Vec<_> = self.selected.iter().copied().collect(); indices.sort_unstable();
            let files = indices.into_iter().filter_map(|i| preview.files.get(i).cloned()).collect();
            self.start(Task::Clean(files), ctx);
        }
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("navigation").frame(egui::Frame::new().fill(BG).inner_margin(16)).show(ctx, |ui| {
            ui.add_enabled_ui(!self.confirm && !self.show_log, |ui| {
                let compact = ui.available_width() < 690.0;
                let width = if compact { 65.0 } else { 123.0 }; let nav_width = width * 4.0 + 24.0;
                ui.horizontal(|ui| {
                    ui.set_height(46.0);
                    if ui.available_width() > 790.0 { ui.label(RichText::new("burrow").font(design::heading(21.0)).color(ACCENT)); }
                    let pad = ((ui.available_width() - nav_width) * 0.5 - 34.0).max(0.0); ui.add_space(pad);
                    egui::Frame::new().fill(PANEL).corner_radius(16).stroke(egui::Stroke::new(1.0, LINE)).inner_margin(5).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        ui.horizontal(|ui| {
                            for (page, short) in [(Page::Overview,"Overview"),(Page::Cleanup,"Clean"),(Page::Explorer,"Files"),(Page::About,"Help")] {
                                let selected = page == self.page; let text = if compact { short } else { page.title() };
                                let response = ui.add_sized([width,34.0], egui::Button::new(RichText::new(text).size(if compact {12.0} else {13.0}).color(if selected {design::ACCENT_INK} else {MUTED}))
                                    .fill(if selected {ACCENT} else {PANEL}).corner_radius(11).stroke(egui::Stroke::NONE));
                                design::record(&response, page.title()); if response.clicked() { self.navigate(page, ctx); }
                            }
                        });
                    });
                });
            });
        });
    }
    fn draw(&mut self, ctx: &egui::Context) {
        self.receive();
        if self.busy.is_some() {
            ctx.request_repaint_after(Duration::from_millis(150));
            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose); self.control.stop();
                self.message = "Stopping the task. Close again after it finishes; completed moves are not undone.".into();
            }
        }
        if !self.confirm && !self.show_log {
            for (key,page) in [(egui::Key::Num1,Page::Overview),(egui::Key::Num2,Page::Cleanup),(egui::Key::Num3,Page::Explorer),(egui::Key::Num4,Page::About)] {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, key)) { self.navigate(page, ctx); }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.confirm { self.confirm = false; } else if self.show_log { self.show_log = false; } else { self.control.stop(); }
        }
        self.header(ctx);
        egui::TopBottomPanel::bottom("status").frame(egui::Frame::new().fill(BG).inner_margin(12)).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(task) = self.busy {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if secondary(ui, "Stop", !self.control.cancelled()).clicked() { self.control.stop(); }
                        ui.add(egui::Label::new(format!("{} · {} entries", if self.control.cancelled() {"Stopping…"} else {task}, self.control.visited.load(Ordering::Relaxed))).truncate());
                    });
                } else {
                    ui.label(RichText::new("Local-only · No automatic cleanup").size(11.0).color(MUTED));
                    if ui.available_width() > 180.0 { ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new(format!("{VERSION} · Preview")).size(11.0).color(MUTED)); }); }
                }
            });
        });
        let small = ctx.content_rect().width() < 600.0;
        egui::CentralPanel::default().frame(egui::Frame::new().fill(BG).inner_margin(if small {16} else {28})).show(ctx, |ui| {
            let width = ui.available_width().min(900.0); let side = ((ui.available_width()-width)*0.5).max(0.0);
            ui.horizontal_top(|ui| {
                ui.add_space(side);
                ui.allocate_ui_with_layout(egui::vec2(width, ui.available_height()), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add_enabled_ui(!self.confirm && !self.show_log, |ui| {
                        if !self.message.is_empty() { design::card().inner_margin(12).show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| { ui.colored_label(AMBER, &self.message); if ui.small_button("Dismiss").clicked() { self.message.clear(); } });
                        }); }
                        ui.add_enabled_ui(self.busy != Some("Moving selected files to Trash"), |ui| {
                            match self.page { Page::Overview => self.overview(ui,ctx), Page::Cleanup => self.cleanup(ui,ctx), Page::Explorer => self.explorer(ui,ctx), Page::About => self.about(ui), }
                        });
                    });
                });
            });
        });
        self.confirmation(ctx);
        if self.show_log {
            egui::Window::new("Cleanup session log").open(&mut self.show_log).default_size([680.0,400.0]).max_width((ctx.content_rect().width()-40.0).max(240.0)).show(ctx, |ui| {
                if let Some(report) = &self.report {
                    design::muted(ui, "Contains original paths. Copy this log before quitting; it is not saved automatically.");
                    if secondary(ui, "Copy full report", true).clicked() {ctx.copy_text(report.log.join("\n"));}
                    egui::ScrollArea::vertical().show_rows(ui,28.0,report.log.len(),|ui,range| { for i in range {ui.add(egui::Label::new(&report.log[i]).truncate()).on_hover_text(&report.log[i]);} });
                }
            });
        }
        // Explicit read-only release smoke mode. Never starts a scan or cleanup.
        if let Some((stage, last)) = self.smoke {
            ctx.request_repaint_after(Duration::from_millis(80));
            if last.elapsed() >= Duration::from_millis(350) {
                let pages = [Page::Overview,Page::Cleanup,Page::Explorer,Page::About];
                if stage < pages.len() { self.navigate(pages[stage],ctx); self.smoke = Some((stage+1,Instant::now())); }
                else { println!("BURROW_UI_SMOKE_OK {VERSION}"); self.smoke = None; ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
            }
        }
    }
}
impl eframe::App for Burrow { fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) { self.draw(ctx); } }
impl Drop for Burrow {fn drop(&mut self) {self.control.stop();}}
fn primary(ui:&mut egui::Ui,text:&str,enabled:bool)->egui::Response {design::button(ui,text,enabled,true)}
fn secondary(ui:&mut egui::Ui,text:&str,enabled:bool)->egui::Response {design::button(ui,text,enabled,false)}
fn page_heading(ui:&mut egui::Ui,title:&str,detail:&str) { design::title(ui,title,28.0); design::muted(ui,detail); ui.add_space(14.0); }
fn centered_actions(ui:&mut egui::Ui,width:f32,contents:impl FnOnce(&mut egui::Ui)) {
    let width=width.min(ui.available_width());
    ui.allocate_ui_with_layout(egui::vec2(width,42.0),egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true),contents);
}
#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
