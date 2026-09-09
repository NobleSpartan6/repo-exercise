#[path = "workspaces.rs"]
mod workspaces;
use crate::design::{self, ACCENT, AMBER, BG, DANGER, LINE, MUTED, PANEL};
use crate::monitor::{DriveSnapshot, Monitor, Snapshot};
use burrow::{
    VERSION,
    metrics::{DiskCapacity, SpaceLevel, Usage, cpu_fraction},
};
use burrow::{command, maintenance, preferences::Preferences, software};
use burrow::{
    engine::{self, Analysis, Candidate, Cleanup, Control, Preview},
    human_bytes, platform,
};
use eframe::egui::{self, RichText};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{
    atomic::Ordering,
    mpsc::{self, Receiver, Sender},
};
use std::time::Duration;
use workspaces::Workspaces;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Overview,
    Cleanup,
    Explorer,
    Software,
    Optimize,
    About,
}
impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Status",
            Self::Cleanup => "Clean",
            Self::Explorer => "Analyze",
            Self::About => "About & help",
            Self::Software => "Apps",
            Self::Optimize => "Optimize",
        }
    }
}

enum Task {
    Preview(u32),
    Analyze(PathBuf),
    Clean(Vec<Candidate>),
    Software,
    Startup,
    Updates,
    MeasureSoftware(software::Application),
    PlanRemoval(software::Application),
    RemoveSoftware(software::RemovalPlan),
    Maintain(Vec<maintenance::Action>),
    OpenSettings(command::SettingsPage),
    Reveal(PathBuf),
    SavePreferences(Preferences),
}
enum Event {
    Preview(Preview),
    Analyzed(Analysis),
    Cleaned(Cleanup),
    Error(String),
    Software(software::Inventory),
    Startup(Vec<software::StartupItem>),
    UpdateReport(String),
    AppDetails(String),
    AppPlan(software::RemovalPlan),
    AppRemoved(String),
    Maintenance(Vec<maintenance::Record>),
    Notice(String),
    PreferencesSaved,
}

pub struct Burrow {
    page: Page,
    workspace: Workspaces,
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
    smoke: Option<crate::qa::NativeCheck>,
}

impl Burrow {
    pub fn new(cc: &eframe::CreationContext<'_>, smoke: bool) -> Self {
        Self::with_context(&cc.egui_ctx, true, smoke)
    }
    fn with_context(ctx: &egui::Context, monitoring: bool, smoke: bool) -> Self {
        design::configure(ctx);
        let (tx, rx) = mpsc::channel();
        let (monitor, mut message) = match monitoring.then(|| Monitor::start(ctx.clone())) {
            Some(Ok(m)) => (Some(m), String::new()),
            Some(Err(e)) => (None, format!("System monitor unavailable: {e}")),
            None => (None, String::new()),
        };
        let mut workspace = Workspaces::default();
        if monitoring && !smoke {
            match Preferences::load() {
                Ok(p) => workspace.preferences = p,
                Err(e) => {
                    workspace.preferences_error = Some(e.clone());
                    message = e;
                }
            }
        }
        Self {
            page: Page::Cleanup,
            workspace,
            monitor,
            sample: Snapshot::default(),
            drive_sample: DriveSnapshot::default(),
            tx,
            rx,
            control: Control::default(),
            busy: None,
            preview: None,
            analysis: None,
            report: None,
            selected: HashSet::new(),
            selected_bytes: 0,
            preview_bytes: 0,
            visible: Vec::new(),
            filter: String::new(),
            days: 7,
            folder: None,
            message,
            confirm: false,
            closed_apps: false,
            show_log: false,
            smoke: smoke.then(crate::qa::NativeCheck::new),
        }
    }
    fn navigate(&mut self, page: Page, ctx: &egui::Context) {
        self.page = page;
        if let Some(monitor) = &self.monitor {
            monitor.set_overview_visible(
                page == Page::Overview || self.workspace.mini.load(Ordering::Relaxed),
            );
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "Burrow — {}",
            page.title()
        )));
    }
    fn start(&mut self, task: Task, ctx: &egui::Context) {
        if self.busy.is_some() {
            return;
        }
        if matches!(&task, Task::Preview(_) | Task::Clean(_))
            && self.workspace.preferences_error.is_some()
        {
            self.message =
                "Cleanup is paused. Review the saved-settings error before scanning.".into();
            return;
        }
        self.message.clear();
        self.control = Control::default();
        self.busy = Some(match &task {
            Task::Preview(_) => "Scanning known caches",
            Task::Analyze(_) => "Reading folder sizes",
            Task::Clean(_) => "Moving selected files to Trash",
            Task::Software => "Reading installed apps",
            Task::Startup => "Reading startup registrations",
            Task::Updates => "Checking update sources",
            Task::MeasureSoftware(_) => "Measuring app files",
            Task::PlanRemoval(_) => "Preparing an app removal review",
            Task::RemoveSoftware(_) => "Moving an app to Trash",
            Task::Maintain(_) => "Running reviewed maintenance",
            Task::OpenSettings(_) => "Opening the system tool",
            Task::Reveal(_) => "Opening the containing folder",
            Task::SavePreferences(_) => "Saving your preferences",
        });
        if matches!(&task, Task::Preview(_)) {
            self.preview = None;
            self.selected.clear();
            self.selected_bytes = 0;
            self.visible.clear();
        }
        if matches!(&task, Task::Analyze(_)) {
            self.analysis = None;
        }
        if matches!(&task, Task::Software) {
            self.workspace.app_selected = None;
            self.workspace.app_details.clear();
        }
        let preferences = self.workspace.preferences.clone();
        let control = self.control.clone();
        let tx = self.tx.clone();
        let repaint = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("burrow-job".into())
            .spawn(move || {
                let event = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match task {
                    Task::Preview(days) => {
                        let roots=platform::cache_roots().into_iter().filter(|r|!preferences.disabled_caches.contains(&r.label)).collect::<Vec<_>>();
                        engine::preview(&roots, days, &control)
                            .map(|mut p| { let before=p.files.len(); p.files.retain(|f|!preferences.excludes(&f.path)); p.skipped+=(before-p.files.len())as u64; Event::Preview(p) })
                            .unwrap_or_else(Event::Error)
                    }
                    Task::Analyze(path) => engine::analyze(&path, &control)
                        .map(Event::Analyzed)
                        .unwrap_or_else(Event::Error),
                    Task::Clean(files) => Event::Cleaned(engine::clean_selected(&files, &control)),
                    Task::Software => software::inventory(&control).map(Event::Software).unwrap_or_else(Event::Error),
                    Task::Startup => software::startup(&control).map(Event::Startup).unwrap_or_else(Event::Error),
                    Task::Updates => software::check_updates(&control).map(Event::UpdateReport).unwrap_or_else(Event::Error),
                    Task::MeasureSoftware(app) => app.path.as_ref().ok_or_else(||"App location unknown".into()).and_then(|p|engine::analyze(p,&control)).map(|r|{
                        let related=software::related_paths(&app).into_iter().map(|p|p.display().to_string()).collect::<Vec<_>>().join("\n");
                        Event::AppDetails(format!("{}: {} across {} readable files. {} {} excluded/unreadable.\n{}",app.name,human_bytes(r.total_bytes),r.files,if r.partial{"Partial scan."}else{""},r.skipped,if related.is_empty(){"No matching related-data paths found.".into()}else{format!("Related paths to inspect (not selected for removal):\n{related}")}))
                    }).unwrap_or_else(Event::Error),
                    Task::PlanRemoval(app) => software::plan_removal(&app,&control).map(Event::AppPlan).unwrap_or_else(Event::Error),
                    Task::RemoveSoftware(plan) => software::remove_app(&plan,&control).map(Event::AppRemoved).unwrap_or_else(Event::Error),
                    Task::Maintain(actions) => Event::Maintenance(maintenance::perform(&actions,&control)),
                    Task::OpenSettings(page) => command::open_settings(page,&control).map(Event::Notice).unwrap_or_else(Event::Error),
                    Task::Reveal(path) => command::reveal(&path,&control).map(Event::Notice).unwrap_or_else(Event::Error),
                    Task::SavePreferences(p) => p.save().map(|()|Event::PreferencesSaved).unwrap_or_else(Event::Error),
                }))
                .unwrap_or_else(|_| {
                    Event::Error(
                        "The worker stopped unexpectedly. Inspect Trash before retrying a cleanup."
                            .into(),
                    )
                });
                let _ = tx.send(event);
                repaint.request_repaint();
            });
        if let Err(error) = spawned {
            if self.busy == Some("Saving your preferences") {
                self.workspace.preferences_error = Some(error.to_string());
            }
            self.busy = None;
            self.message = format!("Could not start the task: {error}");
        }
    }
    fn receive(&mut self) {
        if let Some(monitor) = &self.monitor {
            if let Some(sample) = monitor.system.take() {
                if sample.ready
                    && let (Some(cpu), Some(mem)) = (
                        cpu_fraction(sample.cpu),
                        Usage::new(sample.used_memory, sample.total_memory),
                    )
                {
                    self.workspace.history.push_back((cpu, mem.fraction()));
                    while self.workspace.history.len() > 60 {
                        self.workspace.history.pop_front();
                    }
                }
                self.sample = sample;
            }
            if let Some(sample) = monitor.drives.take() {
                self.drive_sample = sample;
            }
            if let Some(sample) = monitor.details.take() {
                self.workspace.details = sample;
                self.workspace.refilter_processes();
            }
            if let Some(sample) = monitor.power.take() {
                self.workspace.power = sample;
            }
        }
        while let Ok(event) = self.rx.try_recv() {
            let was_saving = self.busy == Some("Saving your preferences");
            self.busy = None;
            match event {
                Event::Preview(result) => {
                    self.preview_bytes = result
                        .files
                        .iter()
                        .map(|f| f.bytes)
                        .fold(0, u64::saturating_add);
                    self.preview = Some(result);
                    self.refilter();
                }
                Event::Analyzed(result) => self.analysis = Some(result),
                Event::Cleaned(result) => {
                    self.message = format!(
                        "{} files moved to Trash; {} not confirmed moved. {}",
                        result.moved,
                        result.refused,
                        if result.cancelled {
                            "Stopped early. Completed moves were not undone."
                        } else {
                            "Space is not reclaimed until you empty Trash yourself."
                        }
                    );
                    self.report = Some(result);
                    self.preview = None;
                    self.visible.clear();
                    self.selected.clear();
                    self.selected_bytes = 0;
                }
                Event::Error(error) => {
                    if was_saving {
                        self.workspace.preferences_error = Some(error.clone());
                    }
                    self.message = error;
                }
                Event::Notice(message) => self.message = message,
                Event::PreferencesSaved => {
                    self.workspace.preferences_error = None;
                    self.message =
                        "Saved your cache choices and protected folders on this computer.".into();
                }
                Event::Software(inventory) => {
                    self.workspace.app_selected = None;
                    self.workspace.app_details.clear();
                    self.workspace.app_plan = None;
                    self.workspace.app_ack = false;
                    self.workspace.apps = Some(inventory);
                    self.workspace.refilter_apps();
                }
                Event::Startup(items) => self.workspace.startup = Some(items),
                Event::UpdateReport(report) => self.workspace.update_report = report,
                Event::AppDetails(report) => self.workspace.app_details = report,
                Event::AppPlan(plan) => {
                    self.workspace.app_plan = Some(plan);
                    self.workspace.app_ack = false;
                }
                Event::AppRemoved(message) => {
                    self.message = message;
                    self.workspace.apps = None;
                    self.workspace.app_selected = None;
                    self.workspace.app_rows.clear();
                    self.workspace.app_details.clear();
                }
                Event::Maintenance(report) => self.workspace.maintenance_report = report,
            }
        }
    }
    fn refilter(&mut self) {
        let query = self.filter.to_lowercase();
        self.visible = self
            .preview
            .as_ref()
            .map(|p| {
                p.files
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| {
                        query.is_empty()
                            || f.path.to_string_lossy().to_lowercase().contains(&query)
                            || f.category.to_lowercase().contains(&query)
                    })
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
    }
    fn age_control(&mut self, ui: &mut egui::Ui) {
        let previous = self.days;
        ui.horizontal_wrapped(|ui| {
            design::muted(ui, "Only files older than");
            ui.add_enabled_ui(self.busy.is_none(), |ui| {
                egui::ComboBox::from_id_salt("minimum-age")
                    .selected_text(format!("{} days", self.days))
                    .show_ui(ui, |ui| {
                        for days in [7, 30, 90] {
                            ui.selectable_value(&mut self.days, days, format!("{days} days"));
                        }
                    });
            });
        });
        if previous != self.days {
            self.preview = None;
            self.selected.clear();
            self.visible.clear();
            self.selected_bytes = 0;
        }
    }
    fn cleanup(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.preview.is_none() {
            egui::ScrollArea::vertical().id_salt("cleanup-start").show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(26.0); design::orbit(ui, 118.0); ui.add_space(18.0);
                    design::title(ui, "A scan is just a look.", 32.0);
                    design::muted(ui, "Review old caches. Keep what matters.");
                    ui.add_space(15.0); centered_actions(ui, 252.0, |ui| self.age_control(ui)); ui.add_space(5.0);
                    if primary(ui, "Scan caches", self.busy.is_none()&&self.workspace.preferences_error.is_none()).clicked() { self.start(Task::Preview(self.days), ctx); }
                    ui.add_space(10.0);
                    ui.label(RichText::new("Personal folders and system files are not included.").size(12.0).color(MUTED));
                });
                ui.add_space(30.0);
                design::card().show(ui, |ui| {
                    ui.set_min_width((ui.available_width()-1.0).max(0.0));
                    design::title(ui, "You choose what goes.", 16.0);
                    design::muted(ui, "Scan first, review the paths, then select individual files. A separate confirmation is always required.");
                    self.clean_preferences(ui,ctx);
                });
                self.last_report(ui);
            });
            return;
        }
        let height = (ui.available_height() - 340.0).clamp(140.0, 500.0);
        egui::ScrollArea::vertical()
            .id_salt("cleanup-page")
            .show(ui, |ui| self.cleanup_results(ui, ctx, height));
    }
    fn cleanup_results(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, height: f32) {
        page_heading(
            ui,
            "Clean up",
            "Only older files in known caches. Nothing selected by default.",
        );
        ui.horizontal_wrapped(|ui| {
            self.age_control(ui);
            if secondary(
                ui,
                "Scan again",
                self.busy.is_none() && self.workspace.preferences_error.is_none(),
            )
            .clicked()
            {
                self.start(Task::Preview(self.days), ctx);
            }
        });
        self.clean_preferences(ui, ctx);
        let Some(preview) = &self.preview else {
            return;
        };
        let total = self.preview_bytes;
        ui.add_space(10.0);
        design::title(ui, &format!("{} to review", human_bytes(total)), 27.0);
        design::muted(
            ui,
            format!(
                "{} eligible files  ·  {:.2}s  ·  {} entries visited  ·  {} excluded/unreadable",
                preview.files.len(),
                preview.elapsed.as_secs_f64(),
                preview.visited,
                preview.skipped
            ),
        );
        if preview.partial {
            ui.colored_label(
                AMBER,
                "Partial scan — stopped or reached a limit. These are not full-folder totals.",
            );
        }
        for warning in preview.warnings.iter().take(2) {
            ui.colored_label(AMBER, warning);
        }
        if preview.files.is_empty() {
            ui.add_space(24.0);
            design::title(ui, "Nothing eligible in the caches checked.", 22.0);
            design::muted(
                ui,
                "This is not a whole-computer scan. Analyze can inspect a folder of your choice.",
            );
            if secondary(ui, "Open Analyze", true).clicked() {
                self.navigate(Page::Explorer, ctx);
            }
            self.last_report(ui);
            return;
        }
        ui.add_space(7.0);
        let filter = ui.add(
            egui::TextEdit::singleline(&mut self.filter)
                .hint_text("Filter by cache or file path…")
                .desired_width(f32::INFINITY)
                .margin(egui::vec2(12.0, 10.0)),
        );
        design::record(&filter, "filter");
        if filter.changed() {
            self.refilter();
        }
        ui.horizontal_wrapped(|ui| {
            if secondary(ui, "Select visible", self.busy.is_none()).clicked()
                && let Some(preview) = &self.preview
            {
                for i in &self.visible {
                    if self.selected.insert(*i) {
                        self.selected_bytes =
                            self.selected_bytes.saturating_add(preview.files[*i].bytes);
                    }
                }
            }
            if secondary(ui, "Clear selection", !self.selected.is_empty()).clicked() {
                self.selected.clear();
                self.selected_bytes = 0;
            }
            if primary(
                ui,
                "Review selection…",
                !self.selected.is_empty() && self.busy.is_none(),
            )
            .clicked()
            {
                self.closed_apps = false;
                self.confirm = true;
            }
        });
        ui.label(
            RichText::new(format!(
                "{} selected · {} · includes selections hidden by the filter",
                self.selected.len(),
                human_bytes(self.selected_bytes)
            ))
            .size(12.0)
            .color(MUTED),
        );
        ui.separator();
        if self.visible.is_empty() {
            design::muted(
                ui,
                "No files match this filter. Hidden selections are still selected.",
            );
        }
        if let Some(preview) = &self.preview {
            egui::ScrollArea::vertical()
                .id_salt("cleanup-results")
                .auto_shrink([false, false])
                .max_height(height)
                .show_rows(ui, 50.0, self.visible.len(), |ui, range| {
                    for row in range {
                        let i = self.visible[row];
                        let file = &preview.files[i];
                        ui.horizontal(|ui| {
                            ui.set_height(50.0);
                            let mut checked = self.selected.contains(&i);
                            let response = ui
                                .checkbox(&mut checked, "")
                                .on_hover_text("Select this file");
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::Checkbox,
                                    ui.is_enabled(),
                                    checked,
                                    format!("Select {}", file.path.display()),
                                )
                            });
                            if response.changed() {
                                if checked {
                                    self.selected.insert(i);
                                    self.selected_bytes =
                                        self.selected_bytes.saturating_add(file.bytes);
                                } else {
                                    self.selected.remove(&i);
                                    self.selected_bytes =
                                        self.selected_bytes.saturating_sub(file.bytes);
                                }
                            }
                            let width = (ui.available_width() - 104.0).max(40.0);
                            ui.allocate_ui(egui::vec2(width, 44.0), |ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                let name =
                                    file.path.file_name().unwrap_or_default().to_string_lossy();
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(name).font(design::heading(13.0)),
                                    )
                                    .truncate(),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(file.path.to_string_lossy())
                                            .size(11.0)
                                            .color(MUTED),
                                    )
                                    .truncate(),
                                )
                                .on_hover_text(format!(
                                    "{}\n{}",
                                    file.category,
                                    file.path.display()
                                ));
                            });
                            ui.label(
                                RichText::new(human_bytes(file.bytes))
                                    .size(13.0)
                                    .color(ACCENT),
                            );
                        });
                    }
                });
        }
    }
    fn last_report(&mut self, ui: &mut egui::Ui) {
        if let Some(report) = &self.report {
            ui.add_space(12.0);
            design::muted(
                ui,
                format!(
                    "Last cleanup: {} files / {} moved to Trash",
                    report.moved,
                    human_bytes(report.moved_bytes)
                ),
            );
            if secondary(ui, "Session log", true).clicked() {
                self.show_log = true;
            }
        }
    }
    fn choose_folder(&mut self) {
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Choose a folder to inspect")
            .pick_folder()
        {
            if self.folder.as_ref() != Some(&folder) {
                self.analysis = None;
            }
            self.folder = Some(folder);
        }
    }
    fn about(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().id_salt("about").show(ui,|ui|{
            page_heading(ui,"Burrow",&format!("Burrow {VERSION} · Preview · Free and open source"));
            design::title(ui,"Five tools. You stay in control.",22.0);
            ui.label("Clean: review old caches. Apps: inspect software and startup registrations. Optimize: run reviewed maintenance. Analyze: explore folder sizes. Status: view live readings.");
            ui.add_space(12.0);design::title(ui,"Before you remove anything",18.0);
            ui.label("Keep backups and close affected apps. Trash is not a backup. Moving files does not free disk space until you empty Trash yourself. Burrow never empties it.");
            ui.label("Mac app removal moves only the reviewed app bundle. Windows uninstall and startup changes stay in system settings. Some advanced Mole features are not implemented; the feature guide lists them plainly.");
            ui.add_space(12.0);design::title(ui,"Privacy and performance",18.0);
            ui.label("A native Rust app, not a browser. Scans and readings stay on your computer. App-update checks use the internet only after you allow and start them. No accounts, ads or telemetry.");
            ui.label("Cache choices and protected-folder paths are saved locally. Inventories and session logs are not saved automatically. The mini monitor closes when you quit Burrow.");
            ui.add_space(12.0);design::title(ui,"Keyboard shortcuts",18.0);
            ui.label("Ctrl / Command + 1–5: Clean, Apps, Optimize, Analyze, Status. + 6: Help. Tab and Space: controls. Escape: close a review or request Stop. Plus/minus: text size.");
            ui.add_space(10.0);ui.horizontal_wrapped(|ui|{
                ui.hyperlink_to("Downloads","https://github.com/NobleSpartan6/burrow/releases");
                ui.hyperlink_to("Install guide","https://github.com/NobleSpartan6/burrow/blob/main/docs/INSTALL.md");
                ui.hyperlink_to("Feature guide","https://github.com/NobleSpartan6/burrow/blob/main/docs/FEATURES.md");
                ui.hyperlink_to("Report a problem","https://github.com/NobleSpartan6/burrow/issues/new/choose");
            });
            ui.add_space(8.0);design::muted(ui,"Independent project inspired by Mole. MIT licensed. Not affiliated with Mole's authors.");
            ui.colored_label(AMBER,"Preview: unsigned on Windows, not notarized on Mac. Do not disable security protections.");
        });
    }
    fn confirmation(&mut self, ctx: &egui::Context) {
        if !self.confirm {
            return;
        }
        let mut open = true;
        let mut accept = false;
        let mut cancel = false;
        let screen = ctx.content_rect();
        let width = (screen.width() - 64.0).clamp(240.0, 480.0);
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
            let mut indices: Vec<_> = self.selected.iter().copied().collect();
            indices.sort_unstable();
            let files = indices
                .into_iter()
                .filter_map(|i| preview.files.get(i).cloned())
                .collect();
            self.start(Task::Clean(files), ctx);
        }
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("navigation")
            .frame(egui::Frame::new().fill(BG).inner_margin(16))
            .show(ctx, |ui| {
                ui.add_enabled_ui(!self.modal_open(), |ui| {
                    // Compact navigation must not inherit the larger action-button
                    // padding. Segoe UI otherwise grows the help button past the rail.
                    ui.spacing_mut().button_padding = egui::vec2(6.0, 8.0);
                    ui.spacing_mut().interact_size.y = 34.0;
                    let width = ((ui.available_width() - 76.0) / 5.0).clamp(54.0, 100.0);
                    let nav_width = width * 5.0 + 22.0;
                    let pad = ((ui.available_width() - nav_width - 44.0) * 0.5).max(0.0);
                    ui.horizontal(|ui| {
                        ui.set_height(46.0);
                        ui.add_space(pad);
                        egui::Frame::new()
                            .fill(PANEL)
                            .corner_radius(20)
                            .stroke(egui::Stroke::new(1.0, LINE))
                            .inner_margin(5)
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.x = 3.0;
                                ui.spacing_mut().button_padding = egui::vec2(6.0, 8.0);
                                ui.horizontal(|ui| {
                                    for page in [
                                        Page::Cleanup,
                                        Page::Software,
                                        Page::Optimize,
                                        Page::Explorer,
                                        Page::Overview,
                                    ] {
                                        let selected = page == self.page;
                                        let r = ui.add_sized(
                                            [width, 34.0],
                                            egui::Button::new(
                                                RichText::new(page.title()).size(13.0).color(
                                                    if selected {
                                                        design::ACCENT_INK
                                                    } else {
                                                        MUTED
                                                    },
                                                ),
                                            )
                                            .fill(if selected { ACCENT } else { PANEL })
                                            .corner_radius(17)
                                            .stroke(egui::Stroke::NONE),
                                        );
                                        design::record(&r, page.title());
                                        if r.clicked() {
                                            self.navigate(page, ctx);
                                        }
                                    }
                                });
                            });
                        let help = ui
                            .add_sized([30.0, 32.0], egui::Button::new("?"))
                            .on_hover_text("About & help");
                        design::record(&help, "About & help");
                        if help.clicked() {
                            self.navigate(Page::About, ctx);
                        }
                    });
                });
            });
    }
    fn modal_open(&self) -> bool {
        self.confirm
            || self.show_log
            || self.workspace.app_plan.is_some()
            || self.workspace.maintenance_confirm
            || self.workspace.reset_preferences
    }
    fn draw(&mut self, ctx: &egui::Context) {
        if let Some(mut smoke) = self.smoke.take() {
            if let Some(page) = smoke.step(ctx) {
                self.navigate(
                    [
                        Page::Cleanup,
                        Page::Software,
                        Page::Optimize,
                        Page::Explorer,
                        Page::Overview,
                        Page::About,
                    ][page],
                    ctx,
                );
            }
            self.smoke = Some(smoke);
        }
        self.receive();
        if let Some(session) = &self.workspace.awake
            && session.finished()
        {
            self.message = session
                .error
                .take()
                .map(|e| format!("Screen-on session failed: {e}"))
                .unwrap_or_else(|| {
                    "Screen-on session finished. Normal power settings are active.".into()
                });
            self.workspace.awake = None;
        }
        if let Some(m) = &self.monitor {
            m.set_overview_visible(
                self.page == Page::Overview || self.workspace.mini.load(Ordering::Relaxed),
            );
        }
        if self.busy.is_some() {
            ctx.request_repaint_after(Duration::from_millis(150));
            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.control.stop();
                self.message = "Stopping the task. Close again after it finishes; completed moves are not undone.".into();
            }
        }
        if !self.modal_open() {
            for (key, page) in [
                (egui::Key::Num1, Page::Cleanup),
                (egui::Key::Num2, Page::Software),
                (egui::Key::Num3, Page::Optimize),
                (egui::Key::Num4, Page::Explorer),
                (egui::Key::Num5, Page::Overview),
                (egui::Key::Num6, Page::About),
            ] {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, key)) {
                    self.navigate(page, ctx);
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.workspace.app_plan.is_some() {
                self.workspace.app_plan = None;
                self.workspace.app_ack = false;
            } else if self.workspace.reset_preferences {
                self.workspace.reset_preferences = false;
            } else if self.workspace.maintenance_confirm {
                self.workspace.maintenance_confirm = false;
            } else if self.confirm {
                self.confirm = false;
            } else if self.show_log {
                self.show_log = false;
            } else {
                self.control.stop();
            }
        }
        self.header(ctx);
        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame::new().fill(BG).inner_margin(12))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(task) = self.busy {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if secondary(ui, "Stop", !self.control.cancelled()).clicked() {
                                self.control.stop();
                            }
                            ui.add(
                                egui::Label::new(format!(
                                    "{} · {} entries",
                                    if self.control.cancelled() {
                                        "Stopping…"
                                    } else {
                                        task
                                    },
                                    self.control.visited.load(Ordering::Relaxed)
                                ))
                                .truncate(),
                            );
                        });
                    } else {
                        ui.label(
                            RichText::new("Your files stay local · No automatic cleanup")
                                .size(11.0)
                                .color(MUTED),
                        );
                        if ui.available_width() > 180.0 {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(format!("{VERSION} · Preview"))
                                            .size(11.0)
                                            .color(MUTED),
                                    );
                                },
                            );
                        }
                    }
                });
            });
        let small = ctx.content_rect().width() < 600.0;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(if small { 16 } else { 28 }),
            )
            .show(ctx, |ui| {
                let width = ui.available_width().min(900.0);
                let side = ((ui.available_width() - width) * 0.5).max(0.0);
                ui.horizontal_top(|ui| {
                    ui.add_space(side);
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.add_enabled_ui(!self.modal_open(), |ui| {
                                if !self.message.is_empty() {
                                    design::card().inner_margin(12).show(ui, |ui| {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.colored_label(AMBER, &self.message);
                                            if ui.small_button("Dismiss").clicked() {
                                                self.message.clear();
                                            }
                                        });
                                    });
                                }
                                ui.add_enabled_ui(
                                    self.busy != Some("Moving selected files to Trash"),
                                    |ui| match self.page {
                                        Page::Overview => self.status_workspace(ui, ctx),
                                        Page::Software => self.software_workspace(ui, ctx),
                                        Page::Optimize => self.optimize_workspace(ui, ctx),
                                        Page::Cleanup => self.cleanup(ui, ctx),
                                        Page::Explorer => self.analyze_workspace(ui, ctx),
                                        Page::About => self.about(ui),
                                    },
                                );
                            });
                        },
                    );
                });
            });
        self.confirmation(ctx);
        self.workspace_confirmations(ctx);
        self.mini_monitor(ctx);
        if self.show_log {
            egui::Window::new("Cleanup session log").open(&mut self.show_log).default_size([680.0,400.0]).max_width((ctx.content_rect().width()-40.0).max(240.0)).show(ctx, |ui| {
                if let Some(report) = &self.report {
                    design::muted(ui, "Contains original paths. Copy this log before quitting; it is not saved automatically.");
                    if secondary(ui, "Copy full report", true).clicked() {ctx.copy_text(report.log.join("\n"));}
                    egui::ScrollArea::vertical().show_rows(ui,28.0,report.log.len(),|ui,range| { for i in range {ui.add(egui::Label::new(&report.log[i]).truncate()).on_hover_text(&report.log[i]);} });
                }
            });
        }
    }
}
impl eframe::App for Burrow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
}
impl Drop for Burrow {
    fn drop(&mut self) {
        self.control.stop();
    }
}
fn primary(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    design::button(ui, text, enabled, true)
}
fn secondary(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    design::button(ui, text, enabled, false)
}
fn page_heading(ui: &mut egui::Ui, title: &str, detail: &str) {
    design::title(ui, title, 28.0);
    design::muted(ui, detail);
    ui.add_space(14.0);
}
fn centered_actions(ui: &mut egui::Ui, width: f32, contents: impl FnOnce(&mut egui::Ui)) {
    let width = width.min(ui.available_width());
    ui.allocate_ui_with_layout(
        egui::vec2(width, 42.0),
        egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true),
        contents,
    );
}
#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
