//! Five focused native workspaces, sharing the existing task/confirmation boundary.
use super::*;
use crate::monitor::{DetailSnapshot, PowerSnapshot};
use burrow::{
    command::SettingsPage,
    maintenance::{Action, Record},
    preferences::Preferences,
    software::{Inventory, RemovalPlan, StartupItem},
    treemap,
};
use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, atomic::AtomicBool};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum SoftwareTab {
    #[default]
    Installed,
    Updates,
    Startup,
}
#[derive(Default)]
pub(super) struct Workspaces {
    pub preferences: Preferences,
    pub preferences_error: Option<String>,
    pub reset_preferences: bool,
    pub details: DetailSnapshot,
    pub power: PowerSnapshot,
    pub history: VecDeque<(f32, f32)>,
    pub process_filter: String,
    pub process_sort: usize,
    pub process_rows: Vec<usize>,
    pub pinned: BTreeSet<(u32, u64)>,
    pub apps: Option<Inventory>,
    pub app_filter: String,
    pub app_sort: usize,
    pub app_rows: Vec<usize>,
    pub app_selected: Option<usize>,
    pub app_details: String,
    pub app_plan: Option<RemovalPlan>,
    pub app_ack: bool,
    pub software_tab: SoftwareTab,
    pub startup: Option<Vec<StartupItem>>,
    pub update_report: String,
    pub allow_online: bool,
    pub maintenance_selected: Vec<Action>,
    pub maintenance_report: Vec<Record>,
    pub maintenance_confirm: bool,
    pub map_list: bool,
    pub mini: Arc<AtomicBool>,
    pub awake: Option<burrow::awake::Awake>,
}
impl Workspaces {
    pub fn refilter_apps(&mut self) {
        let query = self.app_filter.to_lowercase();
        self.app_rows = self
            .apps
            .as_ref()
            .map(|i| {
                i.apps
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| {
                        query.is_empty()
                            || a.name.to_lowercase().contains(&query)
                            || a.publisher.to_lowercase().contains(&query)
                    })
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
        if let Some(inventory) = &self.apps {
            self.app_rows.sort_by(|a, b| {
                let (a, b) = (&inventory.apps[*a], &inventory.apps[*b]);
                if self.app_sort == 1 {
                    b.estimated_bytes
                        .cmp(&a.estimated_bytes)
                        .then(a.name.cmp(&b.name))
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            });
        }
    }
    pub fn refilter_processes(&mut self) {
        let query = self.process_filter.to_lowercase();
        self.process_rows = self
            .details
            .processes
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                query.is_empty()
                    || p.name.to_lowercase().contains(&query)
                    || p.pid.to_string().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        self.process_rows.sort_by(|a, b| {
            let (a, b) = (&self.details.processes[*a], &self.details.processes[*b]);
            self.pinned
                .contains(&(b.pid, b.started))
                .cmp(&self.pinned.contains(&(a.pid, a.started)))
                .then_with(|| match self.process_sort {
                    1 => b.memory.cmp(&a.memory),
                    2 => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    _ => b.cpu.total_cmp(&a.cpu),
                })
                .then(a.pid.cmp(&b.pid))
        });
        self.pinned.retain(|key| {
            self.details
                .processes
                .iter()
                .any(|p| &(p.pid, p.started) == key)
        });
    }
}
fn subtitle(ui: &mut egui::Ui, title: &str, detail: &str) {
    page_heading(ui, title, detail);
}
fn clipped(ui: &mut egui::Ui, text: impl Into<String>) {
    let text = text.into();
    ui.add(egui::Label::new(&text).truncate())
        .on_hover_text(&text);
}
fn chart(ui: &mut egui::Ui, data: &VecDeque<(f32, f32)>, second: bool) {
    let (r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), egui::Sense::hover());
    if data.len() < 2 {
        return;
    }
    let points = data
        .iter()
        .enumerate()
        .map(|(i, v)| {
            egui::pos2(
                r.left() + r.width() * i as f32 / (data.len() - 1) as f32,
                r.bottom() - r.height() * (if second { v.1 } else { v.0 }).clamp(0.0, 1.0),
            )
        })
        .collect();
    ui.painter().add(egui::Shape::line(
        points,
        egui::Stroke::new(1.5, if second { AMBER } else { ACCENT }),
    ));
}
impl Burrow {
    pub(super) fn status_workspace(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical().id_salt("status-page").show(ui,|ui|{
            subtitle(ui,"Your computer, at a glance.","CPU, memory, network and process activity.");
            ui.horizontal_wrapped(|ui|{
                if secondary(ui,"Mini monitor",true).clicked(){self.workspace.mini.store(true,Ordering::Relaxed);}
                if secondary(ui,"Open system monitor",self.busy.is_none()).clicked(){self.start(Task::OpenSettings(SettingsPage::Activity),ctx);}
                design::muted(ui,if self.workspace.details.sampled_at.is_some(){format!("Uptime {}h {}m",self.workspace.details.uptime/3600,(self.workspace.details.uptime/60)%60)}else{"Reading uptime…".into()});
            });
            ui.add_space(12.0);
            let cpu=self.sample.ready.then(||cpu_fraction(self.sample.cpu)).flatten();
            let mem=Usage::new(self.sample.used_memory,self.sample.total_memory);
            let metrics=[("CPU",cpu.map(|c|format!("{:.0}%",c*100.0)).unwrap_or_else(||"—".into()),"Total processor use".into(),cpu),
                ("Memory",mem.map(|m|format!("{:.0}%",m.percent())).unwrap_or_else(||"—".into()),mem.map(|_|format!("{} of {}",human_bytes(self.sample.used_memory),human_bytes(self.sample.total_memory))).unwrap_or_else(||"Reading memory…".into()),mem.map(Usage::fraction)),
                ("Network",self.workspace.details.down.map(|v|format!("{} /s",human_bytes(v.max(0.0) as u64))).unwrap_or_else(||"—".into()),self.workspace.details.up.map(|v|format!("Upload {} /s · non-loopback interfaces",human_bytes(v.max(0.0) as u64))).unwrap_or_else(||"Waiting for a second sample".into()),None),
                ("Battery",self.workspace.power.percent.map(|v|format!("{v:.0}%")).unwrap_or_else(||"—".into()),if self.workspace.power.ready{self.workspace.power.state.clone()}else{"Reading battery status…".into()},self.workspace.power.percent.map(|p|p/100.0))];
            let columns=if ui.available_width()>740.0{4}else if ui.available_width()>360.0{2}else{1};
            for row in metrics.chunks(columns){ui.columns(columns,|cols|{for(i,(label,value,detail,frac))in row.iter().enumerate(){design::metric(&mut cols[i],label,value,detail,*frac);}});ui.add_space(6.0);}
            ui.columns(2,|cols|{chart(&mut cols[0],&self.workspace.history,false);chart(&mut cols[1],&self.workspace.history,true);});
            design::muted(ui,"CPU and memory history · last 60 samples while the app is open");
            ui.add_space(12.0);
            if self.workspace.details.temperatures.is_empty(){design::muted(ui,"Temperature sensors: not reported by this OS. GPU usage and fan controls are not supported in this build.");}
            else{ui.horizontal_wrapped(|ui|{for(name,t)in &self.workspace.details.temperatures{ui.label(format!("{name}: {t:.0} °C"));}});}
            ui.label(RichText::new(format!("Swap in use: {} · Process/network readings every 3s; battery every 30s",if self.workspace.details.sampled_at.is_some(){human_bytes(self.workspace.details.swap)}else{"—".into()})).size(12.0).color(MUTED));
            if let Some(when)=self.sample.sampled_at && when.elapsed()>Duration::from_secs(8){ui.colored_label(AMBER,"CPU and memory readings are stale.");}
            if let Some(when)=self.drive_sample.sampled_at && when.elapsed()>Duration::from_secs(30){ui.colored_label(AMBER,"Drive capacity readings are stale.");}
            if let Some(when)=self.workspace.details.sampled_at && when.elapsed()>Duration::from_secs(12){ui.colored_label(AMBER,"Detailed readings are stale. The OS may be slow to respond.");}
            if let Some(when)=self.workspace.power.sampled_at && when.elapsed()>Duration::from_secs(75){ui.colored_label(AMBER,"Battery reading is stale.");}
            ui.add_space(16.0);design::title(ui,"Storage",19.0);
            if !self.drive_sample.ready{design::muted(ui,"Reading drive capacity…");}
            let drive_columns=if ui.available_width()>540.0{2}else{1};
            egui::ScrollArea::vertical().id_salt("status-volumes").max_height(145.0).show(ui,|ui|{
                for row in self.drive_sample.disks.chunks(drive_columns){
                    ui.columns(drive_columns,|columns|{for(i,disk)in row.iter().enumerate(){
                        let ui=&mut columns[i];
                        design::card().inner_margin(12).show(ui,|ui|{
                            ui.set_min_width(ui.available_width());
                    clipped(ui,format!("{}  ·  {}",disk.name,disk.mount));
                    if let Some(cap)=DiskCapacity::new(disk.total,disk.available){
                        design::muted(ui,format!("{} free of {} · {:.0}% used",human_bytes(disk.available),human_bytes(disk.total),cap.used_fraction()*100.0));
                        design::bar(ui,cap.used_fraction(),if cap.space_level()==SpaceLevel::VeryLow{DANGER}else{ACCENT});
                    }else{design::muted(ui,"Capacity not reported");}
                        });
                    }});
                    ui.add_space(5.0);
                }
            });
            ui.add_space(12.0);design::title(ui,"Processes",19.0);
            let mut change=false;
            ui.horizontal_wrapped(|ui|{
                change|=ui.add(egui::TextEdit::singleline(&mut self.workspace.process_filter).hint_text("Find a name or PID").desired_width(220.0)).changed();
                for(i,label)in ["CPU","Memory","Name"].iter().enumerate(){change|=ui.selectable_value(&mut self.workspace.process_sort,i,*label).changed();}
            });
            if change{self.workspace.refilter_processes();}
            design::muted(ui,format!("{} visible / {} OS processes · CPU: one core = 100% · pin to keep a row near the top",self.workspace.process_rows.len(),self.workspace.details.process_count));
            if self.workspace.process_rows.is_empty(){design::muted(ui,if self.workspace.details.sampled_at.is_some(){"No matching processes."}else{"Waiting for process readings…"});}
            let mut pin=None;
            egui::ScrollArea::vertical().id_salt("process-list").max_height(310.0).show_rows(ui,36.0,self.workspace.process_rows.len(),|ui,range|{
                for row in range{
                    let p=&self.workspace.details.processes[self.workspace.process_rows[row]];
                    ui.horizontal(|ui|{
                        if ui.small_button(if self.workspace.pinned.contains(&(p.pid,p.started)){"Unpin"}else{"Pin"}).clicked(){pin=Some((p.pid,p.started));}
                        let w=(ui.available_width()-200.0).max(50.0);ui.allocate_ui(egui::vec2(w,26.0),|ui|{clipped(ui,format!("{} · {}",p.name,p.pid));});
                        ui.monospace(if self.workspace.details.ready&&p.cpu.is_finite(){format!("{:>6.1}%",p.cpu)}else{"     —".into()});ui.monospace(human_bytes(p.memory));
                    });
                }
            });
            if let Some(key)=pin{if !self.workspace.pinned.remove(&key){self.workspace.pinned.insert(key);}self.workspace.refilter_processes();}
        });
    }
    pub(super) fn software_workspace(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical().id_salt("software-page").show(ui,|ui|{
            subtitle(ui,"Apps","See what is installed. Keep updates and startup choices in your control.");
            ui.horizontal_wrapped(|ui|{for(tab,label)in [(SoftwareTab::Installed,"Installed"),(SoftwareTab::Updates,"Updates"),(SoftwareTab::Startup,"Startup")]{ui.selectable_value(&mut self.workspace.software_tab,tab,label);}});
            ui.add_space(12.0);
            match self.workspace.software_tab{
                SoftwareTab::Installed=>self.installed_apps(ui,ctx),
                SoftwareTab::Updates=>{
                    design::title(ui,"Check supported update sources",20.0);
                    design::muted(ui,"Uses WinGet on Windows or Homebrew casks on Mac. This check contacts your configured sources. It does not download or install app updates.");
                    ui.checkbox(&mut self.workspace.allow_online,"Allow this update check to use the internet");
                    ui.horizontal_wrapped(|ui|{
                        if primary(ui,"Check for app updates",self.workspace.allow_online&&self.busy.is_none()).clicked(){self.start(Task::Updates,ctx);}
                        if secondary(ui,"Open app store",self.busy.is_none()).clicked(){self.start(Task::OpenSettings(SettingsPage::Store),ctx);}
                        if secondary(ui,"Open system updates",self.busy.is_none()).clicked(){self.start(Task::OpenSettings(SettingsPage::Updates),ctx);}
                    });
                    ui.add_space(12.0);
                    if self.workspace.update_report.is_empty(){design::muted(ui,"No update check has run. Use each app's own updater for apps outside these sources.");}
                    else{if secondary(ui,"Copy update report",true).clicked(){ctx.copy_text(self.workspace.update_report.clone());}ui.add(egui::Label::new(RichText::new(&self.workspace.update_report).monospace()).wrap());}
                },
                SoftwareTab::Startup=>{
                    design::title(ui,"What is registered to start",20.0);
                    design::muted(ui,"Registration does not mean an item is enabled or currently running. Use system settings to change startup behavior. Modern Mac login items and some Windows scheduled tasks are not listed here.");
                    ui.horizontal_wrapped(|ui|{
                        if primary(ui,"Read startup items",self.busy.is_none()).clicked(){self.start(Task::Startup,ctx);}
                        if secondary(ui,"Manage startup in settings",self.busy.is_none()).clicked(){self.start(Task::OpenSettings(SettingsPage::Startup),ctx);}
                    });
                    if let Some(items)=&self.workspace.startup{
                        design::muted(ui,format!("{} registered items",items.len()));
                        egui::ScrollArea::vertical().id_salt("startup-list").max_height(420.0).show_rows(ui,68.0,items.len(),|ui,range|{for row in range{let item=&items[row];clipped(ui,&item.name);clipped(ui,&item.location);ui.add(egui::Label::new(RichText::new(&item.command).size(11.0).color(MUTED)).truncate()).on_hover_text(&item.command);}});
                    }
                }
            }
        });
    }
    fn installed_apps(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal_wrapped(|ui| {
            if primary(ui, "Find installed apps", self.busy.is_none()).clicked() {
                self.start(Task::Software, ctx);
            }
            if secondary(
                ui,
                if cfg!(windows) {
                    "Uninstall in Windows"
                } else {
                    "Open Applications"
                },
                self.busy.is_none(),
            )
            .clicked()
            {
                self.start(Task::OpenSettings(SettingsPage::Apps), ctx);
            }
        });
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut self.workspace.app_filter)
                        .hint_text("Search apps or publishers")
                        .desired_width(260.0),
                )
                .changed();
            changed |= ui
                .selectable_value(&mut self.workspace.app_sort, 0, "Name")
                .changed();
            changed |= ui
                .selectable_value(&mut self.workspace.app_sort, 1, "Reported size")
                .changed();
        });
        if changed {
            self.workspace.refilter_apps();
        }
        if cfg!(windows) {
            design::muted(
                ui,
                "Select an app, then open Installed apps in Windows and search its name. Windows runs the vendor's uninstaller; Burrow never executes registry command strings.",
            );
        }
        if !self.workspace.app_details.is_empty() {
            design::card().inner_margin(12).show(ui, |ui| {
                ui.label(&self.workspace.app_details);
            });
        }
        let mut task = None;
        if let Some(app) = self
            .workspace
            .app_selected
            .and_then(|i| {
                self.workspace
                    .apps
                    .as_ref()
                    .and_then(|list| list.apps.get(i))
            })
            .cloned()
        {
            design::card().show(ui,|ui|{
                design::title(ui,&app.name,20.0);clipped(ui,app.path.as_ref().map(|p|p.display().to_string()).unwrap_or_else(||"Installation path not reported".into()));
                ui.horizontal_wrapped(|ui|{
                    if secondary(ui,"Measure app files",app.path.is_some()&&self.busy.is_none()).clicked(){task=Some(Task::MeasureSoftware(app.clone()));}
                    if secondary(ui,"Show app folder",app.path.is_some()&&self.busy.is_none()).clicked(){task=app.path.clone().map(Task::Reveal);}
                    if cfg!(target_os="macos")&&primary(ui,"Review app removal…",self.busy.is_none()&&app.origin!="System").clicked(){task=Some(Task::PlanRemoval(app.clone()));}
                });
                if cfg!(target_os="macos"){design::muted(ui,"Removal moves only the app bundle to Trash. Settings, documents, shared services and related data are kept. Use a vendor uninstaller for apps that install background services.");}
            });
        }
        if let Some(inventory) = &self.workspace.apps {
            for note in &inventory.notes {
                design::muted(ui, note);
            }
            ui.add_space(5.0);
            design::muted(
                ui,
                format!("{} matching apps", self.workspace.app_rows.len()),
            );
            egui::ScrollArea::vertical()
                .id_salt("app-list")
                .max_height(380.0)
                .show_rows(ui, 55.0, self.workspace.app_rows.len(), |ui, range| {
                    for row in range {
                        let i = self.workspace.app_rows[row];
                        let app = &inventory.apps[i];
                        ui.horizontal(|ui| {
                            let w = (ui.available_width() - 110.0).max(60.0);
                            let label = format!(
                                "{}\n{} · {}",
                                app.name,
                                if app.version.is_empty() {
                                    "Version unknown"
                                } else {
                                    &app.version
                                },
                                app.origin
                            );
                            let response = ui
                                .add_enabled_ui(self.busy.is_none(), |ui| {
                                    ui.add_sized(
                                        [w, 47.0],
                                        egui::Button::new(egui::RichText::new(label).size(13.0))
                                            .selected(self.workspace.app_selected == Some(i))
                                            .wrap_mode(egui::TextWrapMode::Truncate),
                                    )
                                })
                                .inner;
                            if response.clicked() {
                                self.workspace.app_selected = Some(i);
                                self.workspace.app_details.clear();
                            }
                            ui.label(
                                app.estimated_bytes
                                    .map(human_bytes)
                                    .unwrap_or_else(|| "Not measured".into()),
                            );
                        });
                    }
                });
        } else {
            ui.add_space(20.0);
            design::muted(
                ui,
                "Find installed apps to load a local inventory. No app scans run until you ask.",
            );
        }
        if let Some(task) = task {
            self.start(task, ctx);
        }
    }
    pub(super) fn optimize_workspace(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical().id_salt("optimize-page").show(ui,|ui|{
            subtitle(ui,"Maintenance, without the mystery.","Choose a specific task. Nothing runs just because you open this page.");
            design::muted(ui,"Cache rebuilding can temporarily make things slower. Burrow never changes fan speeds, forces memory to be purged, repairs the registry or asks for administrator access.");
            ui.add_space(12.0);
            design::card().inner_margin(14).show(ui,|ui|{
                design::title(ui,"Keep the screen on",18.0);
                design::muted(ui,"A timed session, not a permanent power-setting change. May use more battery. Stops when you quit Burrow; explicit Sleep and lid behavior remain controlled by your OS.");
                if let Some(session)=&self.workspace.awake {
                    ui.label(if session.active(){format!("Screen-on session · about {} minutes left",session.until.saturating_duration_since(std::time::Instant::now()).as_secs().div_ceil(60))}else{"Starting the power request…".into()});
                    if secondary(ui,"Stop screen-on session",true).clicked(){self.workspace.awake=None;}
                }else if secondary(ui,"Keep screen on for 30 minutes",self.busy.is_none()).clicked(){
                    let repaint=ctx.clone();match burrow::awake::Awake::start(Duration::from_secs(1800),move||repaint.request_repaint()){
                        Ok(session)=>self.workspace.awake=Some(session),Err(e)=>self.message=format!("Could not start a screen-on session: {e}"),
                    }
                }
                if secondary(ui,"Open power settings",self.busy.is_none()).clicked(){self.start(Task::OpenSettings(SettingsPage::Power),ctx);}
            });
            ui.add_space(15.0);
            for action in Action::ALL{
                design::card().inner_margin(14).show(ui,|ui|{
                    ui.set_min_width(ui.available_width());
                    let mut selected=self.workspace.maintenance_selected.contains(&action);
                    if ui.add_enabled(self.busy.is_none()&&action.supported(),egui::Checkbox::new(&mut selected,action.label())).changed(){
                        self.workspace.maintenance_selected.retain(|a|*a!=action);if selected{self.workspace.maintenance_selected.push(action);}
                    }
                    design::muted(ui,action.detail());if !action.supported(){ui.label(RichText::new("Not available on this OS").size(12.0).color(AMBER));}
                });ui.add_space(6.0);
            }
            if primary(ui,"Review maintenance…",self.busy.is_none()&&!self.workspace.maintenance_selected.is_empty()).clicked(){self.workspace.maintenance_confirm=true;}
            if !self.workspace.maintenance_report.is_empty(){
                ui.add_space(14.0);design::title(ui,"What happened",20.0);
                if secondary(ui,"Copy maintenance report",true).clicked(){ctx.copy_text(self.workspace.maintenance_report.iter().map(|r|format!("{}: {}",r.task,r.result)).collect::<Vec<_>>().join("\n"));}
                for record in &self.workspace.maintenance_report{
                    ui.colored_label(if record.success{ACCENT}else{AMBER},&record.task);ui.label(&record.result);ui.separator();
                }
            }
        });
    }
    pub(super) fn workspace_confirmations(&mut self, ctx: &egui::Context) {
        let width = (ctx.content_rect().width() - 48.0).clamp(240.0, 470.0);
        if let Some(plan) = self.workspace.app_plan.clone() {
            let mut open = true;
            let mut accept = false;
            let mut back = false;
            egui::Window::new("Review app removal").open(&mut open).collapsible(false).resizable(false).max_width(width).default_width(width).max_height((ctx.content_rect().height()-80.0).max(130.0)).vscroll(true).anchor(egui::Align2::CENTER_CENTER,[0.0,0.0]).show(ctx,|ui|{
                ui.set_max_width(width);design::title(ui,&format!("Move {} to Trash?",plan.name),22.0);ui.label(format!("{} · {}",human_bytes(plan.bytes),plan.path.display()));
                ui.label("Only this app bundle will move. Related data and services stay in place. Cancel does not undo an already completed move. Keep a backup.");
                let check=ui.checkbox(&mut self.workspace.app_ack,"I quit this app and reviewed what will move.");design::record(&check,"app-acknowledge");
                ui.horizontal_wrapped(|ui|{back=secondary(ui,"Keep app",true).clicked();accept=primary(ui,"Move app to Trash",self.workspace.app_ack&&self.busy.is_none()).clicked();});
            });
            if !open || back || accept {
                self.workspace.app_plan = None;
                self.workspace.app_ack = false;
            }
            if accept {
                self.start(Task::RemoveSoftware(plan), ctx);
            }
        }
        if self.workspace.reset_preferences {
            let mut open = true;
            let mut accept = false;
            egui::Window::new("Reset cleanup settings?").open(&mut open).collapsible(false).resizable(false).max_width(width).default_width(width).vscroll(true).anchor(egui::Align2::CENTER_CENTER,[0.0,0.0]).show(ctx,|ui|{
                ui.label("This replaces your saved cache choices and protected-folder list with the defaults. It does not scan, select, or remove any files. Add your protected folders again before cleaning.");
                accept=primary(ui,"Reset saved cleanup settings",self.busy.is_none()).clicked();
            });
            self.workspace.reset_preferences = open && !accept;
            if accept {
                self.workspace.preferences = Preferences::default();
                self.preview = None;
                self.selected.clear();
                self.selected_bytes = 0;
                self.visible.clear();
                self.start(
                    Task::SavePreferences(self.workspace.preferences.clone()),
                    ctx,
                );
            }
        }
        if self.workspace.maintenance_confirm {
            let mut open = true;
            let mut accept = false;
            let mut back = false;
            egui::Window::new("Review maintenance").open(&mut open).collapsible(false).resizable(false).max_width(width).default_width(width).max_height((ctx.content_rect().height()-80.0).max(130.0)).vscroll(true).anchor(egui::Align2::CENTER_CENTER,[0.0,0.0]).show(ctx,|ui|{
                ui.set_max_width(width);ui.label("Run these tasks using your normal account? Errors and skipped tasks will be reported. No automatic elevation or retry.");
                for a in &self.workspace.maintenance_selected{ui.label(a.label());}
                ui.horizontal_wrapped(|ui|{back=secondary(ui,"Not now",true).clicked();accept=primary(ui,"Run reviewed tasks",self.busy.is_none()).clicked();});
            });
            self.workspace.maintenance_confirm = open && !back && !accept;
            if accept {
                self.start(
                    Task::Maintain(self.workspace.maintenance_selected.clone()),
                    ctx,
                );
            }
        }
    }
    pub(super) fn clean_preferences(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(error) = &self.workspace.preferences_error {
            ui.colored_label(AMBER,format!("Cleanup is paused because its saved settings could not be read or saved: {error}"));
            if secondary(ui, "Reset cleanup settings…", self.busy.is_none()).clicked() {
                self.workspace.reset_preferences = true;
            }
            return;
        }
        egui::CollapsingHeader::new("Choose caches and protect folders").id_salt("clean-scope").show(ui,|ui|{
            design::muted(ui,"These choices are saved only on this computer. They can narrow the scan, never add new cleanup locations.");
            let mut changed=false;
            ui.add_enabled_ui(self.busy.is_none(),|ui|{
                for root in platform::cache_roots(){
                    let mut enabled=!self.workspace.preferences.disabled_caches.contains(&root.label);
                    if ui.checkbox(&mut enabled,&root.label).on_hover_text(root.path.display().to_string()).changed(){
                        changed=true;if enabled{self.workspace.preferences.disabled_caches.remove(&root.label);}else{self.workspace.preferences.disabled_caches.insert(root.label);}
                    }
                }
                if secondary(ui,"Protect a folder…",true).clicked()&&let Some(p)=rfd::FileDialog::new().pick_folder(){
                    match p.canonicalize(){Ok(p)=>{if !self.workspace.preferences.protected.contains(&p){self.workspace.preferences.protected.push(p);changed=true;}},Err(e)=>self.message=e.to_string()}
                }
                let mut remove=None;
                for(i,path)in self.workspace.preferences.protected.iter().enumerate(){ui.horizontal_wrapped(|ui|{clipped(ui,path.display().to_string());if ui.small_button("Remove protection").clicked(){remove=Some(i);}});}
                if let Some(i)=remove{self.workspace.preferences.protected.remove(i);changed=true;}
            });
            if changed{
                self.preview=None;self.visible.clear();self.selected.clear();self.selected_bytes=0;
                self.start(Task::SavePreferences(self.workspace.preferences.clone()),ctx);
            }
        });
    }
    pub(super) fn analyze_workspace(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut request = None;
        egui::ScrollArea::vertical().id_salt("analyze-page").show(ui,|ui|{
            subtitle(ui,"See where your space goes.","A folder map and the largest files. Explore without deleting anything.");
            ui.horizontal_wrapped(|ui|{
                if secondary(ui,"Choose folder…",self.busy.is_none()).clicked(){self.choose_folder();}
                if primary(ui,"Analyze folder",self.folder.is_some()&&self.busy.is_none()).clicked(){request=self.folder.clone().map(Task::Analyze);}
                ui.selectable_value(&mut self.workspace.map_list,false,"Map");ui.selectable_value(&mut self.workspace.map_list,true,"Largest files");
            });
            if let Some(folder)=&self.folder{clipped(ui,folder.display().to_string());}
            let Some(result)=&self.analysis else{ui.add_space(25.0);design::muted(ui,"Choose a folder, then Analyze folder. Larger rectangles mean more bytes. Scans stay on one volume and skip links and cloud placeholders.");return;};
            ui.add_space(9.0);
            ui.horizontal_wrapped(|ui|{
                let mut crumbs:Vec<_>=result.root.ancestors().take(5).collect();crumbs.reverse();
                for path in crumbs{let label=path.file_name().map(|n|n.to_string_lossy().into_owned()).unwrap_or_else(||path.display().to_string());
                    if ui.add_enabled(self.busy.is_none(),egui::Button::new(label)).on_hover_text(path.display().to_string()).clicked(){request=Some(Task::Analyze(path.to_path_buf()));}
                }
            });
            design::title(ui,&format!("{} · {} files",human_bytes(result.total_bytes),result.files),23.0);
            design::muted(ui,format!("{:.2}s · {} excluded/unreadable · logical sizes, not allocated disk space",result.elapsed.as_secs_f64(),result.skipped));
            if result.partial{ui.colored_label(AMBER,"Partial scan. Totals cover visited files only; narrow the folder or scan again.");}
            if !self.workspace.map_list{
                let count=result.children.len().min(47);
                let mut weights:Vec<_>=result.children.iter().take(count).map(|c|c.bytes).collect();
                let other=result.children.iter().skip(count).map(|c|c.bytes).fold(result.other_bytes,u64::saturating_add);
                if other>0{weights.push(other);}
                if weights.iter().all(|n|*n==0){design::muted(ui,"No non-empty readable files found. Empty folders are listed below.");}
                else{
                    let (canvas,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),330.0),egui::Sense::hover());
                    let palette=[egui::Color32::from_rgb(48,83,71),egui::Color32::from_rgb(83,66,44),egui::Color32::from_rgb(54,72,94),egui::Color32::from_rgb(85,55,60),egui::Color32::from_rgb(67,58,87)];
                    for tile in treemap::layout(&weights){
                        let r=egui::Rect::from_min_size(canvas.min+egui::vec2(tile.x*canvas.width(),tile.y*canvas.height()),egui::vec2(tile.width*canvas.width(),tile.height*canvas.height())).shrink(2.0);
                        if r.width()<1.0||r.height()<1.0{continue;}
                        let (name,path,is_dir)=if tile.index<count{let c=&result.children[tile.index];(c.path.file_name().unwrap_or_default().to_string_lossy().into_owned(),Some(&c.path),c.is_dir)}else{("Other items".into(),None,false)};
                        let label=format!("{}\n{}",name,human_bytes(weights[tile.index]));
                        let response=ui.interact(r,ui.id().with(("tree",tile.index)),egui::Sense::click());
                        response.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Button,ui.is_enabled(),label.clone()));
                        ui.painter().rect_filled(r,7,palette[tile.index%palette.len()]);
                        if response.hovered(){ui.painter().rect_stroke(r,7,egui::Stroke::new(1.5,ACCENT),egui::StrokeKind::Inside);}
                        if r.width()>65.0&&r.height()>36.0{ui.painter().with_clip_rect(r.shrink(5.0)).text(r.center(),egui::Align2::CENTER_CENTER,&label,egui::FontId::proportional(13.0),design::TEXT);}
                        if response.clicked()&&is_dir&&self.busy.is_none(){request=path.cloned().map(Task::Analyze);}
                        let response=response.on_hover_text(format!("{}\n{}",path.map(|p|p.display().to_string()).unwrap_or(name),if is_dir{"Click to look inside"}else{"Read-only item"}));
                        response.context_menu(|ui|{if let Some(path)=path{if ui.button("Copy path").clicked(){ctx.copy_text(path.display().to_string());ui.close();}
if ui.add_enabled(self.busy.is_none(),egui::Button::new("Show in file manager")).clicked(){request=Some(Task::Reveal(path.clone()));ui.close();}}});
                    }
                }
                ui.add_space(12.0);
                egui::ScrollArea::vertical().id_salt("child-list").max_height(240.0).show_rows(ui,36.0,result.children.len(),|ui,range|{for row in range{
                    let child=&result.children[row];ui.horizontal(|ui|{
                        if ui.add_enabled(child.is_dir&&self.busy.is_none(),egui::Button::new("Open")).clicked(){request=Some(Task::Analyze(child.path.clone()));}
                        let w=(ui.available_width()-100.0).max(30.0);ui.allocate_ui(egui::vec2(w,26.0),|ui|clipped(ui,child.path.file_name().unwrap_or_default().to_string_lossy().into_owned()));ui.monospace(human_bytes(child.bytes));
                    });
                }});
                if result.other_bytes>0{design::muted(ui,"The folder has more than 2,048 direct children. Unlisted sizes are included in Other items.");}
            }else{
                egui::ScrollArea::vertical().id_salt("top-file-list").max_height(480.0).show_rows(ui,58.0,result.top.len(),|ui,range|{for row in range{
                    let file=&result.top[row];ui.horizontal(|ui|{
                        let w=(ui.available_width()-190.0).max(40.0);ui.allocate_ui(egui::vec2(w,48.0),|ui|{clipped(ui,file.path.file_name().unwrap_or_default().to_string_lossy().into_owned());ui.add(egui::Label::new(RichText::new(file.path.display().to_string()).size(11.0).color(MUTED)).truncate()).on_hover_text(file.path.display().to_string());});
                        ui.monospace(human_bytes(file.bytes));if ui.small_button("Copy path").clicked(){ctx.copy_text(file.path.display().to_string());}
                    });
                }});
            }
        });
        if let Some(task) = request {
            if let Task::Analyze(path) = &task {
                self.folder = Some(path.clone());
            }
            self.start(task, ctx);
        }
    }
    pub(super) fn mini_monitor(&mut self, ctx: &egui::Context) {
        if !self.workspace.mini.load(Ordering::Relaxed) {
            return;
        }
        let shared = self.monitor.as_ref().map(|m| m.mini.clone());
        let open = self.workspace.mini.clone();
        ctx.show_viewport_deferred(
            crate::monitor::mini_id(),
            egui::ViewportBuilder::default()
                .with_title("Burrow — Mini monitor")
                .with_inner_size([310.0, 390.0])
                .with_min_inner_size([280.0, 300.0])
                .with_window_level(egui::WindowLevel::AlwaysOnTop),
            move |ctx, class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    open.store(false, Ordering::Relaxed);
                    ctx.request_repaint_of(egui::ViewportId::ROOT);
                    return;
                }
                // This window reads the workers directly, even while the main window is minimized.
                ctx.request_repaint_after(Duration::from_secs(2));
                let sample = shared
                    .as_ref()
                    .and_then(|s| match s.try_lock() {
                        Ok(value) => Some(value.clone()),
                        Err(std::sync::TryLockError::Poisoned(error)) => {
                            Some(error.into_inner().clone())
                        }
                        Err(std::sync::TryLockError::WouldBlock) => None,
                    })
                    .unwrap_or_default();
                let cpu = sample.system.cpu;
                let memory = sample.system.used_memory;
                let total = sample.system.total_memory;
                let ready = sample.system.ready;
                let down = sample.down;
                let up = sample.up;
                let power = sample.power;
                let content = |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        design::title(ui, "burrow", 21.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close =
                                ui.small_button("Close").on_hover_text("Close mini monitor");
                            design::record(&close, "Close mini monitor");
                            if close.clicked() {
                                open.store(false, Ordering::Relaxed);
                                if class != egui::ViewportClass::Embedded {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                ctx.request_repaint_of(egui::ViewportId::ROOT);
                            }
                        });
                    });
                    design::muted(ui, "Live system readings");
                    ui.separator();
                    design::metric(
                        ui,
                        "CPU",
                        &cpu_fraction(cpu)
                            .filter(|_| ready)
                            .map(|fraction| format!("{:.0}%", fraction * 100.0))
                            .unwrap_or_else(|| "—".into()),
                        "Total processor use",
                        cpu_fraction(cpu),
                    );
                    design::metric(
                        ui,
                        "Memory",
                        &Usage::new(memory, total)
                            .map(|_| human_bytes(memory))
                            .unwrap_or_else(|| "—".into()),
                        &Usage::new(memory, total)
                            .map(|_| format!("of {}", human_bytes(total)))
                            .unwrap_or_else(|| "Reading memory…".into()),
                        Usage::new(memory, total).map(Usage::fraction),
                    );
                    ui.label(format!(
                        "Download: {} /s",
                        down.map(|v| human_bytes(v.max(0.0) as u64))
                            .unwrap_or_else(|| "—".into())
                    ));
                    ui.label(format!(
                        "Upload: {} /s",
                        up.map(|v| human_bytes(v.max(0.0) as u64))
                            .unwrap_or_else(|| "—".into())
                    ));
                    ui.label(format!(
                        "Battery: {} · {}",
                        power
                            .percent
                            .map(|p| format!("{p:.0}%"))
                            .unwrap_or_else(|| "—".into()),
                        power.state
                    ));
                    if sample
                        .sampled_at
                        .is_none_or(|t| t.elapsed() > Duration::from_secs(8))
                        || sample
                            .detail_at
                            .is_none_or(|t| t.elapsed() > Duration::from_secs(12))
                    {
                        design::muted(ui, "Some readings are waiting or stale.");
                    }
                };
                if class == egui::ViewportClass::Embedded {
                    egui::Window::new("Mini monitor").show(ctx, content);
                } else {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, content);
                    });
                }
            },
        );
    }
}
