//! ChannelMeter — one horizontal level strip, Pro Tools style.
//!
//! Draws into whatever width it is handed and nothing wider, which is what
//! keeps a horizontal scrollbar from ever appearing in the left column. The
//! scale, the peak-hold tick, the clip light, and the numeric readout are all
//! drawn by hand so they line up on the pixel grid instead of drifting with
//! widget metrics.

use eframe::egui::{self, Color32, Rect, Sense, Stroke, Ui, Vec2};

use crate::components::column_controls::{body_font, elide, strip_size};
use crate::components::column_density::{ColumnDensity, UNIT};

/// Bottom of the meter scale, in dBFS. Matches the pipeline's meter range.
pub const SCALE_FLOOR_DB: f32 = -48.0;
/// Metres the strip occupies vertically.
const STRIP_HEIGHT: f32 = UNIT * 2.5;
/// dB marks drawn on the scale.
const MARKS: [f32; 4] = [-48.0, -24.0, -12.0, -6.0];

/// Maps dBFS onto 0.0 – 1.0 along the strip.
pub fn db_to_fraction(db: f32) -> f32 {
    ((db - SCALE_FLOOR_DB) / -SCALE_FLOOR_DB).clamp(0.0, 1.0)
}

/// Meter colour by headroom: green below -12 dB, amber to -3 dB, red above.
/// The same mapping the rest of the application uses for levels.
pub fn level_color(db: f32) -> Color32 {
    if db > -3.0 {
        crate::theme::STATUS_ERROR
    } else if db > -12.0 {
        crate::theme::STATUS_WARNING
    } else {
        crate::theme::STATUS_SUCCESS
    }
}

pub struct ChannelMeter<'a> {
    pub channel: &'a str,
    pub rms_db: f32,
    pub peak_hold_db: f32,
    pub clipped: bool,
    pub density: ColumnDensity,
}

impl<'a> ChannelMeter<'a> {
    pub fn show(self, ui: &mut Ui) {
        let width = ui.available_width();
        let height = self.density.row_height().max(STRIP_HEIGHT + UNIT * 2.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return;
        }

        let label_width = if self.density.shows_labels() {
            strip_size(ui) * 1.4
        } else {
            0.0
        };
        let clip_width = if self.density.shows_labels() {
            UNIT * 2.0
        } else {
            0.0
        };
        // The numeric readout is the one fixed-feeling element in the column,
        // but it is a share of the width, not a constant: it grows and shrinks
        // with the column like everything else.
        let value_width = (width * 0.26).clamp(UNIT * 5.0, UNIT * 11.0);
        let strip_x = rect.left() + label_width + UNIT;
        let strip_width = (rect.right() - clip_width - value_width - UNIT - strip_x)
            .max(UNIT * 3.0);

        let painter = ui.painter();

        if self.density.shows_labels() {
            painter.text(
                egui::pos2(rect.left(), rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.channel,
                body_font(ui),
                crate::theme::text_secondary(),
            );
        }

        let strip = Rect::from_min_size(
            egui::pos2(strip_x, rect.center().y - STRIP_HEIGHT / 2.0),
            Vec2::new(strip_width, STRIP_HEIGHT),
        );
        self.paint_strip(painter, strip);


        // Peak-hold tick: a 2 px mark that sits where the level got to and
        // falls slowly, so a spike the operator missed is still readable.
        if self.peak_hold_db > SCALE_FLOOR_DB {
            let x = strip.left() + strip.width() * db_to_fraction(self.peak_hold_db);
            painter.rect_filled(
                Rect::from_min_size(
                    egui::pos2(x - 1.0, strip.top() - 1.0),
                    Vec2::new(2.0, strip.height() + 2.0),
                ),
                0.0,
                if self.clipped {
                    crate::theme::STATUS_ERROR
                } else {
                    crate::theme::text_primary()
                },
            );
        }

        // Clip light. Stays lit until the operator clears it, which is the
        // difference between "clipping right now" and "you clipped".
        if self.density.shows_labels() {
            let center = egui::pos2(rect.right() - clip_width / 2.0, rect.center().y);
            if self.clipped {
                painter.circle_filled(center, UNIT * 0.9, crate::theme::STATUS_ERROR);
            } else {
                painter.circle_stroke(
                    center,
                    UNIT * 0.9,
                    Stroke::new(1.0, crate::theme::border_subtle()),
                );
            }
        }

        let value_x = rect.right() - clip_width - value_width;
        let value = format!("{:+.1}", self.rms_db);
        painter.galley(
            egui::pos2(value_x, rect.center().y),
            elide(ui, &value, value_width, level_color(self.rms_db), true),
            level_color(self.rms_db),
        );
    }

    fn paint_strip(&self, painter: &egui::Painter, strip: Rect) {
        painter.rect_filled(strip, 1.0, crate::theme::bg_surface_sunken());

        let fraction = db_to_fraction(self.rms_db);
        if fraction > 0.0 {
            painter.rect_filled(
                Rect::from_min_size(
                    strip.min,
                    Vec2::new(strip.width() * fraction, strip.height()),
                ),
                1.0,
                level_color(self.rms_db),
            );
        }

        // Scale marks, thinned out when the strip is too narrow for all of them
        // to mean anything.
        let step = if strip.width() < UNIT * 20.0 { 2 } else { 1 };
        for (index, db) in MARKS.iter().enumerate() {
            if index % step != 0 {
                continue;
            }
            let x = strip.left() + strip.width() * db_to_fraction(*db);
            painter.line_segment(
                [egui::pos2(x, strip.top()), egui::pos2(x, strip.bottom())],
                Stroke::new(1.0, crate::theme::border_subtle()),
            );
        }

        painter.rect_stroke(
            strip,
            1.0,
            Stroke::new(1.0, crate::theme::border_subtle()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{db_to_fraction, level_color, SCALE_FLOOR_DB};

    #[test]
    fn the_scale_spans_the_pipelines_meter_range() {
        assert_eq!(db_to_fraction(SCALE_FLOOR_DB), 0.0);
        assert_eq!(db_to_fraction(0.0), 1.0);
        // Out-of-range input must clamp, never draw outside the strip.
        assert_eq!(db_to_fraction(-90.0), 0.0);
        assert_eq!(db_to_fraction(12.0), 1.0);
    }

    #[test]
    fn the_scale_rises_monotonically() {
        let mut previous = 0.0;
        let mut db = -48.0;
        while db <= 0.0 {
            let fraction = db_to_fraction(db);
            assert!(fraction >= previous, "scale went backwards at {db} dB");
            previous = fraction;
            db += 0.5;
        }
    }

    #[test]
    fn meter_colour_follows_headroom() {
        assert_eq!(level_color(-30.0), crate::theme::STATUS_SUCCESS);
        assert_eq!(level_color(-6.0), crate::theme::STATUS_WARNING);
        assert_eq!(level_color(-1.0), crate::theme::STATUS_ERROR);
    }
}
