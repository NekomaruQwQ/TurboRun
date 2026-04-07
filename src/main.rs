#![expect(clippy::large_include_file, reason = "embedded fonts")]

mod util;
mod data;
mod plugin;
mod worker;
mod engine;
mod theme;
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

/// Font Awesome glyphs from UbuntuMono Nerd Font, available app-wide via the
/// font fallback installed in [`setup_fonts`]. Names mirror Nerd Fonts'
/// `nf-fa-*` cheatsheet entries; codepoints sit in the BMP Private Use Area
/// (U+F000–U+F8FF) so each constant is a single 3-byte UTF-8 sequence and
/// can be passed anywhere a `&str` label is expected.
mod icon {
    // playback / lifecycle
    pub const PLAY:         &str = "\u{F04B}"; // nf-fa-play
    pub const STOP:         &str = "\u{F04D}"; // nf-fa-stop
    pub const REFRESH:      &str = "\u{F021}"; // nf-fa-refresh

    // editing / structural
    pub const PENCIL:       &str = "\u{F040}"; // nf-fa-pencil
    pub const PLUS:         &str = "\u{F067}"; // nf-fa-plus
    pub const TIMES:        &str = "\u{F00D}"; // nf-fa-times
    pub const ARROW_UP:     &str = "\u{F062}"; // nf-fa-arrow_up
    pub const ARROW_DOWN:   &str = "\u{F063}"; // nf-fa-arrow_down

    // domain
    pub const SAVE:         &str = "\u{F0C7}"; // nf-fa-floppy_o
    pub const TRASH:        &str = "\u{F1F8}"; // nf-fa-trash
    pub const PUZZLE_PIECE: &str = "\u{F12E}"; // nf-fa-puzzle_piece
}

#[derive(clap::Parser)]
struct Args {
    /// The path to the TurboRun configuration file (TurboRun.toml),
    /// relative to the directory of the executable.
    #[arg(
        short,
        long,
        env = "TURBORUN_CONFIG",
        default_value = "TurboRun.toml")]
    config_path: String,

    /// The path to the plugins directory, relative to the directory
    /// of the executable.
    #[arg(
        short,
        long,
        env = "TURBORUN_PLUGIN",
        default_value = "plugins")]
    plugin_dir: String,
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
        Box::new(|cc| {
            let egui = &cc.egui_ctx;
            egui.set_zoom_factor(1.25);
            setup_fonts(egui);
            theme::setup_style(egui);
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
