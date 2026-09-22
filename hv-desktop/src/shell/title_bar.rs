use eframe::egui::{self, Context, Sense};

use crate::app::HvBibleApp;
use crate::theme::{
    STATUS_ERROR,
};

// ─── Public entry ─────────────────────────────────────────────────────────────

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    egui::TopBottomPanel::top("unified_title_bar")
        .exact_height(32.0)
        .frame(
            egui::Frame::none()
                .fill(crate::theme::bg_surface())
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            ui.set_height(32.0);
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(8.0);

                for label in [
                    "File", "Edit", "View", "Session",
                    "Audio", "ASR", "Bible", "Broadcast",
                    "Tools", "Window", "Help",
                ] {
                    title_menu_btn(ui, app, label);
                }

                // Draggable centre strip
                let drag_w = (ui.available_width() - 138.0).max(0.0);
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(drag_w, 32.0), Sense::click_and_drag());
                if resp.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if resp.double_clicked() {
                    app.window_maximized = !app.window_maximized;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(app.window_maximized));
                }
                ui.painter().rect_filled(rect, 0.0, crate::theme::bg_surface());

                // Window controls — painted shapes, no unicode dependency
                win_minimize_btn(ui, ctx);
                win_maximize_btn(ui, ctx, app);
                win_close_btn(ui, ctx);
            });
        });
}

// ─── Title-bar ghost menu button ─────────────────────────────────────────────

fn title_menu_btn(ui: &mut egui::Ui, app: &mut HvBibleApp, label: &str) {
    ui.scope(|ui| {
        // Ghost visuals: transparent at rest, highlight only on hover/open.
        // We MUST use button_frame=false to completely eliminate egui's default
        // background frame at rest. Since this strips padding, we manually pad the text.
        ui.visuals_mut().button_frame = false;
        
        ui.visuals_mut().widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, crate::theme::text_secondary());
        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.inactive.rounding = egui::Rounding::ZERO;
        
        ui.visuals_mut().widgets.hovered.bg_fill = crate::theme::bg_surface_raised();
        ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.hovered.rounding = egui::Rounding::ZERO;
        ui.visuals_mut().widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0, crate::theme::text_primary());
            
        ui.visuals_mut().widgets.active.bg_fill = crate::theme::bg_surface_raised();
        ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.active.rounding = egui::Rounding::ZERO;
        ui.visuals_mut().widgets.active.fg_stroke =
            egui::Stroke::new(1.0, crate::theme::text_primary());
            
        ui.visuals_mut().widgets.open.bg_fill = crate::theme::bg_surface_raised();
        ui.visuals_mut().widgets.open.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.open.rounding = egui::Rounding::ZERO;
        ui.visuals_mut().widgets.open.fg_stroke =
            egui::Stroke::new(1.0, crate::theme::text_primary());

        // egui's menu_button establishes the proper "menu context" so nested
        // menu_button calls inside it automatically open to the RIGHT.
        let padded_label = format!("  {}  ", label);
        ui.menu_button(
            egui::RichText::new(padded_label).size(13.0),
            |ui| {
                // Ghost visuals inside the dropdown
                ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                ui.visuals_mut().widgets.hovered.bg_fill = crate::theme::bg_surface_raised();
                ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
                ui.style_mut().spacing.item_spacing.y = 0.0;
                ui.set_min_width(180.0);

                match label {
                    "File"      => file_menu(ui),
                    "Edit"      => edit_menu(ui, app),
                    "View"      => view_menu(ui, app),
                    "Session"   => session_menu(ui, app),
                    "Audio"     => audio_menu(ui, app),
                    "ASR"       => asr_menu(ui),
                    "Bible"     => bible_menu(ui, app),
                    "Broadcast" => broadcast_menu(ui, app),
                    "Tools"     => tools_menu(ui),
                    "Window"    => window_menu(ui, app),
                    _           => help_menu(ui),
                }
            },
        );
    });
}


// ─── Window control buttons (painted, no unicode glyphs) ──────────────────────

fn win_minimize_btn(ui: &mut egui::Ui, ctx: &Context) {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(46.0, 32.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 0.0, crate::theme::bg_surface_raised());
        resp.clone().on_hover_text("Minimize");
    }
    // Draw a single horizontal line in centre
    let cy = rect.center().y + 1.0;
    let x0 = rect.center().x - 5.0;
    let x1 = rect.center().x + 5.0;
    ui.painter().line_segment(
        [egui::pos2(x0, cy), egui::pos2(x1, cy)],
        egui::Stroke::new(1.5, crate::theme::text_primary()),
    );
    if resp.clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
}

fn win_maximize_btn(ui: &mut egui::Ui, ctx: &Context, app: &mut HvBibleApp) {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(46.0, 32.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 0.0, crate::theme::bg_surface_raised());
        resp.clone().on_hover_text(if app.window_maximized { "Restore Down" } else { "Maximize" });
    }
    let cx = rect.center().x;
    let cy = rect.center().y;
    let p = ui.painter();
    if app.window_maximized {
        // Restore icon: two overlapping squares (offset)
        let outer = egui::Rect::from_center_size(egui::pos2(cx + 1.5, cy - 1.5), egui::vec2(8.0, 8.0));
        let inner = egui::Rect::from_center_size(egui::pos2(cx - 1.5, cy + 1.5), egui::vec2(8.0, 8.0));
        p.rect_stroke(outer, 0.0, egui::Stroke::new(1.2, crate::theme::text_primary()));
        p.rect_filled(inner, 0.0, crate::theme::bg_surface()); // erase behind
        p.rect_stroke(inner, 0.0, egui::Stroke::new(1.2, crate::theme::text_primary()));
    } else {
        // Maximize icon: plain square
        let sq = egui::Rect::from_center_size(egui::pos2(cx, cy), egui::vec2(9.0, 9.0));
        p.rect_stroke(sq, 0.0, egui::Stroke::new(1.2, crate::theme::text_primary()));
    }
    if resp.clicked() {
        app.window_maximized = !app.window_maximized;
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(app.window_maximized));
    }
}

fn win_close_btn(ui: &mut egui::Ui, ctx: &Context) {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(46.0, 32.0), Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.painter().rect_filled(rect, 0.0, STATUS_ERROR);
        resp.clone().on_hover_text("Close");
    }
    // Draw X as two diagonal lines
    let cx = rect.center().x;
    let cy = rect.center().y;
    let r = 4.5_f32;
    let stroke_col = if hovered { crate::theme::text_inverse() } else { crate::theme::text_primary() };
    let s = egui::Stroke::new(1.5, stroke_col);
    ui.painter().line_segment([egui::pos2(cx - r, cy - r), egui::pos2(cx + r, cy + r)], s);
    ui.painter().line_segment([egui::pos2(cx + r, cy - r), egui::pos2(cx - r, cy + r)], s);
    if resp.clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

// ─── Dropdown primitives ─────────────────────────────────────────────────────

/// Section header — small, uppercase, dimmed, non-interactive.
fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 18.0),
        Sense::hover(),
    );
    ui.painter().text(
        egui::pos2(rect.left() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(10.0),
        crate::theme::text_tertiary(),
    );
}

/// Ghost menu item — transparent at rest, highlight on hover. Optional right-hand shortcut hint.
fn item(ui: &mut egui::Ui, label: &str, shortcut: &str) -> bool {
    let height = 26.0;
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        Sense::click(),
    );

    if resp.hovered() {
        ui.painter().rect_filled(rect, 2.0, crate::theme::bg_surface_raised());
    }

    // Label
    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        if resp.hovered() { crate::theme::text_primary() } else { crate::theme::text_secondary() },
    );

    // Shortcut
    if !shortcut.is_empty() {
        ui.painter().text(
            egui::pos2(rect.right() - 10.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::proportional(11.0),
            crate::theme::text_tertiary(),
        );
    }

    resp.clicked()
}


/// Thin separator — 1 px line with breathing room.
fn sep(ui: &mut egui::Ui) {
    ui.add_space(3.0);
    ui.painter().line_segment(
        [
            egui::pos2(ui.min_rect().left() + 8.0, ui.cursor().top()),
            egui::pos2(ui.min_rect().right() - 8.0, ui.cursor().top()),
        ],
        egui::Stroke::new(1.0, crate::theme::border_subtle()),
    );
    ui.add_space(4.0);
}

/// Styled submenu — ghost at rest, highlight on hover, nested popup.
fn ghost_submenu(ui: &mut egui::Ui, label: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.hovered.bg_fill = crate::theme::bg_surface_raised();
        ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.inactive.fg_stroke = egui::Stroke::new(1.0, crate::theme::text_secondary());
        ui.visuals_mut().widgets.hovered.fg_stroke = egui::Stroke::new(1.0, crate::theme::text_primary());
        ui.style_mut().spacing.button_padding = egui::vec2(12.0, 6.0);
        ui.menu_button(label, |ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
            ui.visuals_mut().widgets.hovered.bg_fill = crate::theme::bg_surface_raised();
            ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().spacing.item_spacing.y = 0.0;
            add_contents(ui);
        });
    });
}

// ─── File ─────────────────────────────────────────────────────────────────────

fn file_menu(ui: &mut egui::Ui) {
    section(ui, "SERVICE SESSION");
    item(ui, "New Service Session",       "Ctrl+N");
    item(ui, "Open Service Session…",     "Ctrl+O");
    item(ui, "Save",                      "Ctrl+S");
    item(ui, "Save As…",                  "Ctrl+Shift+S");
    item(ui, "Close Session",             "Ctrl+W");
    sep(ui);

    section(ui, "IMPORT");
    item(ui, "Import Sermon Outline…",    "");
    item(ui, "Import Verse List…",        "");
    item(ui, "Import Run Sheet (CSV)…",   "");

    section(ui, "EXPORT");
    item(ui, "Export Session Report…",    "Ctrl+E");
    item(ui, "Export Transcript (TXT)…",  "");
    item(ui, "Export Sermon Log (PDF)…",  "");
    sep(ui);

    section(ui, "RECENT");
    item(ui, "Sunday Morning Service",    "");
    item(ui, "Midweek Bible Study",       "");
    item(ui, "Youth Conference 2026",     "");
    item(ui, "Clear Recent…",             "");
    sep(ui);

    item(ui, "Preferences…",             "Ctrl+,");
    item(ui, "Exit",                      "Alt+F4");
}

// ─── Edit ─────────────────────────────────────────────────────────────────────

fn edit_menu(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    section(ui, "HISTORY");
    if item(ui, "Undo Verse Push",        "Ctrl+Z") { app.undo_verse(); ui.memory_mut(|m| m.close_popup()); }
    item(ui, "Redo",                      "Ctrl+Y");
    sep(ui);

    section(ui, "VERSE ENTRY");
    if item(ui, "Manual Verse Entry…",   "Ctrl+M") { app.focus_manual = true; ui.memory_mut(|m| m.close_popup()); }
    item(ui, "Verse Search…",            "Ctrl+F");
    item(ui, "Approve Current Verse",    "Ctrl+Return");
    item(ui, "Reject Current Verse",     "Escape");
    item(ui, "Edit Verse Text…",         "");
    sep(ui);

    section(ui, "TRANSCRIPT");
    if item(ui, "Clear Transcript",      "") { app.transcript.clear(); ui.memory_mut(|m| m.close_popup()); }
    item(ui, "Copy All Transcript",      "Ctrl+Shift+C");
    item(ui, "Save Transcript…",         "");
    sep(ui);

    item(ui, "Find in Sermon Log…",      "Ctrl+Shift+F");
    item(ui, "Preferences…",             "Ctrl+,");
}

// ─── View ─────────────────────────────────────────────────────────────────────

fn view_menu(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    section(ui, "WORKSPACES");
    ghost_submenu(ui, "  Switch Workspace  ›", |ui| {
        for ws in ["Operator Broadcast","Broadcast Focus","Audio Diagnostics","Minimal"] {
            item(ui, ws, "");
        }
        sep(ui);
        item(ui, "Save Current Layout…", "");
        item(ui, "Custom Layout…", "");
    });
    sep(ui);

    section(ui, "PANELS");
    ghost_submenu(ui, "  Toggle Panels  ›", |ui| {
        for panel in ["Audio Control","Live Verse Stage","Run Sheet",
                      "Sermon Log","Queue","Transcript","Metrics"] {
            ui.checkbox(&mut true, panel);
        }
    });
    sep(ui);

    section(ui, "WINDOWS");
    if item(ui, "Program Out Window", "Ctrl+Shift+P") { app.program_out = !app.program_out; ui.memory_mut(|m| m.close_popup()); }
    item(ui, "Audio Monitor Window",  "");
    item(ui, "LED Status Window",     "");
    item(ui, "Performance Monitor",   "");
    sep(ui);

    section(ui, "DISPLAY");
    item(ui, "Fullscreen",            "F11");
    item(ui, "Zoom In",               "Ctrl++");
    item(ui, "Zoom Out",              "Ctrl+-");
    item(ui, "Reset Zoom",            "Ctrl+0");
    item(ui, "Toggle Status Bar",     "");
    item(ui, "Toggle Command Bar",    "");
}

// ─── Session ──────────────────────────────────────────────────────────────────

fn session_menu(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    section(ui, "TRANSPORT");
    if item(ui, if app.is_listening { "Stop Listening" } else { "Start Listening" }, "F5") {
        app.toggle_listening();
        ui.memory_mut(|m| m.close_popup());
    }
    item(ui, "Pause Capture",         "F6");
    item(ui, "Mute Monitor Output",   "F7");
    sep(ui);

    section(ui, "RECORDING");
    item(ui, "Start Recording",       "Ctrl+R");
    item(ui, "Stop Recording",        "Ctrl+Shift+R");
    item(ui, "Open Recording Folder", "");
    sep(ui);

    section(ui, "SERMON LOG");
    item(ui, "Export Sermon Log…",    "");
    item(ui, "Clear Sermon Log",      "");
    item(ui, "Print Session Report…", "Ctrl+P");
    sep(ui);

    section(ui, "SERVICE");
    item(ui, "Start New Service",     "");
    item(ui, "End Current Service",   "");
    item(ui, "Session Properties…",   "");
}

// ─── Audio ────────────────────────────────────────────────────────────────────

fn audio_menu(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    section(ui, "CAPTURE");
    if item(ui, "Start Listening",    "F5") { if !app.is_listening { app.toggle_listening(); } ui.memory_mut(|m| m.close_popup()); }
    if item(ui, "Stop Listening",     "Shift+F5") { if app.is_listening { app.toggle_listening(); } ui.memory_mut(|m| m.close_popup()); }
    item(ui, "Pause Capture",         "F6");
    sep(ui);

    section(ui, "INPUT DEVICE");
    ghost_submenu(ui, "  Select Input Device  ›", |ui| {
        for dev in app.devices.clone() {
            if ui.selectable_label(app.selected_device == dev, &dev).clicked() {
                app.selected_device = dev.clone();
                app.pipeline.send_command(hv_pipeline::PipelineCommand::SetDevice(dev));
                ui.close_menu();
            }
        }
    });
    item(ui, "Refresh Device List",   "");
    item(ui, "Device Info…",          "");
    sep(ui);

    section(ui, "CALIBRATION");
    item(ui, "Auto-Calibrate Gain",   "");
    item(ui, "Run Noise Floor Test",  "");
    item(ui, "Reset All Gain Staging","");
    sep(ui);

    section(ui, "MONITORING");
    item(ui, "Audio Monitor Window",  "");
    item(ui, "Play 1 kHz Test Tone",  "");
    item(ui, "Mute Monitor",          "F7");
    sep(ui);

    item(ui, "Audio Settings…",       "");
}

// ─── ASR ──────────────────────────────────────────────────────────────────────

fn asr_menu(ui: &mut egui::Ui) {
    section(ui, "ENGINES");
    item(ui, "Model Manager…",        "");
    ghost_submenu(ui, "  Primary Engine  ›", |ui| {
        for e in ["Zipformer (CPU)","Zipformer (GPU)","Whisper Large","Mock / Debug"] {
            let _ = ui.selectable_label(e == "Zipformer (CPU)", e);
        }
    });
    ghost_submenu(ui, "  Fallback Engine  ›", |ui| {
        for e in ["None","Zipformer Tiny","System Speech API"] {
            let _ = ui.selectable_label(e == "None", e);
        }
    });
    sep(ui);

    section(ui, "ACCELERATION");
    item(ui, "GPU (CUDA)",            "");
    item(ui, "GPU (DirectML)",        "");
    item(ui, "CPU Safe Mode",         "");
    sep(ui);

    section(ui, "DIAGNOSTICS");
    item(ui, "Test ASR — Live Mic",   "");
    item(ui, "Test ASR — Audio File…","");
    item(ui, "View ASR Latency Log",  "");
    item(ui, "Reset ASR State",       "");
    sep(ui);

    item(ui, "ASR Settings…",         "");
}

// ─── Bible ────────────────────────────────────────────────────────────────────

fn bible_menu(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    section(ui, "TRANSLATIONS");
    item(ui, "Translation Manager…",  "");
    ghost_submenu(ui, "  Active Translation  ›", |ui| {
        for t in ["KJV","NIV","ESV","NASB","NLT","NKJV","MSG","AMP"] {
            let _ = ui.selectable_label(app.translation == t, t);
        }
    });
    item(ui, "Download Translation…", "");
    sep(ui);

    section(ui, "VERSE TOOLS");
    item(ui, "Verse Search…",         "Ctrl+F");
    if item(ui, "Manual Entry…",      "Ctrl+M") { app.focus_manual = true; ui.memory_mut(|m| m.close_popup()); }
    item(ui, "Cross-Reference…",      "");
    item(ui, "Verse Comparison…",     "");
    sep(ui);

    section(ui, "CONFIGURATION");
    item(ui, "Book Alias Editor…",    "");
    item(ui, "Detection Settings…",   "");
    item(ui, "Bible Cache…",          "");
    item(ui, "Bible Settings…",       "");
}

// ─── Broadcast ────────────────────────────────────────────────────────────────

fn broadcast_menu(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    section(ui, "PROGRAM OUT");
    if item(ui, "Open Program Out Window", "Ctrl+Shift+P") { app.program_out = !app.program_out; ui.memory_mut(|m| m.close_popup()); }
    item(ui, "Go Live",               "");
    item(ui, "Emergency: Clear Output","");
    item(ui, "Fallback Graphic…",     "");
    sep(ui);

    section(ui, "OUTPUT TARGET");
    ghost_submenu(ui, "  Output Mode  ›", |ui| {
        for mode in ["Window","NDI","SDI (Blackmagic)","ProPresenter","OBS Studio"] {
            let _ = ui.selectable_label(mode == "Window", mode);
        }
    });
    item(ui, "NDI Settings…",         "");
    item(ui, "ProPresenter Link…",    "");
    item(ui, "OBS Studio Link…",      "");
    sep(ui);

    section(ui, "GRAPHICS");
    item(ui, "Theme Editor…",         "");
    item(ui, "Font Settings…",        "");
    item(ui, "Logo & Watermark…",     "");
    item(ui, "Transition Settings…",  "");
    sep(ui);

    item(ui, "Broadcast Settings…",   "");
}

// ─── Tools ────────────────────────────────────────────────────────────────────

fn tools_menu(ui: &mut egui::Ui) {
    section(ui, "SESSION TOOLS");
    item(ui, "Service Session Manager…","");
    item(ui, "Run Sheet Editor…",     "");
    item(ui, "Sermon Outline Builder…","");
    sep(ui);

    section(ui, "HARDWARE");
    item(ui, "LED Dashboard…",        "");
    item(ui, "USB LED Controller…",   "");
    item(ui, "MIDI Controller Setup…","");
    sep(ui);

    section(ui, "DIAGNOSTICS");
    item(ui, "Diagnostics…",          "");
    item(ui, "Performance Monitor…",  "");
    item(ui, "Audio Thread Inspector","");
    item(ui, "ASR Latency Log",       "");
    sep(ui);

    section(ui, "CUSTOMIZATION");
    item(ui, "Keyboard Shortcuts…",   "Ctrl+K, Ctrl+S");
    item(ui, "Workspace Presets…",    "");
    item(ui, "Plugin Manager…",       "");
    sep(ui);

    item(ui, "Command Palette…",      "Ctrl+Shift+P");
    item(ui, "Reset to Defaults",     "");
}

// ─── Window ───────────────────────────────────────────────────────────────────

fn window_menu(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    section(ui, "ARRANGE");
    item(ui, "Minimize",              "");
    item(ui, if app.window_maximized { "Restore Down" } else { "Maximize" }, "");
    item(ui, "Tile Windows",          "");
    item(ui, "Bring All to Front",    "");
    sep(ui);

    section(ui, "OPEN WINDOWS");
    item(ui, "Main Window",           "");
    item(ui, "Program Out",           "");
    item(ui, "Audio Monitor",         "");
    item(ui, "LED Status",            "");
    item(ui, "Transcript Log",        "");
    item(ui, "Performance Monitor",   "");
}

// ─── Help ─────────────────────────────────────────────────────────────────────

fn help_menu(ui: &mut egui::Ui) {
    section(ui, "DOCUMENTATION");
    item(ui, "Quick Start Guide",     "");
    item(ui, "User Manual",           "F1");
    item(ui, "Keyboard Shortcuts",    "Ctrl+K, Ctrl+S");
    item(ui, "API / Developer Docs",  "");
    sep(ui);

    section(ui, "SUPPORT");
    item(ui, "Check for Updates…",    "");
    item(ui, "Report a Bug…",         "");
    item(ui, "Request a Feature…",    "");
    item(ui, "Join Community Forum",  "");
    sep(ui);

    section(ui, "ABOUT");
    item(ui, "Release Notes",         "");
    item(ui, "Licenses",              "");
    item(ui, "About HV-Bible Desktop","");
}
