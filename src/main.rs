mod util;
mod data;
mod plugin;
mod worker;
mod engine;
mod ui;
mod app;

mod color {
    use egui::Color32;

    pub const RED: Color32 =
        Color32::from_rgb(0xE0, 0x6C, 0x75);
    pub const ORANGE: Color32 =
        Color32::from_rgb(0xD1, 0x9A, 0x66);
    pub const YELLOW: Color32 =
        Color32::from_rgb(0xE5, 0xC0, 0x7B);
    pub const GREEN: Color32 =
        Color32::from_rgb(0x98, 0xC3, 0x79);
    pub const CYAN: Color32 =
        Color32::from_rgb(0x56, 0xB6, 0xC2);
    pub const BLUE: Color32 =
        Color32::from_rgb(0x61, 0xAF, 0xEF);
    pub const PURPLE: Color32 =
        Color32::from_rgb(0xC6, 0x78, 0xDD);
}

#[derive(clap::Parser)]
struct Args {
    /// The path to the TurboRun configuration file (TurboRun.toml),
    /// relative to the directory of the executable.
    #[arg(
        short,
        long,
        env = "TURBORUN_CONFIG_PATH",
        default_value = "TurboRun.toml")]
    config_path: String,

    /// The path to the plugins directory, relative to the directory
    /// of the executable.
    #[arg(
        short,
        long,
        env = "TURBORUN_PLUGIN_DIR",
        default_value = "plugins")]
    plugin_dir: String,
}

fn main() -> eframe::Result {
    use egui::*;
    use eframe::*;

    pretty_env_logger::init();

    eframe::run_native(
        "TurboRun",
        NativeOptions {
            viewport:
                ViewportBuilder::default()
                    .with_inner_size((960.0, 600.0))
                    .with_resizable(false)
                    .with_maximize_button(false),
            centered: true,
            ..NativeOptions::default()
        },
        Box::new(|cc| {
            let egui = &cc.egui_ctx;
            setup_fonts(egui);

            egui.set_visuals(Visuals::dark());
            egui.set_zoom_factor(1.5);
            egui.global_style_mut(|style| {
                style.interaction.selectable_labels = false;
                style.text_styles = [
                   (TextStyle::Heading,
                        FontId::new(15.0, FontFamily::Proportional)),
                   (TextStyle::Body,
                        FontId::new(12.0, FontFamily::Proportional)),
                   (TextStyle::Monospace,
                        FontId::new(10.0, FontFamily::Monospace)),
                   (TextStyle::Button,
                        FontId::new(12.0, FontFamily::Proportional)),
                   (TextStyle::Small,
                        FontId::new(10.5, FontFamily::Proportional)),
                ].into();
            });

            Ok(Box::new(app::App::new()))
        }))
}

/// Creates a Job Object and assigns the current process to it.
///
/// This ensures that if the launcher process exits for any reason (including
/// crashes), the OS will automatically terminate all child processes in the
/// job, preventing orphaned server/app processes.
fn create_job_object() -> win32job::Job {
    let mut job_info =
        win32job::ExtendedLimitInfo::new();
    job_info
        .limit_kill_on_job_close();
    let job_object =
        win32job::Job::create_with_limit_info(&job_info)
            .expect("failed to create job object");
    job_object
        .assign_current_process()
        .expect("failed to assign current process to job object");
    job_object
}

/// Make egui to support CJK characters by loading *Microsoft YaHei UI*,
/// the default system font for Simplified Chinese in modern Windows versions.
fn setup_fonts(egui: &eframe::egui::Context) {
    use std::fs;
    use std::sync::Arc;
    use tap::prelude::*;
    use egui::epaint::text::*;

    // Load Microsoft YaHei UI for CJK character support.
    // msyh.ttc index 1 = Microsoft YaHei UI (UI-optimized variant).
    let msyh_name = String::from("msyahei_ui");
    let msyh_font =
        fs::read("C:/Windows/Fonts/msyh.ttc")
            .expect("Failed to read Microsoft YaHei UI font (msyh.ttc)")
            .pipe(FontData::from_owned)
            .tap_mut(|data| data.index = 1)
            .pipe(Arc::new);

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert(msyh_name.clone(), msyh_font);
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push(msyh_name.clone());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(msyh_name.clone());

    egui.set_fonts(fonts);

    drop(msyh_name);
}
