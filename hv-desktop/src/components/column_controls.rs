//! The small controls the left column is assembled from.
//!
//! Hand-drawn rather than wrapped egui widgets: a `Checkbox` and a `Button`
//! are sized for a settings form, not for a channel strip. Everything here
//! lays itself out against the available width, so nothing in this module can
//! hardcode a width and cause the column to overflow or grow a scrollbar.

use eframe::egui::{
    self, Align2, Color32, FontId, Galley, Rect, Response, Sense, Stroke, TextStyle, Ui, Vec2,
};
use std::sync::Arc;

use super::column_density::{ColumnDensity, UNIT};

/// The type scale for anything in a strip that is not a raw number: 0.92 of
/// the current small-text height, so a strip tracks the global zoom setting.
pub fn strip_size(ui: &Ui) -> f32 {
    ui.text_style_height(&TextStyle::Small) * 0.92
}

/// The body font, resolved to a `FontId` for the painter's raw text calls.
pub fn body_font(ui: &Ui) -> FontId {
    FontId::proportional(ui.text_style_height(&TextStyle::Body))
}

/// Lays out one line of text, clipped with an ellipsis when it does not fit.
///
/// `monospace` keeps a numeric column from twitching as its digits change,
/// which is the whole reason the dB readouts use it.
pub fn elide(ui: &Ui, text: &str, width: f32, color: Color32, monospace: bool) -> Arc<Galley> {
    let size = strip_size(ui);
    let font = if monospace {
        FontId::monospace(size)
    } else {
        FontId::proportional(size)
    };

    if measure(ui, text, &font, color) <= width {
        return layout(ui, &text.to_string(), &font, color);
    }

    // Binary-search the longest prefix that still leaves room for the ellipsis.
    // A linear walk would re-measure the whole string once per character.
    let budget = width - measure(ui, "…", &font, color);
    let chars: Vec<char> = text.chars().collect();
    let mut low = 0usize;
    let mut high = chars.len();
    while low < high {
        let mid = (low + high + 1) / 2;
        let candidate: String = chars[..mid].iter().collect();
        if measure(ui, &candidate, &font, color) <= budget {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    let mut kept: String = chars[..low].iter().collect();
    kept.push('…');
    layout(ui, &kept, &font, color)
}

/// Lays text out on a single line with no wrapping constraint.
fn layout(ui: &Ui, text: &str, font: &FontId, color: Color32) -> Arc<Galley> {
    ui.painter()
        .fonts(|fonts| fonts.layout(text.to_string(), font.clone(), color, f32::INFINITY))
}

/// Width of a single line of text, in points.
fn measure(ui: &Ui, text: &str, font: &FontId, color: Color32) -> f32 {
    layout(ui, text, font, color).size().x
}

/// One line of clipped text across the full available width.
pub fn line(ui: &mut Ui, text: &str, color: Color32, monospace: bool) {
    let width = ui.available_width();
    let height = strip_size(ui) + UNIT;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter().galley(
        egui::pos2(rect.left(), rect.center().y),
        elide(ui, text, rect.width(), color, monospace),
        color,
    );
}

/// Label on the left, value on the right, both clipped. The workhorse row.
pub fn readout(ui: &mut Ui, label: &str, value: &str) {
    let width = ui.available_width();
    let height = strip_size(ui) + UNIT;
    // The value gets a proportional share, floored so a number is never
    // squeezed out of existence, and capped so the label keeps a readable
    // minimum at wide sizes.
    let value_width = (width * 0.42).clamp(UNIT * 7.0, UNIT * 24.0);
    let label_width = (width - value_width).max(0.0);

    let (label_rect, _) = ui.allocate_exact_size(Vec2::new(label_width, height), Sense::hover());
    ui.painter().galley(
        egui::pos2(label_rect.left(), label_rect.center().y),
        elide(ui, label, label_width, crate::theme::text_secondary(), false),
        crate::theme::text_secondary(),
    );

    let (value_rect, _) = ui.allocate_exact_size(Vec2::new(value_width, height), Sense::hover());
    ui.painter().galley(
        egui::pos2(value_rect.left(), value_rect.center().y),
        elide(
            ui,
            value,
            value_rect.width(),
            crate::theme::text_primary(),
            true,
        ),
        crate::theme::text_primary(),
    );
}


/// A horizontal bar with a filled portion, 0.0 – 1.0. Used for the VAD
/// confidence readout. Sized to whatever the caller hands it, never its own.
pub fn value_bar(ui: &mut Ui, fraction: f32, color: Color32, height: f32) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, crate::theme::bg_surface_sunken());
    let filled = Rect::from_min_size(
        rect.min,
        Vec2::new(rect.width() * fraction.clamp(0.0, 1.0), rect.height()),
    );
    painter.rect_filled(filled, 2.0, color);
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, crate::theme::border_subtle()));
}

/// Splits the available width into `count` equal columns with `gap` between
/// them, guaranteeing the total never exceeds what was given. Shared by every
/// row that lays out proportionally rather than by hardcoded pixels.
pub fn column_width(available: f32, count: usize, gap: f32) -> f32 {
    let count = count.max(1) as f32;
    ((available - gap * (count - 1.0)) / count).max(UNIT * 3.0)
}

/// The accent dimmed toward the sunken surface, for a filled "on" track.
fn accent_track(accent: Color32, amount: f32) -> Color32 {
    let surface = crate::theme::bg_surface_sunken();
    Color32::from_rgba_unmultiplied(
        lerp(surface.r(), accent.r(), amount),
        lerp(surface.g(), accent.g(), amount),
        lerp(surface.b(), accent.b(), amount),
        255,
    )
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

/// A compact on/off control drawn like a console switch.
///
/// egui's `Checkbox` is a checkbox: a square, a tick, and a text label. At
/// 200 px there is no room for either, and at 400 px it still reads as a
/// form. This is a 26×14 track with a knob, which is what a hardware desk uses.
pub struct InlineToggle<'a> {
    pub label: &'a str,
    pub value: &'a mut bool,
    pub density: ColumnDensity,
}

impl<'a> InlineToggle<'a> {
    pub fn new(label: &'a str, value: &'a mut bool, density: ColumnDensity) -> Self {
        Self {
            label,
            value,
            density,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = ui.available_width();
        let height = self.density.row_height();
        let track = Vec2::new(UNIT * 6.5, UNIT * 3.5);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        if response.clicked() {
            *self.value = !*self.value;
        }

        let track_rect = Rect::from_center_size(
            egui::pos2(rect.left() + track.x / 2.0, rect.center().y),
            track,
        );
        let on = *self.value;
        let accent = crate::theme::accent();
        let painter = ui.painter();
        painter.rect_filled(
            track_rect,
            track.y / 2.0,
            if on {
                accent_track(accent, 0.55)
            } else {
                crate::theme::bg_surface_sunken()
            },
        );
        painter.rect_stroke(
            track_rect,
            track.y / 2.0,
            Stroke::new(
                1.0,
                if on {
                    accent
                } else {
                    crate::theme::border_strong()
                },
            ),
        );

        let radius = track.y / 2.0 - UNIT * 0.5;
        let knob_x = if on {
            track_rect.right() - radius - 1.0
        } else {
            track_rect.left() + radius + 1.0
        };
        painter.circle_filled(
            egui::pos2(knob_x, track_rect.center().y),
            radius,
            if on {
                crate::theme::text_primary()
            } else {
                crate::theme::text_tertiary()
            },
        );

        if self.density.shows_labels() {
            let text_left = track_rect.right() + UNIT * 2.0;
            let text_width = (rect.right() - text_left).max(0.0);
            if text_width > UNIT {
                let color = if on {
                    crate::theme::text_primary()
                } else {
                    crate::theme::text_secondary()
                };
                painter.galley(
                    egui::pos2(text_left, rect.center().y),
                    elide(ui, self.label, text_width, color, false),
                    color,
                );
            }
        }

        response.on_hover_text(self.label)
    }
}

/// A small flat button that takes the full available width. Its label elides,
/// and at icon-only density the label is dropped entirely.
pub struct InlineButton<'a> {
    pub label: &'a str,
    pub icon: Option<&'a str>,
    pub density: ColumnDensity,
    pub active: bool,
    pub danger: bool,
}

impl<'a> InlineButton<'a> {
    pub fn new(label: &'a str, density: ColumnDensity) -> Self {
        Self {
            label,
            icon: None,
            density,
            active: false,
            danger: false,
        }
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = ui.available_width();
        let height = self.density.row_height();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        let (fill, stroke, text_color) = if self.danger {
            (
                crate::theme::STATUS_ERROR.linear_multiply(0.18),
                crate::theme::STATUS_ERROR,
                crate::theme::STATUS_ERROR,
            )
        } else if self.active {
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

        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(rect, 3.0, fill.linear_multiply(1.25));
        } else {
            painter.rect_filled(rect, 3.0, fill);
        }
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, stroke));

        // The icon always survives, even with no room for a word: an icon-only
        // column still has to be operable.
        let mut text = String::new();
        if let Some(icon) = self.icon {
            text.push_str(icon);
        }
        if self.density.shows_labels() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(self.label);
        }
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(ui.text_style_height(&TextStyle::Button)),
            text_color,
        );
        response
    }
}


/// A filled dot. `radius` follows the density so the icon-only column still
/// shows a readable state light.
pub fn status_dot(ui: &mut Ui, color: Color32, radius: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(radius * 2.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), radius, color);
    response
}

