//! Shared, low-cost native visual language. No web view, blur, animation loop,
//! remote assets, or bundled system fonts.
use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Stroke};

pub const BG: Color32 = Color32::from_rgb(17, 21, 25);
pub const PANEL: Color32 = Color32::from_rgb(25, 31, 36);
pub const HOVER: Color32 = Color32::from_rgb(36, 45, 51);
pub const LINE: Color32 = Color32::from_rgb(49, 60, 66);
pub const TEXT: Color32 = Color32::from_rgb(237, 242, 240);
pub const MUTED: Color32 = Color32::from_rgb(161, 178, 181);
pub const ACCENT: Color32 = Color32::from_rgb(151, 228, 192);
pub const ACCENT_INK: Color32 = Color32::from_rgb(16, 44, 33);
pub const AMBER: Color32 = Color32::from_rgb(244, 197, 122);
pub const DANGER: Color32 = Color32::from_rgb(255, 154, 154);

pub fn heading(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("burrow-heading".into()))
}

pub fn configure(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    // Read the user's OS font, never distribute Apple's/Microsoft's font files.
    // Validate first so a corrupt optional font cannot crash the first frame.
    let regular: &[&str] = if cfg!(windows) {
        &[r"C:\Windows\Fonts\segoeui.ttf"]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ]
    } else {
        &["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
    };
    let bold: &[&str] = if cfg!(windows) {
        &[r"C:\Windows\Fonts\segoeuib.ttf"]
    } else if cfg!(target_os = "macos") {
        &["/System/Library/Fonts/Supplemental/Arial Bold.ttf"]
    } else {
        &["/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"]
    };
    for (name, paths, family) in [
        ("burrow-system", regular, FontFamily::Proportional),
        (
            "burrow-bold",
            bold,
            FontFamily::Name("burrow-heading".into()),
        ),
    ] {
        let mut names = fallback.clone();
        for path in paths {
            if std::fs::metadata(path).is_ok_and(|m| m.len() <= 16 * 1024 * 1024)
                && let Ok(data) = std::fs::read(path)
                && data.len() <= 16 * 1024 * 1024
                && ab_glyph::FontArc::try_from_vec(data.clone()).is_ok()
            {
                fonts
                    .font_data
                    .insert(name.into(), egui::FontData::from_owned(data).into());
                names.insert(0, name.into());
                break;
            }
        }
        fonts.families.insert(family, names);
    }
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Dark);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = BG;
    visuals.faint_bg_color = PANEL;
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = Color32::from_rgb(42, 80, 65);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.bg_fill = HOVER;
        widget.weak_bg_fill = PANEL;
        widget.bg_stroke = Stroke::new(1.0, LINE);
        widget.fg_stroke = Stroke::new(1.0, TEXT);
        widget.corner_radius = 10.into();
    }
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, ACCENT);
    visuals.window_corner_radius = 18.into();
    ctx.set_visuals(visuals);
    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(17.0, 11.0);
        style.spacing.interact_size.y = 36.0;
        style.animation_time = 0.0; // No animation-induced repaint loop.
        style
            .text_styles
            .insert(egui::TextStyle::Heading, heading(28.0));
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(12.0));
        style.wrap_mode = Some(egui::TextWrapMode::Wrap);
    });
}

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(16)
        .inner_margin(20)
}

pub fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text).color(MUTED));
}

pub fn title(ui: &mut egui::Ui, text: &str, size: f32) {
    ui.label(RichText::new(text).font(heading(size)).color(TEXT));
}

pub fn button(ui: &mut egui::Ui, text: &str, enabled: bool, primary: bool) -> egui::Response {
    let widget =
        egui::Button::new(RichText::new(text).color(if primary { ACCENT_INK } else { TEXT }))
            .fill(if primary { ACCENT } else { PANEL })
            .corner_radius(11)
            .stroke(Stroke::new(1.0, if primary { ACCENT } else { LINE }));
    let response = ui.add_enabled(enabled, widget);
    record(&response, text);
    response
}

// Native focus/hover feedback; only test instrumentation is omitted from installers.
pub fn record(response: &egui::Response, _name: &str) {
    if response.has_focus() || response.hovered() {
        response
            .ctx
            .layer_painter(response.layer_id)
            .with_clip_rect(response.interact_rect.expand(3.0))
            .rect_stroke(
                response.rect.expand(1.0),
                12,
                Stroke::new(1.0, ACCENT),
                egui::StrokeKind::Outside,
            );
    }

    #[cfg(test)]
    response
        .ctx
        .data_mut(|data| data.insert_temp(egui::Id::new(_name), response.rect));
    #[cfg(not(test))]
    let _ = response;
}

/// A static orbital emblem, not a metric or a rotating 3-D scene.
/// At most 192 line segments per redraw. No textures or large assets to load.
pub fn orbit(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let center = rect.center();
    let p = ui.painter();
    for (radius, color) in [
        (0.46, LINE),
        (0.35, Color32::from_rgb(66, 113, 94)),
        (0.23, ACCENT),
    ] {
        p.circle_stroke(
            center,
            size * radius,
            Stroke::new(if radius < 0.3 { 2.0 } else { 1.0 }, color),
        );
    }
    let angle: f32 = -0.72;
    p.circle_filled(
        center + egui::vec2(angle.cos(), angle.sin()) * size * 0.35,
        4.0,
        ACCENT,
    );
    p.circle_filled(center, 3.0, ACCENT);
}

pub fn bar(ui: &mut egui::Ui, fraction: f32, color: Color32) {
    // Native widget retains accessibility semantics; no text sits on the fill.
    ui.add(
        egui::ProgressBar::new(fraction.clamp(0.0, 1.0))
            .fill(color)
            .desired_width(ui.available_width())
            .desired_height(5.0),
    );
}

pub fn metric(ui: &mut egui::Ui, label: &str, value: &str, detail: &str, fraction: Option<f32>) {
    card().inner_margin(16).show(ui, |ui| {
        ui.set_min_width((ui.available_width() - 1.0).max(0.0));
        muted(ui, label);
        title(ui, value, 29.0);
        ui.add(egui::Label::new(RichText::new(detail).size(12.0).color(MUTED)).truncate())
            .on_hover_text(detail);
        ui.add_space(7.0);
        bar(ui, fraction.unwrap_or(0.0), ACCENT);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn luminance(c: Color32) -> f64 {
        let channel = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }
    #[test]
    fn text_and_action_colors_have_readable_contrast() {
        for (fg, bg) in [
            (TEXT, BG),
            (MUTED, BG),
            (TEXT, PANEL),
            (MUTED, PANEL),
            (ACCENT_INK, ACCENT),
            (AMBER, PANEL),
            (DANGER, PANEL),
        ] {
            let a = luminance(fg);
            let b = luminance(bg);
            assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
        }
    }
}
