//! Review UI for explicit Analyze actions and path-free file cleanup totals.
use super::*;

impl Burrow {
    pub(super) fn file_confirmation(&mut self, ctx: &egui::Context) {
        let Some(review) = self.workspace.file_review.clone() else {
            return;
        };
        let width = (ctx.content_rect().width() - 48.0).clamp(240.0, 470.0);
        let mut open = true;
        let mut accept = false;
        let mut back = false;
        egui::Window::new("Review file removal")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(width)
            .max_width(width)
            .max_height((ctx.content_rect().height() - 80.0).max(130.0))
            .vscroll(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_max_width(width);
                design::title(ui, "Move this file to Trash?", 22.0);
                ui.label(review.path().display().to_string());
                ui.label(format!("{} · one file only", human_bytes(review.bytes())));
                ui.label("This is your personal file, not an automatically chosen cache. Keep a backup and close any app using it. The review expires after five minutes; changed files are refused.");
                ui.label("Burrow never empties Trash. Moving the file does not free space immediately. Stopping a task does not undo completed moves.");
                let ack = ui.checkbox(&mut self.workspace.file_ack, "I reviewed this exact file and want to remove it.");
                design::record(&ack, "file-acknowledge");
                ui.horizontal_wrapped(|ui| {
                    back = secondary(ui, "Keep file", true).clicked();
                    accept = primary(ui, "Move reviewed file to Trash", self.workspace.file_ack && self.busy.is_none()).clicked();
                });
            });
        if !open || back || accept {
            self.workspace.file_review = None;
            self.workspace.file_ack = false;
        }
        if accept {
            self.start(Task::TrashFile(review), ctx);
        }
    }
    pub(super) fn file_totals(&self, ui: &mut egui::Ui) {
        if let Some(totals) = &self.workspace.cleanup_totals {
            ui.add_space(10.0);
            design::muted(
                ui,
                format!(
                    "Recorded file cleanup: {} · {} files · {} runs",
                    human_bytes(totals.bytes),
                    totals.files,
                    totals.runs
                ),
            );
            design::muted(
                ui,
                "Confirmed moves to Trash since totals were enabled. Not freed disk space; excludes app bundles and older versions.",
            );
        }
        if let Some(error) = &self.workspace.totals_error {
            ui.colored_label(
                AMBER,
                format!(
                    "Saved totals unavailable: {error}. Session results are reported separately."
                ),
            );
        }
    }
}
