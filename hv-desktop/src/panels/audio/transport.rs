//! transport.rs — the pinned bar at the foot of the left column.
//!
//! The only part of the column that never scrolls. Pause, Stop, Mute, and the
//! gear that opens Settings on the Audio page. Everything an operator must be
//! able to hit without scrolling: if a control is not here or in a section
//! above, it is not needed during a service.

use eframe::egui::{self, Rect, Sense, Stroke, Ui, Vec2};

use crate::app::HvBibleApp;
use crate::components::column_controls::{strip_size};
use crate::components::column_density::{ColumnDensity, UNIT};

/// Bar height: 2 × the 4 px base unit of padding plus one row.
fn bar_height(density: ColumnDensity) -> f32 {
    density.row_height() + UNIT * 3.0
}

pub fn show(ui: &mut Ui, app: &mut HvBibleApp, density: ColumnDensity) {
    let height = bar_height(density);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, crate::theme::bg_surface_raised());
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, crate::theme::border_strong()),
    );

    // Four controls share the width equally, so the bar never depends on a
    // fixed button size and never overflows at 180 px.
    let count = 4;
    let gap = UNIT;
    let button_width = ((rect.width() - gap * (count - 1) as f32) / count as f32).max(UNIT * 4.0);
    let mut x = rect.left();

    let mut pressed = Pressed::default();
    for index in 0..count {
        let button = Rect::from_min_size(
            egui::pos2(x, rect.top() + UNIT),
            Vec2::new(button_width, rect.height() - UNIT * 2.0),
        );
        if transport_button(ui, painter, &button, index, app, density) {
            pressed = match index {
                0 => Pressed::Pause,
                1 => Pressed::Stop,
                2 => Pressed::Mute,
                _ => Pressed::Settings,
            };
        }
        x += button_width + gap;
    }

    // Act after the loop so the four borrows of `app` in `transport_button` have
    // all ended before a method takes `&mut app`.
    match pressed {
        Pressed::Pause => app.toggle_pause(),
        Pressed::Stop => app.stop_capture(),
        Pressed::Mute => {
            app.audio_mock.monitoring.muted = !app.audio_mock.monitoring.muted;
        }
        Pressed::Settings => app.open_audio_settings(),
        Pressed::None => {}
    }
}

/// Which control the operator just hit. Collected during the row so the actions
/// can run after every borrow has ended.
#[derive(Default, Clone, Copy)]
enum Pressed {
    #[default]
    None,
    Pause,
    Stop,
    Mute,
    Settings,
}

fn transport_button(
    ui: &Ui,
    painter: &egui::Painter,
    rect: &Rect,
    index: usize,
    app: &HvBibleApp,
    density: ColumnDensity,
) -> bool {
    let (icon, label, active, danger, tooltip) = match index {
        0 => (
            if app.is_paused { "▶" } else { "❚❚" },
            "Pause",
            app.is_paused,
            false,
            if app.is_paused {
                "Resume recognition"
            } else {
                "Hold the chain without closing the device"
            },
        ),
        1 => ("■", "Stop", false, true, "Close the capture device"),
        2 => (
            if app.audio_mock.monitoring.muted { "🔇" } else { "🔊" },
            "Mute",
            app.audio_mock.monitoring.muted,
            false,
            "Mute the monitor feed",
        ),
        _ => ("⚙", "Settings", false, false, "Open Settings on the Audio page"),
    };

    let response = ui.interact(*rect, ui.make_persistent_id(("transport", index)), Sense::click());
    if response.hovered() {
        painter.rect_filled(*rect, 3.0, crate::theme::bg_surface_raised().linear_multiply(1.2));
    } else {
        painter.rect_filled(*rect, 3.0, crate::theme::bg_surface_sunken());
    }

    let (fill, stroke, text_color) = if danger {
        (
            crate::theme::STATUS_ERROR.linear_multiply(0.16),
            crate::theme::STATUS_ERROR.linear_multiply(0.7),
            crate::theme::STATUS_ERROR,
        )
    } else if active {
        (
            crate::theme::accent_muted(),
            crate::theme::accent(),
            crate::theme::accent(),
        )
    } else {
        (
            crate::theme::bg_surface_sunken(),
            crate::theme::border_subtle(),
            crate::theme::text_secondary(),
        )
    };
    painter.rect_filled(*rect, 3.0, fill);
    painter.rect_stroke(*rect, 3.0, Stroke::new(1.0, stroke));

    let font = egui::FontId::proportional(strip_size(ui));
    if density.shows_labels() {
        // Icon over label, stacked, so the button reads the same at 200 px and
        // at 640 px instead of turning into a strip of letters.
        let icon_y = rect.top() + (rect.height() * 0.38).min(strip_size(ui) * 0.9);
        let label_y = rect.center().y + strip_size(ui) * 0.85;
        painter.text(
            egui::pos2(rect.center().x, icon_y),
            egui::Align2::CENTER_CENTER,
            icon,
            font.clone(),
            text_color,
        );
        painter.text(
            egui::pos2(rect.center().x, label_y),
            egui::Align2::CENTER_CENTER,
            label,
            font,
            text_color,
        );
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            font,
            text_color,
        );
    }

    response.on_hover_text(tooltip).clicked()
}
