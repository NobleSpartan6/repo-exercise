//! Selected updates remain a separate opt-in review from a read-only provider report.
use super::workspaces::clipped;
use super::*;
use burrow::command::SettingsPage;

impl Burrow {
    pub(super) fn updates_workspace(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        design::title(ui, "App updates, on your terms", 20.0);
        design::muted(
            ui,
            "Use an already configured package manager. Nothing is selected, downloaded or installed automatically. App Store, Sparkle and other sources are not combined here.",
        );
        let consent = ui.add_enabled(
            self.busy.is_none(),
            egui::Checkbox::new(
                &mut self.workspace.allow_online,
                "Allow update checks and reviewed installs to use the internet",
            ),
        );
        design::record(&consent, "updates-online-consent");
        if consent.changed() && !self.workspace.allow_online {
            self.workspace.update_catalog = None;
            self.workspace.update_selected.clear();
            self.workspace.update_plan = None;
            self.workspace.update_ack = false;
        }
        let ready = self.workspace.allow_online && self.busy.is_none();
        ui.horizontal_wrapped(|ui| {
            if primary(ui, "Find installable updates", ready).clicked() {
                self.start(Task::FindUpdates, ctx);
            }
            if secondary(ui, "Read provider report", ready).clicked() {
                self.start(Task::Updates, ctx);
            }
            if secondary(ui, "Open app store", self.busy.is_none()).clicked() {
                self.start(Task::OpenSettings(SettingsPage::Store), ctx);
            }
            if secondary(ui, "Open system updates", self.busy.is_none()).clicked() {
                self.start(Task::OpenSettings(SettingsPage::Updates), ctx);
            }
        });
        if cfg!(windows) {
            design::muted(
                ui,
                "Install controls need existing PowerShell 7 and Microsoft.WinGet.Client. Only current-user WinGet upgrades are attempted. The read-only report works without that module; missing tools are never installed for you.",
            );
        } else {
            design::muted(
                ui,
                "Install controls cover named Homebrew casks only. Other apps need their own updater. Close the apps you select before installing updates.",
            );
        }
        let mut review = false;
        if let Some(catalog) = &self.workspace.update_catalog {
            ui.add_space(10.0);
            design::muted(
                ui,
                format!(
                    "{} · {} updates · {} selected",
                    catalog.provider().label(),
                    catalog.entries().len(),
                    self.workspace.update_selected.len()
                ),
            );
            if catalog.entries().is_empty() {
                ui.label("This provider reported no eligible updates. That does not mean every app on this computer is current.");
            }
            egui::ScrollArea::vertical()
                .id_salt("installable-updates")
                .max_height(280.0)
                .show_rows(ui, 62.0, catalog.entries().len(), |ui, range| {
                    for row in range {
                        let entry = &catalog.entries()[row];
                        ui.horizontal(|ui| {
                            let mut selected = self.workspace.update_selected.contains(&row);
                            let response =
                                ui.add_enabled(ready, egui::Checkbox::without_text(&mut selected));
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::Checkbox,
                                    ready,
                                    selected,
                                    entry.name(),
                                )
                            });
                            if response.changed() {
                                if selected {
                                    self.workspace.update_selected.insert(row);
                                } else {
                                    self.workspace.update_selected.remove(&row);
                                }
                            }
                            ui.vertical(|ui| {
                                clipped(ui, format!("{} · {}", entry.name(), entry.id()));
                                clipped(
                                    ui,
                                    format!("{} → {}", entry.installed(), entry.available()),
                                );
                            });
                        });
                    }
                });
            ui.horizontal_wrapped(|ui| {
                review = primary(
                    ui,
                    "Review selected updates",
                    ready && !self.workspace.update_selected.is_empty(),
                )
                .clicked();
                if secondary(ui, "Clear update selection", self.busy.is_none()).clicked() {
                    self.workspace.update_selected.clear();
                }
            });
        }
        if review && let Some(catalog) = &self.workspace.update_catalog {
            match catalog.review(&self.workspace.update_selected) {
                Ok(plan) => {
                    self.workspace.update_plan = Some(plan);
                    self.workspace.update_ack = false;
                }
                Err(error) => self.message = error,
            }
        }
        if !self.workspace.update_results.is_empty() {
            ui.add_space(12.0);
            design::title(ui, "Last update run", 18.0);
            design::muted(
                ui,
                "Provider completion is not an independent audit of the installed app. Check its version when you reopen it. Reports are not saved automatically by Burrow; package managers may keep their own logs.",
            );
            if secondary(ui, "Copy installation report", true).clicked() {
                ctx.copy_text(
                    self.workspace
                        .update_results
                        .iter()
                        .map(|r| {
                            format!("{} ({}) — {}\n{}", r.name, r.id, r.state.label(), r.detail)
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n"),
                );
            }
            for result in &self.workspace.update_results {
                egui::CollapsingHeader::new(format!("{} · {}", result.name, result.state.label()))
                    .id_salt((&result.id, "update-result"))
                    .show(ui, |ui| {
                        ui.label(&result.detail);
                    });
            }
        }
        if !self.workspace.update_report.is_empty() {
            ui.add_space(12.0);
            egui::CollapsingHeader::new("Read-only provider report")
                .default_open(self.workspace.update_catalog.is_none())
                .show(ui, |ui| {
                    if secondary(ui, "Copy update report", true).clicked() {
                        ctx.copy_text(self.workspace.update_report.clone());
                    }
                    ui.add(
                        egui::Label::new(RichText::new(&self.workspace.update_report).monospace())
                            .wrap(),
                    );
                });
        }
    }
    pub(super) fn update_confirmation(&mut self, ctx: &egui::Context) {
        let Some(plan) = self.workspace.update_plan.clone() else {
            return;
        };
        let width = (ctx.content_rect().width() - 48.0).clamp(240.0, 510.0);
        let mut open = true;
        let mut back = false;
        let mut accept = false;
        egui::Window::new("Review app updates").open(&mut open).collapsible(false).resizable(false)
            .default_width(width).max_width(width).max_height((ctx.content_rect().height() - 80.0).max(130.0)).vscroll(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.set_max_width(width);
                design::title(ui, &format!("Update {} selected apps?", plan.entries().len()), 22.0);
                ui.label(plan.provider().label());
                for entry in plan.entries() {
                    ui.label(format!("{} ({})\n{} → {}", entry.name(), entry.id(), entry.installed(), entry.available()));
                }
                ui.separator();
                ui.label("Save your work, close these apps and keep backups. Their package manager will download and run vendor installers; dependencies and app data may change. This is not a Trash operation and Burrow cannot undo it.");
                ui.label("Each entry is checked again before starting. A changed entry or failure stops the remaining queue. The review expires five minutes after the update check. Homebrew chooses the cask version at execution time; do not run another package manager operation concurrently.");
                ui.label("Stop and Close stop the queue after the current installer returns. Burrow stays open during that installer. An interrupted or timed-out installer may need recovery in the package manager; it is never retried automatically.");
                let ack = ui.checkbox(&mut self.workspace.update_ack, "I reviewed these apps and allow their package manager to install the updates.");
                design::record(&ack, "updates-acknowledge");
                ui.horizontal_wrapped(|ui| {
                    back = secondary(ui, "Not now", true).clicked();
                    accept = primary(ui, "Install reviewed updates", self.workspace.update_ack && self.workspace.allow_online && self.busy.is_none()).clicked();
                });
            });
        if !open || back || accept {
            self.workspace.update_plan = None;
            self.workspace.update_ack = false;
        }
        if accept {
            self.start(Task::InstallUpdates(plan), ctx);
        }
    }
}
