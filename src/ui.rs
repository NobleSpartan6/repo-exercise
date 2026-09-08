use burrow::{engine::{self, Analysis, Candidate, Cleanup, Control, Preview}, human_bytes, platform};
use crate::monitor::{DriveSnapshot, Monitor, Snapshot};
use burrow::{VERSION, metrics::{DiskCapacity, SpaceLevel, Usage, cpu_fraction}};
use eframe::egui::{self, Color32, RichText};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{atomic::Ordering, mpsc::{self, Receiver, Sender}};
use std::time::Duration;

const INK: Color32 = Color32::from_rgb(24, 43, 46);
const MUTED: Color32 = Color32::from_rgb(91, 111, 114);
const GREEN: Color32 = Color32::from_rgb(19, 124, 102);
const MINT: Color32 = Color32::from_rgb(222, 244, 234);
const AMBER: Color32 = Color32::from_rgb(143, 85, 14);
const DANGER: Color32 = Color32::from_rgb(170, 48, 48);

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
    visible: Vec<usize>,
    filter: String,
    days: u32,
    folder: Option<PathBuf>,
    message: String,
    confirm: bool,
    closed_apps: bool,
    show_log: bool,
}

impl Burrow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = Color32::WHITE;
        visuals.window_fill = Color32::WHITE;
        visuals.override_text_color = Some(INK);
        visuals.selection.bg_fill = MINT;
        visuals.selection.stroke.color = GREEN;
        cc.egui_ctx.set_visuals(visuals);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 9.0);
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(29.0));
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
        cc.egui_ctx.set_style(style);
        let (tx, rx) = mpsc::channel();
        let (monitor, message) = match Monitor::start(cc.egui_ctx.clone()) {
            Ok(m) => (Some(m), String::new()),
            Err(e) => (None, format!("System monitor unavailable: {e}")),
        };
        Self {
            page: Page::Overview, monitor, sample: Snapshot::default(), drive_sample: DriveSnapshot::default(), tx, rx,
            control: Control::default(), busy: None, preview: None, analysis: None,
            report: None, selected: HashSet::new(), selected_bytes: 0, visible: Vec::new(),
            filter: String::new(), days: 7, folder: None, message,
            confirm: false, closed_apps: false, show_log: false,
        }
    }

    fn navigate(&mut self, page: Page, ctx: &egui::Context) {
        self.page = page;
        if let Some(monitor) = &self.monitor {
            monitor.set_overview_visible(page == Page::Overview);
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("Burrow — {}", page.title())));
    }

    fn start(&mut self, task: Task, ctx: &egui::Context) {
        if self.busy.is_some() { return; }
        self.message.clear();
        self.control = Control::default();
        self.busy = Some(match &task {
            Task::Preview(_) => "Scanning known caches",
            Task::Analyze(_) => "Reading folder sizes",
            Task::Clean(_) => "Moving selected files to Trash",
        });
        if matches!(&task, Task::Preview(_)) {
            self.preview = None;
            self.selected.clear();
            self.selected_bytes = 0;
            self.visible.clear();
        }
        if matches!(&task, Task::Analyze(_)) { self.analysis = None; }
        let control = self.control.clone();
        let tx = self.tx.clone();
        let repaint = ctx.clone();
        let spawned = std::thread::Builder::new().name("burrow-job".into()).spawn(move || {
            let event = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match task {
                Task::Preview(days) => engine::preview(&platform::cache_roots(), days, &control)
                    .map(Event::Preview).unwrap_or_else(Event::Error),
                Task::Analyze(path) => engine::analyze(&path, &control)
                    .map(Event::Analyzed).unwrap_or_else(Event::Error),
                Task::Clean(files) => Event::Cleaned(engine::clean_selected(&files, &control)),
            })).unwrap_or_else(|_| Event::Error("The worker stopped unexpectedly. Inspect Trash before retrying a cleanup.".into()));
            let _ = tx.send(event);
            repaint.request_repaint();
        });
        if let Err(error) = spawned {
            self.busy = None;
            self.message = format!("Could not start the task: {error}");
        }
    }

    fn receive(&mut self) {
        if let Some(monitor) = &self.monitor {
            if let Some(sample) = monitor.system.take() { self.sample = sample; }
            if let Some(sample) = monitor.drives.take() { self.drive_sample = sample; }
        }
        while let Ok(event) = self.rx.try_recv() {
            self.busy = None;
            match event {
                Event::Preview(result) => { self.preview = Some(result); self.refilter(); },
                Event::Analyzed(result) => self.analysis = Some(result),
                Event::Cleaned(result) => {
                    self.message = format!("{} files moved to Trash; {} not confirmed moved. {}",
                        result.moved, result.refused,
                        if result.cancelled { "Stopped early. Completed moves were not undone." }
                        else { "Space is not reclaimed until you empty Trash yourself." });
                    self.report = Some(result);
                    self.preview = None;
                    self.visible.clear();
                    self.selected.clear();
                    self.selected_bytes = 0;
                },
                Event::Error(error) => self.message = error,
            }
        }
    }

    fn refilter(&mut self) {
        let query = self.filter.to_lowercase();
        self.visible = self.preview.as_ref().map(|p| p.files.iter().enumerate()
            .filter(|(_, f)| query.is_empty() || f.path.to_string_lossy().to_lowercase().contains(&query)
                || f.category.to_lowercase().contains(&query))
            .map(|(i, _)| i).collect()).unwrap_or_default();
    }

    fn overview(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Scroll the whole overview: a long drive list stays reachable on short
        // windows and at larger display scales. Keep the navigation fixed.
        egui::ScrollArea::vertical().id_salt("overview")
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| self.overview_contents(ui, ctx));
    }

    fn overview_contents(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        subtitle(ui, "See what is busy. Find what is taking up space.");
        ui.add_space(20.0);
        let cpu = if self.sample.ready { cpu_fraction(self.sample.cpu) } else { None };
        let memory = Usage::new(self.sample.used_memory, self.sample.total_memory);
        let cpu_value = cpu.map(|value| format!("{:.0}%", value * 100.0))
            .unwrap_or_else(|| if self.sample.ready { "Unavailable".into() } else { "Measuring…".into() });
        let memory_value = memory.map(|_| human_bytes(self.sample.used_memory))
            .unwrap_or_else(|| if self.sample.ready { "Unavailable".into() } else { "Measuring…".into() });
        let memory_detail = memory.map(|value| format!("of {} total · {:.1}% used",
            human_bytes(self.sample.total_memory), value.percent()))
            .unwrap_or_else(|| "Waiting for an OS memory reading".into());
        if ui.available_width() >= 600.0 {
            ui.columns(2, |columns| {
                metric(&mut columns[0], "CPU in use", cpu_value, cpu.unwrap_or(0.0), "Overall processor usage");
                metric(&mut columns[1], "Memory in use", memory_value,
                    memory.map(Usage::fraction).unwrap_or(0.0), &memory_detail);
            });
        } else {
            metric(ui, "CPU in use", cpu_value, cpu.unwrap_or(0.0), "Overall processor usage");
            ui.add_space(10.0);
            metric(ui, "Memory in use", memory_value,
                memory.map(Usage::fraction).unwrap_or(0.0), &memory_detail);
        }
        ui.add_space(8.0);
        subtitle(ui, "CPU and memory refresh every 2 seconds. No background service when you quit.");
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(12.0);
        ui.label(RichText::new("Make room, deliberately.").size(23.0).strong());
        subtitle(ui, "Start with a read-only scan. Nothing is selected or removed automatically.");
        ui.horizontal_wrapped(|ui| {
            if primary(ui, "Review old caches", self.busy.is_none()).clicked() {
                self.navigate(Page::Cleanup, ctx);
                self.start(Task::Preview(self.days), ctx);
            }
            if ui.button("Find large files").clicked() { self.navigate(Page::Explorer, ctx); }
        });
        ui.add_space(22.0);
        ui.label(RichText::new("Your drives").size(19.0).strong());
        subtitle(ui, "Bars show used space. OS-reported volumes can share physical storage.");
        subtitle(ui, "Refreshed every 10 seconds; virtual and network drives can take longer.");
        if let Some(sampled_at) = self.drive_sample.sampled_at {
            let age = sampled_at.elapsed().as_secs();
            if age >= 30 {
                ui.colored_label(AMBER, format!("Drive information is {age}s old. A volume may be slow to respond."));
            }
        }
        for disk in &self.drive_sample.disks {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let name = if disk.name.is_empty() { &disk.mount } else { &disk.name };
                ui.add(egui::Label::new(RichText::new(name).strong()).truncate()).on_hover_text(name);
                if !disk.name.is_empty() {
                    ui.add(egui::Label::new(RichText::new(&disk.mount).small().color(MUTED)).truncate())
                        .on_hover_text(&disk.mount);
                }
            });
            if let Some(capacity) = DiskCapacity::new(disk.total, disk.available) {
                let (color, warning) = match capacity.space_level() {
                    SpaceLevel::Normal => (GREEN, None),
                    SpaceLevel::Low => (AMBER, Some("Low free space · 10% or less available")),
                    SpaceLevel::VeryLow => (DANGER, Some("Very low free space · 5% or less available")),
                };
                // Labels are outside the bar so neither short nor full bars
                // cover the numbers, and every bar has the same reference width.
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{} available of {}", human_bytes(disk.available), human_bytes(disk.total)));
                    ui.label(RichText::new(format!("{:.1}% used", capacity.used_percent())).small().color(MUTED));
                });
                ui.add(egui::ProgressBar::new(capacity.used_fraction()).fill(color)
                    .desired_width(ui.available_width()).desired_height(7.0));
                if let Some(warning) = warning { ui.colored_label(color, warning); }
            } else {
                subtitle(ui, "Capacity unavailable — this volume has not reported a usable reading.");
            }
        }
        if self.drive_sample.disks.is_empty() {
            subtitle(ui, if self.drive_sample.ready { "No readable drives were reported by the operating system." }
                else { "Reading drive information… CPU and memory are measured separately." });
        }
    }

    fn cleanup(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        subtitle(ui, "Only older files inside a small, known cache allowlist. Your personal folders are not included.");
        ui.add_space(8.0);
        let previous_days = self.days;
        ui.horizontal_wrapped(|ui| {
            ui.label("Only files older than");
            ui.add_enabled_ui(self.busy.is_none(), |ui| {
                egui::ComboBox::from_id_salt("minimum-age").selected_text(format!("{} days", self.days)).show_ui(ui, |ui| {
                    for days in [7, 30, 90] { ui.selectable_value(&mut self.days, days, format!("{days} days")); }
                });
            });
            if primary(ui, "Scan caches", self.busy.is_none()).clicked() { self.start(Task::Preview(self.days), ctx); }
        });
        if self.days != previous_days {
            self.preview = None;
            self.selected.clear();
            self.selected_bytes = 0;
            self.visible.clear();
        }
        if let Some(report) = &self.report {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Last cleanup: {} files / {} moved to Trash", report.moved, human_bytes(report.moved_bytes)));
                if ui.button("Session log").clicked() { self.show_log = true; }
            });
        }
        let Some(preview) = &self.preview else {
            ui.add_space(35.0);
            ui.label(RichText::new("A scan is just a look.").size(25.0));
            subtitle(ui, "Scan to see eligible files. Review the paths, select what you recognize, then confirm.");
            ui.add_space(14.0);
            egui::CollapsingHeader::new("Which caches are checked?").show(ui, |ui| {
                for root in platform::cache_roots() {
                    ui.label(RichText::new(root.label).strong());
                    ui.label(RichText::new(root.path.to_string_lossy()).small().color(MUTED));
                }
                if !cfg!(any(target_os = "macos", target_os = "windows")) {
                    ui.label("This developer build has no cleanup allowlist on this operating system.");
                }
            });
            return;
        };
        ui.add_space(8.0);
        ui.label(format!("{} eligible files · {} entries visited · {:.2}s · {} unreadable/excluded entries",
            preview.files.len(), preview.visited, preview.elapsed.as_secs_f64(), preview.skipped));
        if preview.partial { ui.colored_label(AMBER, "Partial scan: stopped or reached a safety limit. These are not full-folder totals."); }
        for warning in preview.warnings.iter().take(3) { ui.colored_label(AMBER, warning); }
        if preview.files.is_empty() {
            ui.add_space(22.0);
            ui.label(RichText::new("Nothing eligible in the caches checked.").size(23.0));
            subtitle(ui, "That does not mean the whole computer is empty. Disk explorer can help you inspect another folder.");
            return;
        }
        let changed = ui.add(egui::TextEdit::singleline(&mut self.filter)
            .hint_text("Filter by cache name or path…").desired_width(f32::INFINITY)).changed();
        if changed { self.refilter(); }
        ui.horizontal_wrapped(|ui| {
            if ui.button("Select visible").clicked() {
                if let Some(preview) = &self.preview {
                    for i in &self.visible {
                        if self.selected.insert(*i) { self.selected_bytes = self.selected_bytes.saturating_add(preview.files[*i].bytes); }
                    }
                }
            }
            if ui.button("Clear selection").clicked() { self.selected.clear(); self.selected_bytes = 0; }
            ui.label(format!("{} selected · {} (including hidden selections)", self.selected.len(), human_bytes(self.selected_bytes)));
        });
        ui.horizontal_wrapped(|ui| {
            if primary(ui, "Review & move to Trash…", !self.selected.is_empty() && self.busy.is_none()).clicked() {
                self.closed_apps = false;
                self.confirm = true;
            }
            ui.label(RichText::new("Trash retains disk space until emptied.").small().color(MUTED));
        });
        ui.separator();
        if let Some(preview) = &self.preview {
            let row_height = 35.0;
            egui::ScrollArea::vertical().id_salt("cleanup-results").auto_shrink([false, false])
                .show_rows(ui, row_height, self.visible.len(), |ui, range| {
                    for row in range {
                        let index = self.visible[row];
                        let file = &preview.files[index];
                        ui.horizontal(|ui| {
                            let mut selected = self.selected.contains(&index);
                            if ui.checkbox(&mut selected, "").on_hover_text("Select this file").changed() {
                                if selected { self.selected.insert(index); self.selected_bytes = self.selected_bytes.saturating_add(file.bytes); }
                                else { self.selected.remove(&index); self.selected_bytes = self.selected_bytes.saturating_sub(file.bytes); }
                            }
                            ui.add_sized([86.0, row_height], egui::Label::new(human_bytes(file.bytes)));
                            ui.add(egui::Label::new(file.path.to_string_lossy()).truncate())
                                .on_hover_text(format!("{}\n{}", file.category, file.path.display()));
                        });
                    }
                });
        }
    }

    fn explorer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        subtitle(ui, "A read-only look at a folder. This screen never deletes files.");
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add_enabled(self.busy.is_none(), egui::Button::new("Choose folder…")).clicked() {
                if let Some(folder) = rfd::FileDialog::new().set_title("Choose a folder to inspect").pick_folder() {
                    if self.folder.as_ref() != Some(&folder) { self.analysis = None; }
                    self.folder = Some(folder);
                }
            }
            if primary(ui, "Analyze folder", self.folder.is_some() && self.busy.is_none()).clicked() {
                if let Some(folder) = &self.folder { self.start(Task::Analyze(folder.clone()), ctx); }
            }
        });
        if let Some(folder) = &self.folder { ui.add(egui::Label::new(folder.to_string_lossy()).truncate()).on_hover_text(folder.display().to_string()); }
        let Some(result) = &self.analysis else {
            ui.add_space(35.0);
            ui.label(RichText::new("Find the big things first.").size(25.0));
            subtitle(ui, "Choose a local folder. Burrow reads file sizes, not file contents, and retains only the largest 200 files.");
            subtitle(ui, "Network drives can be slow. Links and cloud placeholders are excluded; cancellation waits for the current OS call.");
            return;
        };
        ui.add_space(12.0);
        ui.label(RichText::new(format!("{} across {} files", human_bytes(result.total_bytes), result.files)).size(25.0));
        ui.add(egui::Label::new(format!("Results for {}", result.root.display())).truncate()).on_hover_text(result.root.display().to_string());
        subtitle(ui, &format!("{:.2}s · {} entries visited · {} unreadable/excluded entries. Logical sizes, not allocated disk space.",
            result.elapsed.as_secs_f64(), result.visited, result.skipped));
        if result.partial { ui.colored_label(AMBER, "Partial result: stopped or reached a limit. Totals cover only visited files."); }
        ui.add_space(8.0);
        ui.separator();
        let largest = result.top.first().map(|f| f.bytes).unwrap_or(1);
        egui::ScrollArea::vertical().id_salt("largest-files").auto_shrink([false, false])
            .show_rows(ui, 59.0, result.top.len(), |ui, range| {
                for i in range {
                    let file = &result.top[i];
                    ui.horizontal(|ui| {
                        if ui.small_button("Copy path").clicked() { ctx.copy_text(file.path.to_string_lossy().into_owned()); }
                        ui.label(RichText::new(human_bytes(file.bytes)).strong());
                        ui.add(egui::Label::new(file.path.to_string_lossy()).truncate()).on_hover_text(file.path.display().to_string());
                    });
                    ui.add(egui::ProgressBar::new(ratio(file.bytes, largest)).fill(GREEN).desired_height(4.0));
                }
            });
    }

    fn about(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(RichText::new("Small app. Clear boundaries.").size(25.0));
            subtitle(ui, &format!("Burrow {VERSION} · Preview software · Free and open source (MIT)"));
            ui.add_space(16.0);
            ui.label("Built in Rust with a GPU-rendered egui interface. No Electron, browser runtime, account, advertising or telemetry.");
            ui.label("Burrow is an independent Mole-inspired project. It does not include or execute Mole, and is not affiliated with Tw93 or Faberon.");
            ui.add_space(12.0);
            ui.label(RichText::new("What this version does").strong());
            ui.label("Review older, allowlisted cache files; move selected files to the OS Trash; inspect large files; view CPU, RAM and drive capacity.");
            ui.label("It does not uninstall apps, alter the registry, clear system files, purge Docker, change startup items, or promise to make your computer faster.");
            ui.add_space(12.0);
            ui.label(RichText::new("Before a cleanup").strong());
            ui.label("Close the apps whose caches you select. Caches can be needed offline, and rebuilding them can temporarily make apps slower. Keep a backup. Never run Burrow as administrator or with sudo.");
            ui.label("Trash is a recovery step, not a backup. Burrow never empties it. OS settings and free space can affect recovery. Inspect Trash after an error before retrying.");
            ui.label("A cancelled cleanup does not undo files already moved. Copy the session log before quitting; the app does not persist file paths or history.");
            ui.add_space(12.0);
            ui.label(RichText::new("Keyboard & display").strong());
            ui.label("Ctrl/Command + 1–4: switch pages. Escape: request cancellation. Tab and Space: navigate and activate controls. Ctrl/Command + plus/minus: zoom (egui default).");
            ui.add_space(12.0);
            ui.hyperlink_to("Download a newer release", "https://github.com/NobleSpartan6/repo-exercise/releases");
            ui.label("To update: quit Burrow, run the newer Windows installer or replace the Mac app, then reopen it. No uninstaller or terminal is needed.");
            ui.hyperlink_to("Installation and recovery guide", "https://github.com/NobleSpartan6/repo-exercise/blob/main/docs/INSTALL.md");
            ui.hyperlink_to("Source code and issue tracker", "https://github.com/NobleSpartan6/repo-exercise");
            ui.label(RichText::new("Only these explicit links open your browser. Scans and monitoring stay on your computer.").small().color(MUTED));
        });
    }

    fn confirmation(&mut self, ctx: &egui::Context) {
        if !self.confirm { return; }
        let mut open = true;
        let mut accept = false;
        let mut cancel = false;
        egui::Window::new("Review before moving files").open(&mut open).collapsible(false)
            .resizable(false).default_width(440.0).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new(format!("Move {} files ({}) to Trash?", self.selected.len(), human_bytes(self.selected_bytes))).size(21.0).strong());
                ui.label("This includes all selected files, even those hidden by the filter. Only the files you selected will be considered; changed or inaccessible files are refused.");
                ui.label("This does not immediately free disk space. Burrow never empties Trash. Rebuilding caches may require an internet connection.");
                ui.checkbox(&mut self.closed_apps, "I have closed the apps whose caches I selected.");
                ui.horizontal(|ui| {
                    accept = primary(ui, "Move selected files to Trash", self.closed_apps).clicked();
                    cancel = ui.button("Go back").clicked();
                });
            });
        self.confirm = open && !cancel && !accept;
        if accept {
            if let Some(preview) = &self.preview {
                let mut indices: Vec<_> = self.selected.iter().copied().collect();
                indices.sort_unstable();
                let files = indices.into_iter().filter_map(|i| preview.files.get(i).cloned()).collect();
                self.start(Task::Clean(files), ctx);
            }
        }
    }
}

impl eframe::App for Burrow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.receive();
        if self.busy.is_some() {
            ctx.request_repaint_after(Duration::from_millis(150));
            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.control.stop();
                self.message = "Stopping the task. Close the window again after it finishes; completed moves are not undone.".into();
            }
        }
        if !self.confirm {
            for (key, page) in [(egui::Key::Num1, Page::Overview), (egui::Key::Num2, Page::Cleanup), (egui::Key::Num3, Page::Explorer), (egui::Key::Num4, Page::About)] {
                if ctx.input(|i| i.modifiers.command && i.key_pressed(key)) { self.navigate(page, ctx); }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { self.control.stop(); self.confirm = false; }
        egui::TopBottomPanel::bottom("status").exact_height(34.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(task) = self.busy {
                    ui.spinner();
                    ui.label(format!("{task} · {} entries processed", self.control.visited.load(Ordering::Relaxed)));
                    if ui.small_button("Cancel").clicked() { self.control.stop(); }
                } else { ui.label(RichText::new("Local-only · No automatic cleanup").small().color(MUTED)); }
            });
        });
        egui::SidePanel::left("navigation").exact_width(180.0).resizable(false)
            .frame(egui::Frame::new().fill(INK).inner_margin(18)).show(ctx, |ui| {
                ui.add_enabled_ui(!self.confirm, |ui| {
                    ui.add_space(18.0);
                    ui.label(RichText::new("burrow").size(32.0).strong().color(Color32::WHITE));
                    ui.label(RichText::new("Room to breathe.").small().color(MINT));
                    ui.add_space(35.0);
                    for page in [Page::Overview, Page::Cleanup, Page::Explorer, Page::About] {
                        let selected = self.page == page;
                        let text = RichText::new(page.title()).color(if selected { INK } else { Color32::WHITE });
                        if ui.add_sized([144.0, 42.0], egui::Button::new(text)
                            .fill(if selected { MINT } else { INK }).stroke(egui::Stroke::NONE)).clicked() { self.navigate(page, ctx); }
                    }
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.label(RichText::new("No subscriptions.\nNo account.").small().color(MINT));
                        ui.label(RichText::new(format!("{VERSION} · Preview")).small().color(Color32::WHITE));
                    });
                });
            });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(26)).show(ctx, |ui| {
            ui.add_enabled_ui(!self.confirm, |ui| {
                ui.heading(self.page.title());
                if !self.message.is_empty() { ui.colored_label(AMBER, &self.message); }
                // Freeze the selection while a cleanup is running.
                ui.add_enabled_ui(self.busy != Some("Moving selected files to Trash"), |ui| {
                    match self.page {
                        Page::Overview => self.overview(ui, ctx),
                        Page::Cleanup => self.cleanup(ui, ctx),
                        Page::Explorer => self.explorer(ui, ctx),
                        Page::About => self.about(ui),
                    }
                });
            });
        });
        self.confirmation(ctx);
        if self.show_log {
            egui::Window::new("Cleanup session log").open(&mut self.show_log).default_size([740.0, 440.0]).show(ctx, |ui| {
                if let Some(report) = &self.report {
                    ui.label("Original paths are shown below. Keep this report before quitting if you may need to restore files.");
                    if ui.button("Copy full report").clicked() { ctx.copy_text(report.log.join("\n")); }
                    egui::ScrollArea::vertical().show_rows(ui, 28.0, report.log.len(), |ui, range| {
                        for i in range { ui.add(egui::Label::new(&report.log[i]).truncate()).on_hover_text(&report.log[i]); }
                    });
                }
            });
        }
    }
}

impl Drop for Burrow {
    fn drop(&mut self) { self.control.stop(); }
}

fn subtitle(ui: &mut egui::Ui, text: &str) { ui.label(RichText::new(text).color(MUTED)); }
fn ratio(value: u64, total: u64) -> f32 { if total == 0 { 0.0 } else { (value as f64 / total as f64).clamp(0.0, 1.0) as f32 } }
fn primary(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(enabled, egui::Button::new(RichText::new(text).color(Color32::WHITE)).fill(GREEN))
}
fn metric(ui: &mut egui::Ui, title: &str, value: String, percent: f32, detail: &str) {
    subtitle(ui, title);
    ui.label(RichText::new(value).size(38.0).strong());
    ui.label(RichText::new(detail).small().color(MUTED));
    ui.add(egui::ProgressBar::new(percent).fill(GREEN)
        .desired_width(ui.available_width()).desired_height(7.0));
}
