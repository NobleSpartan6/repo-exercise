#![forbid(unsafe_code)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod design;
mod monitor;
mod qa;
mod ui;

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        println!("Burrow {}", burrow::VERSION);
        return std::process::ExitCode::SUCCESS;
    }
    let smoke = args == ["--smoke-test"];
    let interaction = args == ["--interaction-test"];
    if !args.is_empty() && !smoke && !interaction {
        eprintln!("Usage: burrow [--version | --smoke-test | --interaction-test]");
        return std::process::ExitCode::FAILURE;
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Burrow — Clean")
            .with_inner_size([1060.0, 800.0])
            .with_min_inner_size([720.0, 560.0])
            .with_icon(icon()),
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        renderer: eframe::Renderer::Wgpu,
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        wgpu_options: native_gpu(),
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    match eframe::run_native(
        "Burrow",
        options,
        Box::new(move |cc| {
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            if smoke && let Some(state) = &cc.wgpu_render_state {
                println!("BURROW_RENDERER {:?}", state.adapter.get_info());
            }
            if interaction {
                cc.egui_ctx
                    .data_mut(|d| d.insert_temp(eframe::egui::Id::new("burrow-hitboxes"), true));
            }
            Ok(Box::new(ui::Burrow::new(cc, smoke)))
        }),
    ) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Burrow could not start: {error}");
            if !smoke {
                rfd::MessageDialog::new().set_title("Burrow could not open a window")
                    .set_level(rfd::MessageLevel::Error)
                    .set_description(format!("Your graphics driver or desktop session could not create the native window.\n\n{error}\n\nTry a local desktop session and your computer manufacturer's graphics updates. Do not disable security protections."))
                    .show();
            }
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn native_gpu() -> eframe::egui_wgpu::WgpuConfiguration {
    use eframe::{
        egui_wgpu::{WgpuConfiguration, WgpuSetupCreateNew},
        wgpu,
    };
    // Only the OS-native backend is compiled. Prefer integrated graphics for this
    // small 2-D interface rather than waking a discrete GPU unnecessarily.
    let setup = WgpuSetupCreateNew {
        instance_descriptor: wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "macos") {
                wgpu::Backends::METAL
            } else {
                wgpu::Backends::DX12
            },
            ..Default::default()
        },
        power_preference: wgpu::PowerPreference::LowPower,
        ..Default::default()
    };
    WgpuConfiguration {
        wgpu_setup: setup.into(),
        ..Default::default()
    }
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
            rgba.extend_from_slice(if !outer {
                &[0, 0, 0, 0]
            } else if tunnel && !inner {
                &[151, 228, 192, 255]
            } else {
                &[17, 21, 25, 255]
            });
        }
    }
    eframe::egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}
