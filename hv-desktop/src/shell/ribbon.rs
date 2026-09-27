//! Studio Deck — the premium two-tier command bar for the broadcast operator.
//!
//! The legacy icon-card ribbon is gone. The deck is built from three ideas:
//!
//! * **Rail** — the brand lockup, a segmented tab switcher with a gliding accent
//!   underline, and a right-hand live cluster (zoom, latency, program out, state).
//! * **Deck** — contextual command clusters. Each cluster is a row of command
//!   tiles with its group caption placed *underneath* the row, so the commands —
//!   not the chrome — stay the hero of the bar.
//! * **Scale** — every measurement derives from the active theme's base font size,
//!   so the deck breathes with the rest of the interface (Ctrl+= / Ctrl+-).
//!
//! Tiles are painted by hand (dots, transport shapes, hairlines) and the rail is
//! pure typography, so the deck never depends on emoji or icon fonts and looks
//! identical on every machine.

use eframe::egui::{
    Align, Align2, Color32, Context, FontId, Layout, Rect, Response, RichText, Rounding, Sense,
    Stroke, Ui, UiBuilder, Vec2,
};
use hv_pipeline::PipelineState;

use crate::app::HvBibleApp;
use crate::theme::{STATUS_ERROR, STATUS_INFO, STATUS_SUCCESS, STATUS_WARNING};

/// Height of the custom title bar the collapsed deck tucks under.
const TITLE_BAR_H: f32 = 32.0;

/// Bible translations offered by the deck capsules.
const TRANSLATIONS: [&str; 8] = ["KJV", "NIV", "ESV", "NASB", "NLT", "NKJV", "MSG", "AMP"];

/// Rail tabs: label plus the tooltip shown on hover.
pub const DECK_TABS: [(&str, &str); 5] = [
    ("CONSOLE", "Transport, approvals and program routing"),
    ("BROADCAST", "On-air tally, overlays and output timing"),
    ("BIBLE", "Reference lookup, translations and reading size"),
    ("AUDIO", "Capture device, gain staging and signal chain"),
    ("TOOLS", "Dock visibility, session and layout controls"),
];

// ─── Persistent state ───────────────────────────────────────────────────────

#[derive(Default, Clone)]
pub struct RibbonState {
    pub active_tab: usize,
}

// ─── Metrics ────────────────────────────────────────────────────────────────

/// Every dimension of the deck, derived from the theme's base font size.
///
/// Keeping this in one place means the rail, the tiles and the capsules always
/// stay in proportion — no matter how the operator scales the interface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deck {
    pub base_px: f32,
    pub rail_h: f32,
    pub deck_h: f32,
    pub tile_h: f32,
    pub pad: f32,
    pub gap: f32,
}

impl Deck {
    const MIN_BASE_PX: f32 = 9.0;
    const MAX_BASE_PX: f32 = 34.0;

    pub fn new(base_px: f32) -> Self {
        let base_px = base_px.clamp(Self::MIN_BASE_PX, Self::MAX_BASE_PX);
        Self {
            base_px,
            rail_h: (base_px * 2.2).max(30.0).round(),
            deck_h: (base_px * 3.9).max(54.0).round(),
            tile_h: (base_px * 2.15).max(30.0).round(),
            pad: (base_px * 0.85).max(10.0).round(),
            gap: (base_px * 0.45).max(6.0).round(),
        }
    }

    pub fn from_theme(app: &HvBibleApp) -> Self {
        Self::new(app.current_theme.font_config.size.to_pixels())
    }

    pub fn total_h(&self) -> f32 {
        self.rail_h + self.deck_h
    }

    /// Corner radius shared by the rail, the tiles and the capsules.
    fn radius(&self) -> f32 {
        (self.base_px * 0.38).clamp(3.0, 8.0).round()
    }

    /// Themed font size helper: `factor` × base font size, never below 8.5 px.
    fn text(&self, factor: f32) -> f32 {
        (self.base_px * factor).max(8.5)
    }

    fn tile_label_px(&self) -> f32 {
        self.text(0.88)
    }

    fn caption_px(&self) -> f32 {
        self.text(0.62)
    }

    fn rail_label_px(&self) -> f32 {
        self.text(0.8)
    }

    fn rail_chip_h(&self) -> f32 {
        (self.rail_h - self.base_px * 0.55).max(16.0).round()
    }

    fn capsule_pad(&self) -> f32 {
        (self.base_px * 0.62).max(7.0).round()
    }

    /// Height reserved for a cluster caption plus the gap above it.
    fn caption_block_h(&self) -> f32 {
        self.base_px.max(12.0) + 4.0
    }

    /// Vertical offset that centres a cluster inside the deck row.
    fn top_pad(&self) -> f32 {
        ((self.deck_h - (self.tile_h + self.caption_block_h())) / 2.0).max(0.0)
    }
}

/// Interface zoom stepper (0.5× – 2.0× in 0.1× steps).
pub fn stepped_scale(current: f32, up: bool) -> f32 {
    let delta = if up { 0.1 } else { -0.1 };
    (((current + delta) * 10.0).round() / 10.0).clamp(0.5, 2.0)
}

/// Base text stepper (10 – 32 px in 1 px steps).
pub fn stepped_font_px(current: f32, up: bool) -> f32 {
    let delta = if up { 1.0 } else { -1.0 };
    (current + delta).clamp(10.0, 32.0).round()
}

// ─── Entry point ────────────────────────────────────────────────────────────

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    let deck = Deck::from_theme(app);
    let sid = egui::Id::new("hvb_studio_deck");
    let mut state: RibbonState = ctx.data_mut(|d| d.get_temp(sid).unwrap_or_default());
    state.active_tab = state.active_tab.min(DECK_TABS.len() - 1);

    if app.config.layout_state.ribbon_collapsed {
        collapsed_strip(ctx, app, &deck);
        ctx.data_mut(|d| d.insert_temp(sid, state));
        return;
    }

    egui::TopBottomPanel::top("studio_deck")
        .exact_height(deck.total_h())
        .frame(egui::Frame::none().fill(crate::theme::bg_surface()))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            rail(ui, app, &deck, &mut state);
            command_deck(ui, app, &deck, state.active_tab);
        });

    ctx.data_mut(|d| d.insert_temp(sid, state));
}

/// Slim accent strip shown while the deck is collapsed; click to expand again.
fn collapsed_strip(ctx: &Context, app: &mut HvBibleApp, deck: &Deck) {
    let height = (deck.base_px * 0.44).max(5.0).round();
    let screen = ctx.input(|i| i.screen_rect());
    egui::Area::new(egui::Id::new("deck_restore_strip"))
        .fixed_pos(egui::pos2(screen.left(), screen.top() + TITLE_BAR_H))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(screen.width(), height), Sense::click());
            let hovered = response.hovered();
            let rest = mix(
                crate::theme::accent_muted(),
                crate::theme::bg_surface(),
                0.45,
            );
            ui.painter().rect_filled(
                rect,
                0.0,
                if hovered {
                    crate::theme::accent_muted()
                } else {
                    rest
                },
            );

            let notch = Rect::from_center_size(
                rect.center(),
                Vec2::new((screen.width() * 0.1).min(180.0), 2.0),
            );
            ui.painter().rect_filled(
                notch,
                Rounding::same(1.0),
                if hovered {
                    crate::theme::accent()
                } else {
                    crate::theme::accent_hover().linear_multiply(0.8)
                },
            );

            if response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text("Expand the studio deck")
                .clicked()
            {
                app.config.layout_state.ribbon_collapsed = false;
            }
        });
}

// ─── Rail ───────────────────────────────────────────────────────────────────

fn rail(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck, state: &mut RibbonState) {
    let height = deck.rail_h;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, crate::theme::bg_surface_sunken());
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    let mut rail = ui.new_child(
        UiBuilder::new()
            .id_salt("deck_rail")
            .max_rect(rect.shrink2(Vec2::new(deck.pad, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    rail.spacing_mut().item_spacing = Vec2::ZERO;

    brand(&mut rail, deck, height);
    v_sep(&mut rail, deck, deck.rail_chip_h());
    if tab_switcher(&mut rail, deck, &mut state.active_tab) {
        rail.ctx().memory_mut(|memory| memory.close_popup());
    }
    rail_status_cluster(&mut rail, app, deck);
}

/// Monogram tile + two-line wordmark: the deck's brand lockup.
fn brand(ui: &mut Ui, deck: &Deck, height: f32) {
    let title_font = FontId::proportional(deck.text(0.78));
    let sub_font = FontId::proportional(deck.caption_px());
    let title_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap("HV BIBLE".to_owned(), title_font.clone(), Color32::WHITE)
            .size()
            .x
    });
    let sub_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap("STUDIO DECK".to_owned(), sub_font.clone(), Color32::WHITE)
            .size()
            .x
    });

    let monogram = (height * 0.68).round();
    let text_w = title_w.max(sub_w);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(monogram + 8.0 + text_w, height), Sense::click());

    let tile = Rect::from_center_size(
        egui::pos2(rect.left() + monogram / 2.0, rect.center().y),
        Vec2::splat(monogram),
    );
    ui.painter()
        .rect_filled(tile, Rounding::same(deck.radius()), crate::theme::accent());
    ui.painter().text(
        tile.center(),
        Align2::CENTER_CENTER,
        "HV",
        FontId::proportional(monogram * 0.44),
        crate::theme::text_inverse(),
    );

    let left = rect.left() + monogram + 8.0;
    ui.painter().text(
        egui::pos2(left, rect.center().y - deck.text(0.42)),
        Align2::LEFT_CENTER,
        "HV BIBLE",
        title_font,
        crate::theme::text_primary(),
    );
    ui.painter().text(
        egui::pos2(left, rect.center().y + deck.text(0.5)),
        Align2::LEFT_CENTER,
        "STUDIO DECK",
        sub_font,
        crate::theme::text_tertiary(),
    );

    let _ = response.on_hover_text("HV-Bible Broadcast Engine");
}

/// Hairline separator used between rail clusters.
fn v_sep(ui: &mut Ui, deck: &Deck, height: f32) {
    let width = (deck.base_px * 1.4).max(16.0).round();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let inset = height * 0.22;
    ui.painter().line_segment(
        [
            egui::pos2(rect.center().x, rect.top() + inset),
            egui::pos2(rect.center().x, rect.bottom() - inset),
        ],
        Stroke::new(1.0, crate::theme::border_subtle()),
    );
}

/// Segmented, typography-only tab switcher with a gliding accent underline.
///
/// Returns `true` when the operator picked a different tab this frame.
fn tab_switcher(ui: &mut Ui, deck: &Deck, active: &mut usize) -> bool {
    let font = FontId::proportional(deck.rail_label_px());
    let pad_x = (deck.base_px * 0.9).max(9.0);
    let widths: Vec<f32> = DECK_TABS
        .iter()
        .map(|(label, _)| {
            let text_w = ui.fonts(|fonts| {
                fonts
                    .layout_no_wrap((*label).to_owned(), font.clone(), Color32::WHITE)
                    .size()
                    .x
            });
            text_w + pad_x * 2.0
        })
        .collect();

    let height = deck.rail_chip_h();
    let total: f32 = widths.iter().sum::<f32>() + 8.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total, height), Sense::hover());
    let radius = deck.radius();
    ui.painter().rect(
        rect,
        Rounding::same(radius),
        crate::theme::bg_base(),
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    let inner = rect.shrink2(Vec2::new(4.0, 3.0));
    let segment_radius = Rounding::same((radius - 1.0).max(2.0));
    let mut segments = Vec::with_capacity(DECK_TABS.len());
    let mut changed = false;
    let mut x = inner.left();

    for (index, (label, hint)) in DECK_TABS.iter().enumerate() {
        let segment = Rect::from_min_size(
            egui::pos2(x, inner.top()),
            Vec2::new(widths[index], inner.height()),
        );
        segments.push(segment);
        let response = ui.interact(segment, ui.id().with(("deck_tab", index)), Sense::click());
        let is_active = *active == index;
        let hovered = response.hovered();

        if is_active {
            ui.painter().rect(
                segment,
                segment_radius,
                crate::theme::bg_surface_raised(),
                Stroke::new(1.0, crate::theme::accent().linear_multiply(0.85)),
            );
        } else if hovered {
            ui.painter()
                .rect_filled(segment, segment_radius, crate::theme::bg_surface());
        }

        let color = if is_active || hovered {
            crate::theme::text_primary()
        } else {
            crate::theme::text_secondary()
        };
        ui.painter().text(
            segment.center(),
            Align2::CENTER_CENTER,
            *label,
            font.clone(),
            color,
        );

        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(*hint)
            .clicked()
        {
            *active = index;
            changed = true;
        }
        x += widths[index];
    }

    // Gliding accent underline — the deck's signature motion cue.
    let glide =
        ui.ctx()
            .animate_value_with_time(egui::Id::new("deck_tab_glide"), *active as f32, 0.16);
    let lower = (glide.floor().max(0.0) as usize).min(segments.len() - 1);
    let upper = (glide.ceil().max(0.0) as usize).min(segments.len() - 1);
    let t = glide - glide.floor();
    let from = segments[lower];
    let to = segments[upper];
    let glide_between = |a: f32, b: f32| a + (b - a) * t;
    let underline = Rect::from_min_size(
        egui::pos2(
            glide_between(from.left(), to.left()) + 7.0,
            rect.bottom() - 3.5,
        ),
        Vec2::new(
            (glide_between(from.width(), to.width()) - 14.0).max(6.0),
            2.0,
        ),
    );
    ui.painter()
        .rect_filled(underline, Rounding::same(1.0), crate::theme::accent());

    ui.add_space(deck.gap);
    changed
}

/// Live cluster pinned to the right of the rail.
///
/// Right-to-left order, so read top-to-bottom here = right-to-left on screen.
fn rail_status_cluster(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck) {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = (deck.base_px * 0.35).max(4.0);

        if icon_button(ui, deck, "Collapse the studio deck", chevron_up).clicked() {
            app.config.layout_state.ribbon_collapsed = true;
        }
        if icon_button(ui, deck, "Settings  ·  Ctrl+,", sliders_glyph).clicked() {
            app.config.layout_state.settings_open = true;
        }

        let zoom_label = format!(
            "{}%",
            (app.current_theme.font_config.ui_scale * 100.0).round() as i32
        );
        const ZOOM_MIN: f32 = 0.5;
        const ZOOM_MAX: f32 = 2.0;
        match micro_stepper(
            ui,
            deck,
            "deck_zoom",
            &zoom_label,
            "Zoom out  ·  Ctrl+-",
            "Zoom in  ·  Ctrl+=",
        ) {
            -1 => {
                let current = app.current_theme.font_config.ui_scale;
                if current > ZOOM_MIN {
                    app.current_theme.font_config.ui_scale = stepped_scale(current, false);
                    persist_font(app);
                }
            }
            1 => {
                let current = app.current_theme.font_config.ui_scale;
                if current < ZOOM_MAX {
                    app.current_theme.font_config.ui_scale = stepped_scale(current, true);
                    persist_font(app);
                }
            }
            _ => {}
        }

        v_sep(ui, deck, deck.rail_chip_h());

        let latency = app.broadcast_mock.latency_ms;
        let _ = readout_pill(
            ui,
            deck,
            &format!("{latency} ms"),
            latency_color(latency),
            "Program output latency",
        );
        v_sep(ui, deck, deck.rail_chip_h());

        if chip_pill(
            ui,
            deck,
            "PGM OUT",
            app.program_out,
            "Route approved verses to the program feed  ·  F11",
        )
        .clicked()
        {
            app.program_out = !app.program_out;
        }

        let (label, color) = pipeline_pill(app);
        lamp_pill(ui, deck, label, color, pipeline_pulse(ui, app));
    });
}

fn pipeline_pill(app: &HvBibleApp) -> (&'static str, Color32) {
    match app.pipeline_state {
        PipelineState::Stopped => ("STANDBY", STATUS_ERROR),
        PipelineState::Listening => ("ARMED", STATUS_WARNING),
        PipelineState::Silence => ("SILENCE", STATUS_WARNING),
        PipelineState::Speaking => ("SPEAKING", STATUS_SUCCESS),
    }
}

/// Slow breathing pulse for the live lamp while the pipeline is running.
fn pipeline_pulse(ui: &Ui, app: &HvBibleApp) -> f32 {
    if app.pipeline_state == PipelineState::Stopped {
        return 0.45;
    }
    let time = ui.ctx().input(|input| input.time) as f32;
    (0.6 + 0.4 * ((time * 3.2).sin() * 0.5 + 0.5)).clamp(0.0, 1.0)
}

/// Square ghost button with a hand-painted glyph.
fn icon_button(
    ui: &mut Ui,
    deck: &Deck,
    hint: &str,
    draw: fn(&egui::Painter, Rect, Color32),
) -> Response {
    let size = deck.rail_chip_h();
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        if hovered {
            ui.painter().rect_filled(
                rect,
                Rounding::same(deck.radius()),
                crate::theme::bg_surface_raised(),
            );
        }
        let color = if hovered {
            crate::theme::text_primary()
        } else {
            crate::theme::text_tertiary()
        };
        draw(ui.painter(), rect.shrink(size * 0.3), color);
    }
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(hint)
}

/// Compact `−  value  +` capsule. Returns `-1`, `0` or `1`.
fn micro_stepper(
    ui: &mut Ui,
    deck: &Deck,
    id_salt: &str,
    value: &str,
    minus_hint: &str,
    plus_hint: &str,
) -> i8 {
    let height = deck.rail_chip_h();
    let font = FontId::proportional(deck.text(0.74));
    let cell = (deck.base_px * 1.5).max(16.0);
    let value_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(value.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    }) + 12.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(cell * 2.0 + value_w, height), Sense::hover());

    let radius = (deck.radius() - 1.0).max(2.0);
    ui.painter().rect(
        rect,
        Rounding::same(radius),
        crate::theme::bg_base().linear_multiply(1.08),
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    let minus_rect = Rect::from_min_size(rect.min, Vec2::new(cell, height));
    let plus_rect = Rect::from_min_size(
        egui::pos2(rect.right() - cell, rect.top()),
        Vec2::new(cell, height),
    );

    let mut result = 0i8;
    for (button, is_plus) in [(minus_rect, false), (plus_rect, true)] {
        let response = ui.interact(button, ui.id().with((id_salt, is_plus)), Sense::click());
        let hovered = response.hovered();
        if hovered {
            ui.painter().rect_filled(
                button.shrink(1.5),
                Rounding::same((radius - 1.0).max(1.0)),
                crate::theme::bg_surface_raised(),
            );
        }
        let color = if hovered {
            crate::theme::text_primary()
        } else {
            crate::theme::text_secondary()
        };
        let stroke = Stroke::new(1.4, color);
        let center = button.center();
        let half = (deck.base_px * 0.3).max(3.5);
        ui.painter().line_segment(
            [
                egui::pos2(center.x - half, center.y),
                egui::pos2(center.x + half, center.y),
            ],
            stroke,
        );
        if is_plus {
            ui.painter().line_segment(
                [
                    egui::pos2(center.x, center.y - half),
                    egui::pos2(center.x, center.y + half),
                ],
                stroke,
            );
        }
        let hint = if is_plus { plus_hint } else { minus_hint };
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(hint)
            .clicked()
        {
            result = if is_plus { 1 } else { -1 };
        }
    }

    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        value,
        font,
        crate::theme::text_primary(),
    );
    result
}

/// Small pill toggle (dot + label) used for rail-level switches.
fn chip_pill(ui: &mut Ui, deck: &Deck, label: &str, on: bool, hint: &str) -> Response {
    let height = deck.rail_chip_h();
    let font = FontId::proportional(deck.text(0.72));
    let text_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    });
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(text_w + height * 0.95, height), Sense::click());
    let hovered = response.hovered();
    let radius = Rounding::same((deck.radius() - 1.0).max(2.0));
    let (fill, stroke) = if on {
        (
            crate::theme::accent_muted(),
            Stroke::new(1.0, crate::theme::accent().linear_multiply(0.85)),
        )
    } else if hovered {
        (
            crate::theme::bg_surface_raised(),
            Stroke::new(1.0, crate::theme::border_subtle()),
        )
    } else {
        (Color32::TRANSPARENT, Stroke::NONE)
    };
    ui.painter().rect(rect, radius, fill, stroke);

    let dot = egui::pos2(rect.left() + height * 0.42, rect.center().y);
    if on {
        ui.painter().circle_filled(dot, 3.0, crate::theme::accent());
    } else {
        ui.painter()
            .circle_stroke(dot, 3.0, Stroke::new(1.0, crate::theme::text_tertiary()));
    }
    let color = if on || hovered {
        crate::theme::text_primary()
    } else {
        crate::theme::text_secondary()
    };
    ui.painter().text(
        egui::pos2(dot.x + 6.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        color,
    );

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(hint)
}

/// Read-only pill showing a colour-coded value.
fn readout_pill(ui: &mut Ui, deck: &Deck, value: &str, color: Color32, hint: &str) -> Response {
    let height = deck.rail_chip_h();
    let font = FontId::proportional(deck.text(0.72));
    let text_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(value.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    });
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(text_w + height * 0.7, height), Sense::hover());
    ui.painter().rect(
        rect,
        Rounding::same((deck.radius() - 1.0).max(2.0)),
        crate::theme::bg_surface_raised(),
        Stroke::new(1.0, crate::theme::border_subtle()),
    );
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, value, font, color);
    response.on_hover_text(hint)
}

/// Live pipeline lamp: glowing dot + state label.
fn lamp_pill(ui: &mut Ui, deck: &Deck, label: &str, color: Color32, pulse: f32) {
    let height = deck.rail_chip_h();
    let font = FontId::proportional(deck.text(0.74));
    let text_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    });
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(text_w + height * 1.15, height), Sense::hover());
    ui.painter().rect(
        rect,
        Rounding::same((deck.radius() - 1.0).max(2.0)),
        crate::theme::bg_surface_raised(),
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    let dot = egui::pos2(rect.left() + height * 0.55, rect.center().y);
    let halo = (86.0 * pulse.clamp(0.0, 1.0)) as u8;
    ui.painter().circle_filled(
        dot,
        height * 0.26,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), halo),
    );
    ui.painter().circle_filled(dot, height * 0.14, color);
    ui.painter().text(
        egui::pos2(dot.x + height * 0.32, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        crate::theme::text_primary(),
    );

    let _ = response.on_hover_text(format!("Pipeline: {label}"));
}

// ─── Deck ───────────────────────────────────────────────────────────────────

fn command_deck(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck, active_tab: usize) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), deck.deck_h), Sense::hover());

    vertical_gradient(
        ui.painter(),
        rect,
        crate::theme::bg_surface(),
        mix(crate::theme::bg_surface(), crate::theme::bg_base(), 0.55),
    );

    // Hairline that separates the deck from the workspace below.
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    // On-air tally: a red hairline burning along the bottom of the deck.
    if app.program_out && app.is_listening {
        ui.painter().rect_filled(
            Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - 2.0),
                rect.right_bottom(),
            ),
            0.0,
            STATUS_ERROR.linear_multiply(0.9),
        );
    }

    let mut deck_ui = ui.new_child(
        UiBuilder::new()
            .id_salt("deck_body")
            .max_rect(rect.shrink2(Vec2::new(deck.pad, 0.0)))
            .layout(Layout::left_to_right(Align::Min)),
    );
    deck_ui.spacing_mut().item_spacing = Vec2::ZERO;

    match active_tab {
        0 => tab_console(&mut deck_ui, app, deck),
        1 => tab_broadcast(&mut deck_ui, app, deck),
        2 => tab_bible(&mut deck_ui, app, deck),
        3 => tab_audio(&mut deck_ui, app, deck),
        _ => tab_tools(&mut deck_ui, app, deck),
    }
}

/// One command group: a row of tiles with its caption underneath.
fn cluster(ui: &mut Ui, deck: &Deck, caption: &str, content: impl FnOnce(&mut Ui)) {
    let caption_h = deck.caption_block_h() - 4.0;
    let caption_font = FontId::proportional(deck.caption_px());
    let caption_text = caption.to_uppercase();

    ui.add_space(deck.gap);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.add_space(deck.top_pad());
        let row = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = (deck.base_px * 0.5).max(6.0);
                content(ui);
            })
            .response
            .rect;
        ui.add_space(4.0);
        let (caption_rect, _) =
            ui.allocate_exact_size(Vec2::new(row.width(), caption_h), Sense::hover());
        ui.painter().text(
            caption_rect.center(),
            Align2::CENTER_CENTER,
            caption_text,
            caption_font,
            crate::theme::text_tertiary(),
        );
    });
    ui.add_space(deck.gap);
    cluster_sep(ui, deck);
}

/// Soft hairline that keeps neighbouring clusters visually separate.
fn cluster_sep(ui: &mut Ui, deck: &Deck) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new((deck.gap * 1.7).max(9.0), deck.deck_h),
        Sense::hover(),
    );
    ui.painter().line_segment(
        [
            egui::pos2(rect.center().x, rect.top() + deck.top_pad() + 2.0),
            egui::pos2(
                rect.center().x,
                rect.top() + deck.top_pad() + deck.tile_h - 2.0,
            ),
        ],
        Stroke::new(1.0, crate::theme::border_subtle().linear_multiply(0.9)),
    );
}

// ─── Tiles ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum TileKind {
    /// Quiet command: transparent until hovered.
    Ghost,
    /// Command carrying a status colour in its label and hover glow.
    Tint(Color32),
    /// Primary call to action: filled, glowing, inverse label.
    Cta(Color32),
    /// Two-state switch with a dot + accent bar while engaged.
    Toggle(bool),
}

/// A deck command tile — hand-painted, so no icon font is required.
struct Tile<'a> {
    label: &'a str,
    kind: TileKind,
    enabled: bool,
    hint: Option<&'a str>,
}

impl<'a> Tile<'a> {
    fn new(label: &'a str) -> Self {
        Self {
            label,
            kind: TileKind::Ghost,
            enabled: true,
            hint: None,
        }
    }

    fn cta(mut self, color: Color32) -> Self {
        self.kind = TileKind::Cta(color);
        self
    }

    fn tint(mut self, color: Color32) -> Self {
        self.kind = TileKind::Tint(color);
        self
    }

    fn toggle(mut self, on: bool) -> Self {
        self.kind = TileKind::Toggle(on);
        self
    }

    fn ghost(mut self) -> Self {
        self.kind = TileKind::Ghost;
        self
    }

    fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    fn show(self, ui: &mut Ui, deck: &Deck) -> Response {
        let height = deck.tile_h;
        let font = FontId::proportional(deck.tile_label_px());
        let pad_x = (deck.base_px * 0.78).max(9.0);
        let has_mark = matches!(self.kind, TileKind::Toggle(_));
        let mark_w = if has_mark {
            (deck.base_px * 0.95).max(10.0)
        } else {
            0.0
        };
        let text_w = ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(self.label.to_owned(), font.clone(), Color32::WHITE)
                .size()
                .x
        });
        let width = (text_w + pad_x * 2.0 + mark_w).round();

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);

        let radius = deck.radius();
        let hovered = self.enabled && response.hovered();
        let (fill, stroke, text_color) = match self.kind {
            TileKind::Cta(color) => (
                if hovered {
                    color.linear_multiply(1.14)
                } else {
                    color
                },
                Stroke::new(1.0, color.linear_multiply(1.3)),
                crate::theme::text_inverse(),
            ),
            TileKind::Tint(color) => (
                if hovered {
                    translucent(color, 26)
                } else {
                    Color32::TRANSPARENT
                },
                if hovered {
                    Stroke::new(1.0, translucent(color, 120))
                } else {
                    Stroke::NONE
                },
                if self.enabled {
                    color
                } else {
                    color.linear_multiply(0.5)
                },
            ),
            TileKind::Toggle(true) => (
                crate::theme::accent_muted(),
                Stroke::new(1.0, crate::theme::accent().linear_multiply(0.85)),
                crate::theme::text_primary(),
            ),
            TileKind::Toggle(false) | TileKind::Ghost => (
                if hovered {
                    crate::theme::bg_surface_raised()
                } else {
                    Color32::TRANSPARENT
                },
                if hovered {
                    Stroke::new(1.0, crate::theme::border_subtle())
                } else {
                    Stroke::NONE
                },
                if self.enabled {
                    crate::theme::text_secondary()
                } else {
                    crate::theme::text_tertiary().linear_multiply(0.75)
                },
            ),
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            if let TileKind::Cta(color) = self.kind {
                let glow = if hovered { 46 } else { 26 };
                painter.rect_filled(
                    rect.expand(2.0),
                    Rounding::same(radius + 2.0),
                    translucent(color, glow),
                );
            }
            painter.rect(rect, Rounding::same(radius), fill, stroke);

            if let TileKind::Toggle(true) = self.kind {
                let bar = Rect::from_min_size(
                    egui::pos2(rect.left() + 1.5, rect.top() + 6.0),
                    Vec2::new(2.5, (rect.height() - 12.0).max(4.0)),
                );
                painter.rect_filled(bar, Rounding::same(1.5), crate::theme::accent());
            }

            let mut text_left = rect.left() + pad_x;
            if has_mark {
                let mark = egui::pos2(text_left + mark_w * 0.3, rect.center().y);
                let dot = (deck.base_px * 0.2).max(2.4);
                match self.kind {
                    TileKind::Toggle(true) => {
                        painter.circle_filled(mark, dot, crate::theme::accent());
                    }
                    _ => {
                        painter.circle_stroke(
                            mark,
                            dot,
                            Stroke::new(1.0, crate::theme::text_tertiary()),
                        );
                    }
                }
                text_left += mark_w;
            }
            painter.text(
                egui::pos2(text_left, rect.center().y),
                Align2::LEFT_CENTER,
                self.label,
                font,
                text_color,
            );
        }

        let response = if self.enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        };
        match self.hint {
            Some(hint) => response.on_hover_text(hint),
            None => response,
        }
    }
}

// ─── Capsules ───────────────────────────────────────────────────────────────

/// Sunken capsule holding a real widget, with a micro-caption at its left edge.
fn control_capsule(
    ui: &mut Ui,
    deck: &Deck,
    caption: &str,
    width: f32,
    content: impl FnOnce(&mut Ui),
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, deck.tile_h), Sense::hover());
    ui.painter().rect(
        rect,
        Rounding::same(deck.radius()),
        crate::theme::bg_surface_sunken(),
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    let mut capsule = ui.new_child(
        UiBuilder::new()
            .id_salt(("deck_capsule", caption))
            .max_rect(rect.shrink2(Vec2::new(deck.capsule_pad(), 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    capsule.spacing_mut().item_spacing.x = (deck.base_px * 0.42).max(5.0);
    capsule.label(
        RichText::new(caption.to_uppercase())
            .size(deck.caption_px())
            .color(crate::theme::text_tertiary()),
    );
    content(&mut capsule);
}

/// Live RMS / peak readout with mini meters and a clip flag.
fn meter_capsule(ui: &mut Ui, deck: &Deck, app: &HvBibleApp, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, deck.tile_h), Sense::hover());
    ui.painter().rect(
        rect,
        Rounding::same(deck.radius()),
        crate::theme::bg_surface_sunken(),
        Stroke::new(1.0, crate::theme::border_subtle()),
    );

    let inner = rect.shrink2(Vec2::new(deck.capsule_pad(), deck.base_px * 0.3));
    let bar_h = (deck.base_px * 0.32).max(4.0);
    ui.painter().text(
        egui::pos2(inner.left(), inner.center().y),
        Align2::LEFT_CENTER,
        "LEVEL",
        FontId::proportional(deck.caption_px()),
        crate::theme::text_tertiary(),
    );

    let label_x = inner.left() + (deck.base_px * 2.6).max(34.0);
    let meter_left = label_x + (deck.base_px * 1.6).max(20.0);
    let track_w = (inner.right() - meter_left - (deck.base_px * 2.6).max(34.0)).max(18.0);
    let row_h = inner.height() / 2.0;
    let rows = [
        (
            "RMS",
            app.meter_rms,
            meter_color(app.meter_rms),
            crate::theme::text_secondary(),
        ),
        (
            "PK",
            app.meter_peak,
            if app.is_clipping {
                STATUS_ERROR
            } else {
                crate::theme::text_secondary()
            },
            if app.is_clipping {
                STATUS_ERROR
            } else {
                crate::theme::text_secondary()
            },
        ),
    ];

    for (index, (label, value, bar_color, readout_color)) in rows.iter().enumerate() {
        let y = inner.top() + row_h * (index as f32 + 0.5);
        ui.painter().text(
            egui::pos2(label_x, y),
            Align2::LEFT_CENTER,
            *label,
            FontId::proportional(deck.caption_px()),
            crate::theme::text_tertiary(),
        );
        let track = Rect::from_min_size(
            egui::pos2(meter_left, y - bar_h / 2.0),
            Vec2::new(track_w, bar_h),
        );
        ui.painter()
            .rect_filled(track, Rounding::same(bar_h / 2.0), crate::theme::bg_base());
        let level = ((value + 60.0) / 60.0).clamp(0.0, 1.0);
        ui.painter().rect_filled(
            Rect::from_min_size(track.min, Vec2::new(track.width() * level, track.height())),
            Rounding::same(bar_h / 2.0),
            *bar_color,
        );
        ui.painter().text(
            egui::pos2(inner.right(), y),
            Align2::RIGHT_CENTER,
            format!("{value:.1}"),
            FontId::proportional(deck.caption_px()),
            *readout_color,
        );
    }

    if app.is_clipping {
        ui.painter().circle_filled(
            egui::pos2(
                rect.right() - deck.capsule_pad() * 0.7,
                rect.top() + deck.capsule_pad() * 0.7,
            ),
            2.5,
            STATUS_ERROR,
        );
    }
}

// ─── Telemetry ──────────────────────────────────────────────────────────────

/// Right-pinned metric strip: caption over value, hairline separated.
fn deck_telemetry(ui: &mut Ui, deck: &Deck, items: &[(&str, String, Color32)]) {
    if items.is_empty() || ui.available_width() < deck.base_px * 17.0 {
        return;
    }
    let top_pad = deck.top_pad();
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.add_space(top_pad);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (caption, value, color) in items.iter() {
                    telemetry_chip(ui, deck, caption, value, *color);
                }
            });
        });
    });
}

fn telemetry_chip(ui: &mut Ui, deck: &Deck, caption: &str, value: &str, color: Color32) {
    let caption_font = FontId::proportional(deck.caption_px());
    let value_font = FontId::proportional(deck.text(0.82));
    let caption_text = caption.to_uppercase();
    let caption_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(caption_text.clone(), caption_font.clone(), Color32::WHITE)
            .size()
            .x
    });
    let value_w = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(value.to_owned(), value_font.clone(), Color32::WHITE)
            .size()
            .x
    });

    let width = caption_w.max(value_w) + deck.base_px * 1.6;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, deck.tile_h), Sense::hover());
    let center_x = rect.center().x + deck.base_px * 0.3;

    let painter = ui.painter();
    painter.line_segment(
        [
            egui::pos2(rect.left(), rect.center().y - deck.base_px * 0.6),
            egui::pos2(rect.left(), rect.center().y + deck.base_px * 0.6),
        ],
        Stroke::new(1.0, crate::theme::border_subtle()),
    );
    painter.text(
        egui::pos2(center_x, rect.center().y - deck.text(0.44)),
        Align2::CENTER_CENTER,
        caption_text,
        caption_font,
        crate::theme::text_tertiary(),
    );
    painter.text(
        egui::pos2(center_x, rect.center().y + deck.text(0.5)),
        Align2::CENTER_CENTER,
        value,
        value_font,
        color,
    );
}

// ─── Tab content ────────────────────────────────────────────────────────────

fn tab_console(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck) {
    cluster(ui, deck, "transport", |ui| {
        let (label, tone, hint) = if app.is_listening {
            (
                "Stop Listening",
                STATUS_ERROR,
                "Disarm the ASR pipeline  ·  Ctrl+S",
            )
        } else {
            (
                "Start Listening",
                STATUS_SUCCESS,
                "Arm the ASR pipeline  ·  Ctrl+S",
            )
        };
        if Tile::new(label)
            .cta(tone)
            .hint(hint)
            .show(ui, deck)
            .clicked()
        {
            app.toggle_listening();
        }

        let has_verse = app.current_verse.is_some();
        if Tile::new("Approve")
            .tint(STATUS_SUCCESS)
            .enabled(has_verse)
            .hint("Send the preview to program  ·  Space")
            .show(ui, deck)
            .clicked()
        {
            app.approve_current();
        }
        if Tile::new("Clear")
            .ghost()
            .enabled(has_verse)
            .hint("Drop the preview  ·  Esc")
            .show(ui, deck)
            .clicked()
        {
            app.reject_current();
        }
        if Tile::new("Undo")
            .ghost()
            .hint("Step back through the verse history  ·  Ctrl+Z")
            .show(ui, deck)
            .clicked()
        {
            app.undo_verse();
        }
    });

    cluster(ui, deck, "program out", |ui| {
        if Tile::new("Program Out")
            .toggle(app.program_out)
            .hint("Route approved verses to the program display  ·  F11")
            .show(ui, deck)
            .clicked()
        {
            app.program_out = !app.program_out;
        }
        if Tile::new("Clean Feed")
            .toggle(app.broadcast_mock.clean_feed)
            .hint("Strip operator overlays from the program feed")
            .show(ui, deck)
            .clicked()
        {
            app.broadcast_mock.clean_feed = !app.broadcast_mock.clean_feed;
        }
        if Tile::new("Lower Third")
            .toggle(app.broadcast_mock.lower_third)
            .hint("Render the verse as a lower third")
            .show(ui, deck)
            .clicked()
        {
            app.broadcast_mock.lower_third = !app.broadcast_mock.lower_third;
        }
        if Tile::new("NDI")
            .toggle(app.broadcast_mock.ndi_enabled)
            .hint("Publish the program feed over NDI")
            .show(ui, deck)
            .clicked()
        {
            app.broadcast_mock.ndi_enabled = !app.broadcast_mock.ndi_enabled;
        }
    });

    cluster(ui, deck, "captioning", |ui| {
        control_capsule(ui, deck, "bible", 208.0, |ui| {
            translation_combo(ui, app, "deck_console_translation", 70.0);
        });
        control_capsule(ui, deck, "gate", 214.0, |ui| {
            ui.spacing_mut().slider_width = 104.0;
            let mut gate = app.broadcast_mock.confidence_gate as i32;
            if ui
                .add(
                    egui::Slider::new(&mut gate, 0..=100)
                        .show_value(false)
                        .trailing_fill(true),
                )
                .changed()
            {
                app.broadcast_mock.confidence_gate = gate.clamp(0, 100) as u8;
            }
            ui.label(
                RichText::new(format!("{}%", app.broadcast_mock.confidence_gate))
                    .size(deck.text(0.76))
                    .color(crate::theme::text_secondary()),
            );
        });
    });

    deck_telemetry(
        ui,
        deck,
        &[
            (
                "conf",
                format!("{:.0}%", app.asr_confidence.max(0.0) * 100.0),
                confidence_color(app.asr_confidence),
            ),
            (
                "snr",
                format!("{} dB", app.snr_db),
                crate::theme::text_secondary(),
            ),
            (
                "latency",
                format!("{} ms", app.broadcast_mock.latency_ms),
                latency_color(app.broadcast_mock.latency_ms),
            ),
            ("engine", asr_engine_label(app).to_owned(), STATUS_INFO),
        ],
    );
}

fn tab_broadcast(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck) {
    cluster(ui, deck, "tally", |ui| {
        let live = app.program_out && app.is_listening;
        let (label, tone, hint) = if live {
            (
                "End Broadcast",
                STATUS_ERROR,
                "Drop the program feed and disarm the pipeline",
            )
        } else {
            (
                "Go Live",
                STATUS_SUCCESS,
                "Arm the pipeline and open the program feed",
            )
        };
        if Tile::new(label)
            .cta(tone)
            .hint(hint)
            .show(ui, deck)
            .clicked()
        {
            if live {
                app.program_out = false;
            } else {
                app.program_out = true;
                if !app.is_listening {
                    app.toggle_listening();
                }
            }
        }
        if Tile::new("Emergency Clear")
            .tint(STATUS_ERROR)
            .hint("Drop program out and clear the preview")
            .show(ui, deck)
            .clicked()
        {
            app.program_out = false;
            app.reject_current();
        }
    });

    cluster(ui, deck, "overlays", |ui| {
        if Tile::new("Lower Third")
            .toggle(app.broadcast_mock.lower_third)
            .hint("Render the verse as a lower third")
            .show(ui, deck)
            .clicked()
        {
            app.broadcast_mock.lower_third = !app.broadcast_mock.lower_third;
        }
        if Tile::new("Clean Feed")
            .toggle(app.broadcast_mock.clean_feed)
            .hint("Strip operator overlays from the program feed")
            .show(ui, deck)
            .clicked()
        {
            app.broadcast_mock.clean_feed = !app.broadcast_mock.clean_feed;
        }
        if Tile::new("NDI")
            .toggle(app.broadcast_mock.ndi_enabled)
            .hint("Publish the program feed over NDI")
            .show(ui, deck)
            .clicked()
        {
            app.broadcast_mock.ndi_enabled = !app.broadcast_mock.ndi_enabled;
        }
    });

    cluster(ui, deck, "timing", |ui| {
        control_capsule(ui, deck, "gate", 214.0, |ui| {
            ui.spacing_mut().slider_width = 104.0;
            let mut gate = app.broadcast_mock.confidence_gate as i32;
            if ui
                .add(
                    egui::Slider::new(&mut gate, 0..=100)
                        .show_value(false)
                        .trailing_fill(true),
                )
                .changed()
            {
                app.broadcast_mock.confidence_gate = gate.clamp(0, 100) as u8;
            }
            ui.label(
                RichText::new(format!("{}%", app.broadcast_mock.confidence_gate))
                    .size(deck.text(0.76))
                    .color(crate::theme::text_secondary()),
            );
        });
        Tile::new(&format!("{} ms", app.broadcast_mock.latency_ms))
            .tint(latency_color(app.broadcast_mock.latency_ms))
            .hint("Measured program output latency")
            .show(ui, deck);
    });

    let (state_label, state_color) = pipeline_pill(app);
    deck_telemetry(
        ui,
        deck,
        &[
            ("state", state_label.to_owned(), state_color),
            (
                "target",
                app.broadcast_mock
                    .program_output
                    .chars()
                    .take(22)
                    .collect::<String>(),
                crate::theme::text_secondary(),
            ),
            (
                "uptime",
                app.audio_mock.diagnostics.uptime.clone(),
                crate::theme::text_secondary(),
            ),
        ],
    );
}

fn tab_bible(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck) {
    cluster(ui, deck, "lookup", |ui| {
        if Tile::new("Reference Search")
            .tint(STATUS_INFO)
            .hint("Focus the manual reference field  ·  Ctrl+F")
            .show(ui, deck)
            .clicked()
        {
            app.focus_manual = true;
        }
        if Tile::new("Recall Last")
            .ghost()
            .enabled(!app.sermon_log.is_empty())
            .hint("Restore the most recent approved verse  ·  Ctrl+1")
            .show(ui, deck)
            .clicked()
        {
            app.select_log_hotkey(1);
        }
        if Tile::new("Undo")
            .ghost()
            .hint("Step back through the verse history  ·  Ctrl+Z")
            .show(ui, deck)
            .clicked()
        {
            app.undo_verse();
        }
    });

    cluster(ui, deck, "translation", |ui| {
        control_capsule(ui, deck, "bible", 208.0, |ui| {
            translation_combo(ui, app, "deck_bible_translation", 70.0);
        });
        control_capsule(ui, deck, "text size", 236.0, |ui| {
            let label = format!("{:.0} px", app.current_theme.font_config.size.to_pixels());
            match micro_stepper(
                ui,
                deck,
                "deck_text_size",
                &label,
                "Smaller base text",
                "Larger base text",
            ) {
                -1 => {
                    let px = stepped_font_px(app.current_theme.font_config.size.to_pixels(), false);
                    app.current_theme.font_config.size = crate::theme::FontSize::from_pixels(px);
                    persist_font(app);
                }
                1 => {
                    let px = stepped_font_px(app.current_theme.font_config.size.to_pixels(), true);
                    app.current_theme.font_config.size = crate::theme::FontSize::from_pixels(px);
                    persist_font(app);
                }
                _ => {}
            }
        });
    });

    cluster(ui, deck, "preview", |ui| {
        let has_verse = app.current_verse.is_some();
        if Tile::new("Approve")
            .tint(STATUS_SUCCESS)
            .enabled(has_verse)
            .hint("Send the preview to program  ·  Space")
            .show(ui, deck)
            .clicked()
        {
            app.approve_current();
        }
        if Tile::new("Clear")
            .ghost()
            .enabled(has_verse)
            .hint("Drop the preview  ·  Esc")
            .show(ui, deck)
            .clicked()
        {
            app.reject_current();
        }
    });

    let (reference, reference_color) = match &app.current_verse {
        Some((reference, _)) => (reference.clone(), crate::theme::text_primary()),
        None => ("No verse staged".to_owned(), crate::theme::text_tertiary()),
    };
    deck_telemetry(
        ui,
        deck,
        &[
            ("reference", reference, reference_color),
            (
                "conf",
                format!("{:.0}%", app.asr_confidence.max(0.0) * 100.0),
                confidence_color(app.asr_confidence),
            ),
            (
                "bible",
                app.translation.clone(),
                crate::theme::text_secondary(),
            ),
        ],
    );
}

/// AUDIO tab — transport, metering and telemetry only.
///
/// Device selection, gain staging, the signal chain (HPF / noise gate /
/// compressor) and monitoring mute-solo are **owned by the left audio console**
/// (`panels::audio`). Those controls used to be duplicated here, which let the
/// two surfaces drift: the deck and the console could each hold a different
/// value for the same setting. This tab is deliberately read-only for
/// everything the console owns, so there is exactly one editor per setting.
fn tab_audio(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck) {
    cluster(ui, deck, "capture", |ui| {
        let (label, tone, hint) = if app.is_listening {
            (
                "Stop ASR",
                STATUS_ERROR,
                "Disarm the ASR pipeline  ·  Ctrl+S",
            )
        } else {
            (
                "Start ASR",
                STATUS_SUCCESS,
                "Arm the ASR pipeline  ·  Ctrl+S",
            )
        };
        if Tile::new(label)
            .cta(tone)
            .hint(hint)
            .show(ui, deck)
            .clicked()
        {
            app.toggle_listening();
        }
    });

    cluster(ui, deck, "levels", |ui| {
        meter_capsule(ui, deck, app, 210.0);
    });

    // Point operators at the single owner of the audio controls so the ribbon
    // reads as intentional rather than stripped.
    cluster(ui, deck, "audio console", |ui| {
        ui.label(
            RichText::new("Device · gain · chain · monitoring")
                .size(deck.text(0.72))
                .color(crate::theme::text_tertiary()),
        );
    });

    deck_telemetry(
        ui,
        deck,
        &[
            (
                "rms",
                format!("{:.1} dB", app.meter_rms),
                meter_color(app.meter_rms),
            ),
            (
                "peak",
                format!("{:.1} dB", app.meter_peak),
                if app.is_clipping {
                    STATUS_ERROR
                } else {
                    crate::theme::text_secondary()
                },
            ),
            (
                "snr",
                format!("{} dB", app.snr_db),
                crate::theme::text_secondary(),
            ),
            (
                "rate",
                format!(
                    "{} kHz · {} bit",
                    app.audio_mock.device.sample_rate / 1000,
                    app.audio_mock.device.bit_depth
                ),
                crate::theme::text_secondary(),
            ),
        ],
    );
}

fn tab_tools(ui: &mut Ui, app: &mut HvBibleApp, deck: &Deck) {
    let right_tabs = app.workspace_preset.right_tabs;
    cluster(ui, deck, "right dock", |ui| {
        for (index, tab) in right_tabs.iter().enumerate() {
            let engaged = app.active_right_tab == index && !app.config.layout_state.right_collapsed;
            if Tile::new(tab)
                .toggle(engaged)
                .hint("Show this panel on the right dock")
                .show(ui, deck)
                .clicked()
            {
                app.active_right_tab = index;
                app.config.layout_state.right_collapsed = false;
            }
        }
    });

    let bottom_tabs = app.workspace_preset.bottom_tabs;
    cluster(ui, deck, "bottom dock", |ui| {
        for (index, tab) in bottom_tabs.iter().enumerate() {
            let engaged =
                app.active_bottom_tab == index && !app.config.layout_state.middle_bottom_collapsed;
            if Tile::new(tab)
                .toggle(engaged)
                .hint("Show this panel on the bottom dock")
                .show(ui, deck)
                .clicked()
            {
                app.active_bottom_tab = index;
                app.config.layout_state.middle_bottom_collapsed = false;
            }
        }
    });

    cluster(ui, deck, "session", |ui| {
        if Tile::new("Settings")
            .tint(STATUS_INFO)
            .hint("Open the settings window  ·  Ctrl+,")
            .show(ui, deck)
            .clicked()
        {
            app.config.layout_state.settings_open = true;
        }
        if Tile::new("Save Config")
            .ghost()
            .hint("Write the current session settings to disk")
            .show(ui, deck)
            .clicked()
        {
            let _ = app.config.save();
        }
        if Tile::new("Reset Zoom")
            .ghost()
            .hint("Return interface zoom and base text to 100%")
            .show(ui, deck)
            .clicked()
        {
            app.current_theme.font_config.ui_scale = 1.0;
            app.current_theme.font_config.size = crate::theme::FontSize::Medium;
            persist_font(app);
        }
        if Tile::new("Reset Layout")
            .ghost()
            .hint("Restore default panel widths and expand every dock")
            .show(ui, deck)
            .clicked()
        {
            app.config.layout_state = crate::layout::layout_state::LayoutState::default();
        }
    });

    deck_telemetry(
        ui,
        deck,
        &[
            (
                "model",
                app.selected_model_id.clone(),
                crate::theme::text_secondary(),
            ),
            ("engine", asr_engine_label(app).to_owned(), STATUS_INFO),
            (
                "uptime",
                app.audio_mock.diagnostics.uptime.clone(),
                crate::theme::text_secondary(),
            ),
            (
                "build",
                format!("v{}", env!("CARGO_PKG_VERSION")),
                crate::theme::text_tertiary(),
            ),
        ],
    );
}

// ─── Shared widgets ─────────────────────────────────────────────────────────

/// Translation picker shared by the Console and Bible tabs.
fn translation_combo(ui: &mut Ui, app: &mut HvBibleApp, id_salt: &str, width: f32) {
    ui.spacing_mut().combo_width = width;
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(app.translation.clone())
        .width(width)
        .show_ui(ui, |ui| {
            for code in TRANSLATIONS {
                if ui
                    .selectable_value(&mut app.translation, code.to_string(), code)
                    .changed()
                {
                    app.pipeline
                        .send_command(hv_pipeline::PipelineCommand::SetTranslation(
                            code.to_string(),
                        ));
                }
            }
        });
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Linear blend between two theme colours.
fn mix(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color32::from_rgb(
        channel(from.r(), to.r()),
        channel(from.g(), to.g()),
        channel(from.b(), to.b()),
    )
}

/// Same colour with a different alpha.
fn translucent(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Premium touch: a soft top-to-bottom wash across the deck.
fn vertical_gradient(painter: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    let base = mesh.vertices.len() as u32;
    for (pos, color) in [
        (rect.left_top(), top),
        (rect.right_top(), top),
        (rect.right_bottom(), bottom),
        (rect.left_bottom(), bottom),
    ] {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: egui::epaint::WHITE_UV,
            color,
        });
    }
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    painter.add(egui::Shape::mesh(mesh));
}

/// Hand-painted glyph for the collapse button.
fn chevron_up(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let tip = egui::pos2(rect.center().x, rect.top() + rect.height() * 0.2);
    painter.line_segment(
        [
            egui::pos2(rect.left(), rect.center().y + rect.height() * 0.15),
            tip,
        ],
        stroke,
    );
    painter.line_segment(
        [
            tip,
            egui::pos2(rect.right(), rect.center().y + rect.height() * 0.15),
        ],
        stroke,
    );
}

/// Hand-painted glyph for the settings button.
fn sliders_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.3, color);
    for (index, fraction) in [0.16f32, 0.5, 0.84].iter().enumerate() {
        let y = rect.top() + rect.height() * fraction;
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            stroke,
        );
        let knob = if index == 1 { 0.68 } else { 0.34 };
        painter.circle_filled(egui::pos2(rect.left() + rect.width() * knob, y), 1.8, color);
    }
}

fn persist_font(app: &mut HvBibleApp) {
    app.config.font_config = Some(app.current_theme.font_config.clone());
    let _ = app.config.save();
}

fn asr_engine_label(app: &HvBibleApp) -> &'static str {
    match app.asr_info {
        crate::pipeline_integration::AsrEngineInfo::Mock => "Mock ASR",
        crate::pipeline_integration::AsrEngineInfo::SherpaCpu => "Zipformer CPU",
        crate::pipeline_integration::AsrEngineInfo::SherpaGpu => "Zipformer GPU",
    }
}

fn meter_color(db: f32) -> Color32 {
    if db > -6.0 {
        STATUS_ERROR
    } else if db > -18.0 {
        STATUS_SUCCESS
    } else {
        STATUS_WARNING
    }
}

fn latency_color(ms: u16) -> Color32 {
    if ms < 200 {
        STATUS_SUCCESS
    } else {
        STATUS_WARNING
    }
}

fn confidence_color(confidence: f32) -> Color32 {
    if confidence >= 0.9 {
        STATUS_SUCCESS
    } else if confidence >= 0.7 {
        STATUS_WARNING
    } else {
        STATUS_ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::{stepped_font_px, stepped_scale, Deck, DECK_TABS};

    #[test]
    fn deck_geometry_scales_with_the_theme_base_font() {
        let deck = Deck::new(14.0);
        assert_eq!(deck.rail_h, 31.0);
        assert_eq!(deck.deck_h, 55.0);
        assert_eq!(deck.tile_h, 30.0);
        assert_eq!(deck.pad, 12.0);
        assert_eq!(deck.total_h(), deck.rail_h + deck.deck_h);

        let larger = Deck::new(24.0);
        assert!(larger.rail_h > deck.rail_h);
        assert!(larger.deck_h > deck.deck_h);
        assert!(larger.tile_h > deck.tile_h);
        assert!(larger.pad > deck.pad);

        for base in [9.0_f32, 12.0, 14.0, 20.0, 24.0, 34.0] {
            let deck = Deck::new(base);
            assert!(
                deck.tile_h + deck.caption_block_h() <= deck.deck_h,
                "cluster content must fit inside the deck at {base}px"
            );
            assert!(deck.top_pad() >= 0.0);
            assert!(deck.rail_chip_h() < deck.rail_h);
        }
    }

    #[test]
    fn deck_geometry_is_clamped_for_extreme_font_sizes() {
        assert_eq!(Deck::new(4.0), Deck::new(9.0));
        assert_eq!(Deck::new(90.0), Deck::new(34.0));

        let smallest = Deck::new(9.0);
        assert!(smallest.rail_h >= 30.0);
        assert!(smallest.deck_h >= 54.0);
        assert!(smallest.tile_h >= 30.0);
    }

    #[test]
    fn zoom_stepper_walks_in_tenths_and_stops_at_the_limits() {
        assert_eq!(stepped_scale(1.0, true), 1.1);
        assert_eq!(stepped_scale(1.0, false), 0.9);
        assert_eq!(stepped_scale(1.96, true), 2.0);
        assert_eq!(stepped_scale(2.0, true), 2.0);
        assert_eq!(stepped_scale(0.52, false), 0.5);
        assert_eq!(stepped_scale(0.5, false), 0.5);
    }

    #[test]
    fn text_stepper_walks_in_whole_pixels_and_stops_at_the_limits() {
        assert_eq!(stepped_font_px(14.0, true), 15.0);
        assert_eq!(stepped_font_px(14.0, false), 13.0);
        assert_eq!(stepped_font_px(31.6, true), 32.0);
        assert_eq!(stepped_font_px(32.0, true), 32.0);
        assert_eq!(stepped_font_px(10.4, false), 10.0);
        assert_eq!(stepped_font_px(10.0, false), 10.0);
    }

    #[test]
    fn rail_exposes_five_uppercase_tabs_with_tooltips() {
        assert_eq!(DECK_TABS.len(), 5);
        for (label, hint) in DECK_TABS {
            assert_eq!(label, label.to_uppercase());
            assert!(label.is_ascii());
            assert!(!hint.is_empty());
        }
    }

    #[test]
    fn deck_widgets_render_headlessly_without_panicking() {
        use super::{
            cluster, control_capsule, deck_telemetry, micro_stepper, tab_switcher, Tile,
            STATUS_SUCCESS,
        };
        use eframe::egui;

        let ctx = egui::Context::default();
        let deck = Deck::new(14.0);
        let mut active_tab = 2usize;
        let mut stepper = 0i8;

        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1440.0, 900.0),
            )),
            ..Default::default()
        };

        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                assert!(!tab_switcher(ui, &deck, &mut active_tab));
                cluster(ui, &deck, "transport", |ui| {
                    let _ = Tile::new("Start Listening")
                        .cta(STATUS_SUCCESS)
                        .hint("Arm the pipeline")
                        .show(ui, &deck);
                    let _ = Tile::new("Program Out").toggle(true).show(ui, &deck);
                    let _ = Tile::new("Undo").ghost().enabled(false).show(ui, &deck);
                    control_capsule(ui, &deck, "bible", 180.0, |ui| {
                        ui.label("KJV");
                    });
                });
                deck_telemetry(ui, &deck, &[("conf", "94%".to_owned(), STATUS_SUCCESS)]);
                stepper = micro_stepper(ui, &deck, "smoke_zoom", "100%", "less", "more");
            });
        });

        assert_eq!(stepper, 0, "no interaction in a static smoke frame");
    }
}
