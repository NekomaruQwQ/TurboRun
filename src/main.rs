#![expect(clippy::large_include_file, reason = "embedded fonts")]

extern crate nerd_font_symbols as nf;

mod util;
mod data;
mod engine;
mod ui;
mod app;

mod prelude {
    pub use anyhow::Context;
    pub use itertools::Itertools;
    pub use smol_str::SmolStr;
    pub use tap::prelude::*;
}

#[derive(clap::Parser)]
struct Args {
    /// The path to the TurboRun configuration file (TurboRun.toml),
    /// relative to the directory of the executable.
    #[arg(short, long)]
    config: smol_str::SmolStr,

    /// The path to the plugins directory, relative to the directory
    /// of the executable.
    #[arg(short, long)]
    plugin_pack: Vec<smol_str::SmolStr>,
}

fn main() -> eframe::Result {
    use egui::*;
    use eframe::*;

    let _job_object = create_job_object();

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
        Box::new(|&CreationContext { egui_ctx: ref egui, .. }| {
            use crate::app::App;
            use crate::ui::setup_style;

            egui.set_zoom_factor(1.25);
            setup_fonts(egui);
            setup_style(egui);

            #[cfg(debug_assertions)]
            egui.global_style_mut(|style| {
                style.debug.warn_if_rect_changes_id = false;
            });

            Ok(Box::new(App::new()))
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

/// Installs custom fonts into the egui [`Context`]:
///
/// 1. **Microsoft YaHei UI** (`msyh.ttc` index 1) — system CJK font, picked
///    from `C:/Windows/Fonts` so we don't have to vendor a multi-megabyte CJK
///    blob.
/// 2. **Ubuntu Nerd Font** — embedded at compile time via `include_bytes!`,
///    providing characters for the Font Awesome icons in the BMP Private Use
///    Area (U+F000–U+F8FF).
/// 3. **Maple Mono Nerd Font** — embedded at compile time via `include_bytes!`,
///    a monospaced companion to the above, used in the task editor and also
///    providing the same FA icons as the proportional Nerd Font for consistency.
///
/// Note that the egui default font is not used at all.
fn setup_fonts(egui: &eframe::egui::Context) {
    use std::fs;
    use std::sync::Arc;
    use tap::prelude::*;
    use egui::epaint::text::*;

    // Load Microsoft YaHei UI for CJK character support.
    // msyh.ttc index 1 = Microsoft YaHei UI (UI-optimized variant).
    let msyh_name = "msyahei_ui";
    let msyh_font =
        fs::read("C:/Windows/Fonts/msyh.ttc")
            .expect("Failed to read Microsoft YaHei UI font (msyh.ttc)")
            .pipe(FontData::from_owned)
            .tap_mut(|data| data.index = 1)
            .pipe(Arc::new);

    // Vendored Ubuntu Nerd Font for FA icon glyphs. `from_static` avoids
    // the `Vec` allocation `from_owned` would do, since `include_bytes!`
    // already gives us a `&'static [u8]`.
    let nerd_name = "ubuntu_nf";
    let nerd_font =
        FontData::from_static(include_bytes!("../assets/UbuntuNerdFont-Regular.ttf"))
            .pipe(Arc::new);

    let mono_name = "maple_mono_nf";
    let mono_font =
        FontData::from_static(include_bytes!("../assets/MapleMonoNerdFont-Regular.ttf"))
            .pipe(Arc::new);

    let mut fonts = FontDefinitions::empty();
    fonts.font_data.insert(nerd_name.into(), nerd_font);
    fonts.font_data.insert(msyh_name.into(), msyh_font);
    fonts.font_data.insert(mono_name.into(), mono_font);

    // Append order matters: Latin (default Ubuntu-Light) → CJK (YaHei) →
    // FA icons (Nerd Font).

    let proportional =
        fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.push(nerd_name.into());
    proportional.push(msyh_name.into());

    let monospace =
        fonts.families.entry(FontFamily::Monospace).or_default();
    monospace.push(mono_name.into());
    monospace.push(msyh_name.into());

    egui.set_fonts(fonts);
}
