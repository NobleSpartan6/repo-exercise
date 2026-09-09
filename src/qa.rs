//! Explicit read-only release checks. No scan or deletion; inactive in normal launches.
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct NativeCheck {
    stage: usize,
    configured: bool,
    requested: bool,
    since: Instant,
    output: Option<PathBuf>,
    finished: bool,
}

impl NativeCheck {
    pub fn new() -> Self {
        Self {
            stage: 0,
            configured: false,
            requested: false,
            since: Instant::now(),
            output: std::env::var_os("BURROW_SMOKE_OUTPUT").map(PathBuf::from),
            finished: false,
        }
    }

    /// Returns a page index when the next view must be shown. Exactly one screenshot
    /// request may be outstanding. Capture uses egui's supported GPU event path.
    pub fn step(&mut self, ctx: &egui::Context) -> Option<usize> {
        if self.finished {
            return None;
        }
        if self.requested {
            let image = ctx.input(|input| {
                input.events.iter().find_map(|event| {
                    if let egui::Event::Screenshot { image, .. } = event {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(image) = image {
                if let Err(error) = self.save(&image) {
                    self.fail(ctx, &error);
                    return None;
                }
                self.advance();
            } else if self.since.elapsed() > Duration::from_secs(15) {
                self.fail(ctx, "Timed out waiting for the native GPU screenshot");
                return None;
            }
        }
        if self.stage >= 12 {
            println!("BURROW_UI_SMOKE_OK {}", burrow::VERSION);
            self.finished = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return None;
        }
        ctx.request_repaint_after(Duration::from_millis(80));
        if !self.configured {
            let group = self.stage / 4;
            let size = if group == 0 {
                [1060.0, 800.0]
            } else {
                [720.0, 560.0]
            };
            ctx.set_zoom_factor(if group == 2 { 1.5 } else { 1.0 });
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size.into()));
            self.configured = true;
            self.since = Instant::now();
            return Some(self.stage % 4);
        }
        let settle = if self.stage == 0 {
            Duration::from_millis(2500)
        } else {
            Duration::from_millis(450)
        };
        if !self.requested && self.since.elapsed() >= settle {
            if self.output.is_some() {
                self.requested = true;
                self.since = Instant::now();
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            } else {
                self.advance();
            }
        }
        None
    }

    fn advance(&mut self) {
        self.stage += 1;
        self.configured = false;
        self.requested = false;
    }

    fn fail(&mut self, ctx: &egui::Context, error: &str) {
        eprintln!("BURROW_UI_SMOKE_FAILED: {error}");
        self.finished = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn save(&self, image: &egui::ColorImage) -> Result<(), String> {
        let [width, height] = image.size;
        if width < 300 || height < 250 || width > 8192 || height > 8192 {
            return Err(format!(
                "Unexpected screenshot dimensions: {width}x{height}"
            ));
        }
        if image.pixels.len() != width * height {
            return Err("Screenshot pixel count does not match dimensions".into());
        }
        // A created window alone is not proof that UI rendering succeeded.
        let mut tones = std::collections::HashSet::new();
        for pixel in image.pixels.iter().step_by(11) {
            tones.insert(pixel.to_array());
            if tones.len() >= 24 {
                break;
            }
        }
        if tones.len() < 24 {
            return Err("Blank or insufficiently rendered screenshot".into());
        }
        let output = self
            .output
            .as_ref()
            .ok_or("No screenshot output directory")?;
        std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
        let page = ["overview", "cleanup", "explorer", "help"][self.stage % 4];
        let size = ["desktop", "compact", "large-text"][self.stage / 4];
        let path = output.join(format!("native-{page}-{size}.png"));
        // Refuse to overwrite old evidence, rather than accidentally validating it.
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
        let rgba = image::RgbaImage::from_raw(width as u32, height as u32, bytes)
            .ok_or("Invalid RGBA screenshot")?;
        rgba.write_to(&mut std::io::BufWriter::new(file), image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        println!("BURROW_CAPTURE_OK {page} {size} {width}x{height}");
        Ok(())
    }
}
