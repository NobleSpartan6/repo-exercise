#![forbid(unsafe_code)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod monitor;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Burrow — Overview")
            .with_inner_size([1060.0, 760.0])
            .with_min_inner_size([860.0, 620.0])
            .with_icon(icon()),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native("Burrow", options, Box::new(|cc| Ok(Box::new(ui::Burrow::new(cc)))))
}

fn icon() -> eframe::egui::IconData {
    let size = 64;
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let (x, y) = (x as f32 - 31.5, y as f32 - 31.5);
            let outer = x * x + y * y < 29.0 * 29.0;
            let tunnel = x * x / 15.0_f32.powi(2) + (y - 6.0).powi(2) / 19.0_f32.powi(2) < 1.0;
            let inner = x * x / 8.0_f32.powi(2) + (y - 10.0).powi(2) / 14.0_f32.powi(2) < 1.0;
            rgba.extend_from_slice(if !outer { &[0, 0, 0, 0] }
                else if tunnel && !inner { &[244, 252, 248, 255] }
                else { &[19, 124, 102, 255] });
        }
    }
    eframe::egui::IconData { rgba, width: size as u32, height: size as u32 }
}
