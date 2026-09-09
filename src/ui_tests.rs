use super::*;
use eframe::egui::{Event as InputEvent, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
fn frame(
    app: &mut Burrow,
    ctx: &egui::Context,
    size: [f32; 2],
    events: Vec<InputEvent>,
) -> egui::FullOutput {
    ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::from(size))),
            events,
            ..Default::default()
        },
        |ctx| app.draw(ctx),
    )
}
fn rect(ctx: &egui::Context, name: &str) -> Rect {
    ctx.data(|d| d.get_temp::<Rect>(egui::Id::new(name)))
        .unwrap_or_else(|| panic!("Missing control {name}"))
}
fn click(app: &mut Burrow, ctx: &egui::Context, name: &str) {
    let pos = rect(ctx, name).center();
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            [1060.0, 800.0],
            vec![
                InputEvent::PointerMoved(pos),
                InputEvent::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    frame(app, ctx, [1060.0, 800.0], vec![]);
}
fn key(app: &mut Burrow, ctx: &egui::Context, key: egui::Key, modifiers: Modifiers) {
    frame(
        app,
        ctx,
        [1060.0, 800.0],
        vec![InputEvent::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }],
    );
}
fn fixture(app: &mut Burrow) -> tempfile::TempDir {
    let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
    for (name, bytes) in [("one.cache", 19), ("two.cache", 27)] {
        let path = dir.path().join(name);
        std::fs::write(&path, vec![7; bytes]).unwrap();
        filetime::set_file_mtime(
            &path,
            filetime::FileTime::from_system_time(
                std::time::SystemTime::now() - Duration::from_secs(30 * 86400),
            ),
        )
        .unwrap();
    }
    let preview = engine::preview(
        &[platform::CacheRoot {
            label: "Fixture".into(),
            path: dir.path().canonicalize().unwrap(),
        }],
        7,
        &Control::default(),
    )
    .unwrap();
    app.tx.send(Event::Preview(preview)).unwrap();
    app.page = Page::Cleanup;
    dir
}
#[test]
fn all_pages_render_at_desktop_compact_and_large_text_sizes() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    for size in [[1060.0, 800.0], [720.0, 560.0], [480.0, 373.0]] {
        for page in [
            Page::Cleanup,
            Page::Software,
            Page::Optimize,
            Page::Explorer,
            Page::Overview,
            Page::About,
        ] {
            app.navigate(page, &ctx);
            for _ in 0..3 {
                let output = frame(&mut app, &ctx, size, vec![]);
                assert!(!output.shapes.is_empty());
            }
            for name in [
                "Clean",
                "Apps",
                "Optimize",
                "Analyze",
                "Status",
                "About & help",
            ] {
                let r = rect(&ctx, name);
                assert!(
                    r.height() <= 40.0,
                    "Navigation wraps at {size:?}: {name} {r:?}"
                );
                assert!(
                    r.left() >= 0.0 && r.right() <= size[0],
                    "{name} clips at {size:?}: {r:?}"
                );
            }
        }
    }
}
#[test]
fn navigation_buttons_switch_pages_without_starting_work() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    click(&mut app, &ctx, "Clean");
    assert!(app.page == Page::Cleanup);
    assert!(app.busy.is_none());
    click(&mut app, &ctx, "Analyze");
    assert!(app.page == Page::Explorer);
    click(&mut app, &ctx, "About & help");
    assert!(app.page == Page::About);
    click(&mut app, &ctx, "Status");
    assert!(app.page == Page::Overview);
}
#[test]
fn scan_preview_is_unselected_and_totals_are_cached() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    let _dir = fixture(&mut app);
    frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    assert_eq!(app.visible.len(), 2);
    assert!(app.selected.is_empty());
    assert_eq!(app.preview_bytes, 46);
    assert_eq!(app.selected_bytes, 0);
}
#[test]
fn hidden_selections_remain_explicit_and_clearable() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    let _dir = fixture(&mut app);
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    click(&mut app, &ctx, "Select visible");
    assert_eq!(app.selected.len(), 2);
    assert_eq!(app.selected_bytes, 46);
    app.filter = "one.cache".into();
    app.refilter();
    assert_eq!(app.visible.len(), 1);
    assert_eq!(app.selected.len(), 2);
    click(&mut app, &ctx, "Clear selection");
    assert!(app.selected.is_empty());
    assert_eq!(app.selected_bytes, 0);
}
#[test]
fn confirmation_requires_acknowledgement_and_escape_never_moves_files() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    let dir = fixture(&mut app);
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    click(&mut app, &ctx, "Select visible");
    click(&mut app, &ctx, "Review selection…");
    assert!(app.confirm);
    assert!(!app.closed_apps);
    click(&mut app, &ctx, "Move selected files to Trash");
    assert!(app.busy.is_none());
    assert!(app.confirm);
    click(&mut app, &ctx, "acknowledge");
    assert!(app.closed_apps);
    key(&mut app, &ctx, egui::Key::Num3, Modifiers::COMMAND);
    assert!(app.page == Page::Cleanup);
    key(&mut app, &ctx, egui::Key::Escape, Modifiers::NONE);
    assert!(!app.confirm);
    assert!(app.busy.is_none());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}
#[test]
fn returning_from_confirmation_preserves_files_and_selection() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    let dir = fixture(&mut app);
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    click(&mut app, &ctx, "Select visible");
    click(&mut app, &ctx, "Review selection…");
    click(&mut app, &ctx, "Go back");
    assert!(!app.confirm);
    assert_eq!(app.selected.len(), 2);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}
#[test]
fn large_results_render_a_bounded_number_of_rows() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    let _dir = fixture(&mut app);
    app.receive();
    let file = app.preview.as_ref().unwrap().files[0].clone();
    app.preview.as_mut().unwrap().files = vec![file; engine::MAX_CANDIDATES];
    app.refilter();
    let mut count = 0;
    for _ in 0..3 {
        count = frame(&mut app, &ctx, [1060.0, 800.0], vec![]).shapes.len();
    }
    assert!(count < 1500, "Virtual list drew too many shapes: {count}");
    assert!(app.selected.is_empty());
}
#[test]
fn explorer_is_read_only_for_empty_and_populated_folders() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    let dir = fixture(&mut app);
    app.receive();
    app.page = Page::Explorer;
    app.folder = Some(dir.path().to_path_buf());
    app.analysis = Some(engine::analyze(dir.path(), &Control::default()).unwrap());
    for size in [[1060.0, 800.0], [720.0, 560.0], [480.0, 373.0]] {
        frame(&mut app, &ctx, size, vec![]);
    }
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    assert!(app.busy.is_none());
}

#[test]
fn new_workspaces_do_not_run_commands_on_navigation() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    for page in [
        Page::Software,
        Page::Optimize,
        Page::Explorer,
        Page::Overview,
    ] {
        app.navigate(page, &ctx);
        for _ in 0..3 {
            frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
        }
        assert!(app.busy.is_none());
        assert!(app.workspace.maintenance_selected.is_empty());
        assert!(!app.workspace.allow_online);
        assert!(app.workspace.awake.is_none());
    }
}
#[test]
fn online_checks_require_explicit_opt_in() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.page = Page::Software;
    app.workspace.software_tab = workspaces::SoftwareTab::Updates;
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    for label in ["Find installable updates", "Read provider report"] {
        click(&mut app, &ctx, label);
        assert!(app.busy.is_none());
        assert!(!app.workspace.allow_online);
    }
}
#[test]
fn app_and_process_filters_keep_bounded_virtual_rows() {
    use crate::monitor::{DetailSnapshot, ProcessRow};
    use burrow::software::{Application, Inventory};
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.workspace.apps = Some(Inventory {
        apps: (0..2000)
            .map(|i| Application {
                name: format!("App {i}"),
                ..Default::default()
            })
            .collect(),
        notes: vec![],
    });
    app.workspace.refilter_apps();
    app.page = Page::Software;
    let output = frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    assert!(output.shapes.len() < 2500);
    app.workspace.app_filter = "App 1999".into();
    app.workspace.refilter_apps();
    assert_eq!(app.workspace.app_rows.len(), 1);
    app.workspace.details = DetailSnapshot {
        ready: true,
        process_count: 4096,
        processes: (0..4096)
            .map(|i| ProcessRow {
                pid: i,
                started: 1,
                name: format!("Process {i}"),
                cpu: i as f32,
                memory: u64::from(i) * 1024,
            })
            .collect(),
        ..Default::default()
    };
    app.workspace.refilter_processes();
    assert_eq!(
        app.workspace.details.processes[app.workspace.process_rows[0]].pid,
        4095
    );
    app.workspace.pinned.insert((1, 1));
    app.workspace.refilter_processes();
    assert_eq!(
        app.workspace.details.processes[app.workspace.process_rows[0]].pid,
        1
    );
    app.workspace.details.processes[1].started = 2;
    app.workspace.refilter_processes();
    assert!(!app.workspace.pinned.contains(&(1, 1)));
    app.page = Page::Overview;
    let output = frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    assert!(output.shapes.len() < 2500);
}
#[test]
fn maintenance_review_can_be_cancelled_without_running_anything() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.page = Page::Optimize;
    app.workspace.maintenance_selected = vec![burrow::maintenance::Action::LookupCache];
    app.workspace.maintenance_confirm = true;
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    key(&mut app, &ctx, egui::Key::Escape, Modifiers::NONE);
    assert!(!app.workspace.maintenance_confirm);
    assert!(app.busy.is_none());
    assert!(app.workspace.maintenance_report.is_empty());
}
#[test]
fn broken_preferences_pause_cleanup_but_not_analysis_navigation() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.workspace.preferences_error = Some("Invalid preferences".into());
    app.start(Task::Preview(7), &ctx);
    assert!(app.busy.is_none());
    assert!(app.preview.is_none());
    app.navigate(Page::Explorer, &ctx);
    frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    assert!(app.page == Page::Explorer);
    app.busy = Some("Saving your preferences");
    app.tx.send(Event::Error("Disk is full".into())).unwrap();
    app.receive();
    assert_eq!(
        app.workspace.preferences_error.as_deref(),
        Some("Disk is full")
    );
    app.tx.send(Event::PreferencesSaved).unwrap();
    app.receive();
    assert!(app.workspace.preferences_error.is_none());
}

#[test]
fn refreshed_inventory_clears_previous_selection_and_details() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.workspace.app_selected = Some(0);
    app.workspace.app_details = "Previous app details".into();
    app.workspace.app_ack = true;
    app.tx
        .send(Event::Software(software::Inventory::default()))
        .unwrap();
    frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    assert!(app.workspace.app_selected.is_none());
    assert!(app.workspace.app_details.is_empty());
    assert!(!app.workspace.app_ack);
}

#[test]
fn unreadable_preferences_block_manual_file_reviews_too() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.workspace.preferences_error = Some("Unreadable".into());
    app.start(
        Task::ReviewFile {
            root: PathBuf::from("/"),
            path: PathBuf::from("/example"),
        },
        &ctx,
    );
    assert!(app.busy.is_none());
    assert!(app.workspace.file_review.is_none());
    assert!(app.message.contains("paused"));
}

#[test]
fn confirmed_result_updates_totals_without_retrying_cleanup() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.tx
        .send(Event::Cleaned(
            Cleanup {
                moved: 1,
                moved_bytes: 42,
                ..Default::default()
            },
            Ok(cleanup_totals::Totals {
                runs: 1,
                files: 1,
                bytes: 42,
            }),
        ))
        .unwrap();
    app.receive();
    assert_eq!(app.workspace.cleanup_totals.unwrap().bytes, 42);
    assert_eq!(app.report.as_ref().unwrap().moved, 1);
    assert!(app.busy.is_none());
    app.tx
        .send(Event::Cleaned(
            Cleanup {
                moved: 1,
                moved_bytes: 9,
                ..Default::default()
            },
            Err("Disk unavailable".into()),
        ))
        .unwrap();
    app.receive();
    assert_eq!(app.report.as_ref().unwrap().moved, 1);
    assert!(app.workspace.totals_error.is_some());
    assert!(app.message.contains("1 files moved"));
    assert!(app.busy.is_none());
}

#[test]
fn update_workers_require_consent_and_revocation_clears_selection() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    for task in [Task::Updates, Task::FindUpdates] {
        app.start(task, &ctx);
        assert!(app.busy.is_none());
        assert!(app.message.contains("Allow internet"));
    }
    app.page = Page::Software;
    app.workspace.software_tab = workspaces::SoftwareTab::Updates;
    app.workspace.allow_online = true;
    app.workspace.update_selected.insert(0);
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    click(&mut app, &ctx, "updates-online-consent");
    assert!(!app.workspace.allow_online);
    assert!(app.workspace.update_selected.is_empty());
    assert!(app.workspace.update_plan.is_none());
    assert!(app.busy.is_none());
}

#[test]
fn update_and_startup_tabs_render_without_starting_provider_processes() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.page = Page::Software;
    for tab in [
        workspaces::SoftwareTab::Updates,
        workspaces::SoftwareTab::Startup,
    ] {
        app.workspace.software_tab = tab;
        for size in [[1060.0, 800.0], [720.0, 560.0], [480.0, 373.0]] {
            for _ in 0..3 {
                let output = frame(&mut app, &ctx, size, vec![]);
                assert!(!output.shapes.is_empty());
                assert!(output.shapes.len() < 2500);
                assert!(app.busy.is_none());
                assert!(!app.workspace.allow_online);
            }
        }
    }
}

#[test]
fn a_file_review_needs_acknowledgement_and_escape_never_moves_it() {
    // A uniquely named disposable fixture inside the required scope; never an existing home file.
    // No Trash operation is invoked by this test.
    let home = dirs::home_dir().unwrap().canonicalize().unwrap();
    let dir = tempfile::Builder::new()
        .prefix(".burrow-ui-review-")
        .tempdir_in(home)
        .unwrap();
    let path = dir.path().join("keep-this-fixture.txt");
    std::fs::write(&path, "unchanged fixture").unwrap();
    let review = file_review::review(
        dir.path(),
        &path,
        &Preferences::default(),
        &Control::default(),
    )
    .unwrap();
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.page = Page::Explorer;
    app.workspace.file_review = Some(review);
    for _ in 0..3 {
        frame(&mut app, &ctx, [1060.0, 800.0], vec![]);
    }
    assert!(app.modal_open());
    assert!(!app.workspace.file_ack);
    click(&mut app, &ctx, "Move reviewed file to Trash");
    assert!(app.busy.is_none());
    assert!(path.exists());
    key(&mut app, &ctx, egui::Key::Num5, Modifiers::COMMAND);
    assert!(app.page == Page::Explorer);
    click(&mut app, &ctx, "file-acknowledge");
    assert!(app.workspace.file_ack);
    key(&mut app, &ctx, egui::Key::Escape, Modifiers::NONE);
    assert!(!app.modal_open());
    assert!(!app.workspace.file_ack);
    assert!(app.busy.is_none());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "unchanged fixture");
}

#[test]
fn update_outcomes_keep_failures_and_invalidate_stale_inventory() {
    let ctx = egui::Context::default();
    let mut app = Burrow::with_context(&ctx, false, false);
    app.workspace.apps = Some(software::Inventory::default());
    app.workspace.app_details = "Stale size".into();
    app.tx
        .send(Event::UpdatesInstalled(vec![updates::Outcome {
            name: "Example".into(),
            id: "Example.App".into(),
            state: updates::State::Failed,
            detail: "Provider did not confirm completion".into(),
        }]))
        .unwrap();
    app.receive();
    assert!(app.message.contains("0 provider completions"));
    assert_eq!(
        app.workspace.update_results[0].state,
        updates::State::Failed
    );
    assert!(app.workspace.apps.is_none());
    assert!(app.workspace.app_details.is_empty());
    assert!(app.busy.is_none());
}
