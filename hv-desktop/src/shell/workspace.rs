use eframe::egui::{self, Context, CursorIcon, Id, Rect, Sense, Ui};

use crate::app::HvBibleApp;
use crate::layout::layout_state::LayoutState;
use crate::panels;

const SIDE_MIN_WIDTH: f32 = 200.0;
const SIDE_MAX_VIEWPORT_FRACTION: f32 = 0.40;
const BOTTOM_MIN_HEIGHT: f32 = 80.0;
const BOTTOM_MAX_MIDDLE_FRACTION: f32 = 0.50;
const RESIZE_HANDLE_SIZE: f32 = 6.0;
const REOPEN_HANDLE_SIZE: f32 = 4.0;

pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(crate::theme::bg_base())
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            let workspace_rect = ui.max_rect();
            clamp_layout_state(
                &mut app.config.layout_state,
                workspace_rect.width(),
                workspace_rect.height(),
            );

            let layout = app.config.layout_state.clone();
            let slots = WorkspaceSlots::new(workspace_rect, &layout);

            ui.painter()
                .rect_filled(workspace_rect, 0.0, crate::theme::bg_base());

            if let Some(rect) = slots.left_column {
                panel_slot(
                    ui,
                    rect,
                    crate::theme::bg_surface(),
                    egui::vec2(0.0, 0.0),
                    |ui| {
                        panels::audio::show(ui, app);
                    },
                );
            }

            panel_slot(
                ui,
                slots.middle_primary,
                crate::theme::bg_base(),
                egui::vec2(0.0, 0.0),
                |ui| {
                    panels::verse_stage::show(ui, app);
                },
            );

            if let Some(rect) = slots.middle_bottom {
                panel_slot(
                    ui,
                    rect,
                    crate::theme::bg_surface(),
                    egui::vec2(10.0, 8.0),
                    |ui| {
                        bottom_tabs(ui, app);
                    },
                );
            }

            if let Some(rect) = slots.right_column {
                panel_slot(
                    ui,
                    rect,
                    crate::theme::bg_surface(),
                    egui::vec2(10.0, 8.0),
                    |ui| {
                        right_tabs(ui, app);
                    },
                );
            }

            show_column_handles(ui, app, &slots, workspace_rect);
            show_bottom_handle(ui, app, &slots, workspace_rect);
        });
}

pub(crate) fn clamp_layout_state(
    state: &mut LayoutState,
    viewport_width: f32,
    middle_column_height: f32,
) {
    if !state.left_collapsed {
        state.left_width = clamp_side_width(state.left_width, viewport_width);
    }
    if !state.right_collapsed {
        state.right_width = clamp_side_width(state.right_width, viewport_width);
    }
    if !state.middle_bottom_collapsed {
        state.middle_bottom_height =
            clamp_bottom_height(state.middle_bottom_height, middle_column_height);
    }
}

fn clamp_side_width(width: f32, viewport_width: f32) -> f32 {
    let max_width = (viewport_width * SIDE_MAX_VIEWPORT_FRACTION).max(SIDE_MIN_WIDTH);
    width.clamp(SIDE_MIN_WIDTH, max_width)
}

fn clamp_bottom_height(height: f32, middle_column_height: f32) -> f32 {
    let max_height = (middle_column_height * BOTTOM_MAX_MIDDLE_FRACTION).max(BOTTOM_MIN_HEIGHT);
    height.clamp(BOTTOM_MIN_HEIGHT, max_height)
}

#[derive(Debug, Clone)]
struct WorkspaceSlots {
    left_column: Option<Rect>,
    left_resize: Option<Rect>,
    left_restore: Option<Rect>,
    middle_primary: Rect,
    middle_bottom: Option<Rect>,
    middle_bottom_resize: Option<Rect>,
    middle_bottom_restore: Option<Rect>,
    right_column: Option<Rect>,
    right_resize: Option<Rect>,
    right_restore: Option<Rect>,
}

impl WorkspaceSlots {
    fn new(rect: Rect, layout: &LayoutState) -> Self {
        let mut middle_left = rect.left();
        let left_column;
        let left_resize;
        let left_restore;

        if layout.left_collapsed {
            let restore = Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.left() + REOPEN_HANDLE_SIZE, rect.bottom()),
            );
            left_column = None;
            left_resize = None;
            left_restore = Some(restore);
            middle_left += REOPEN_HANDLE_SIZE;
        } else {
            let column = Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.left() + layout.left_width, rect.bottom()),
            );
            let resize = Rect::from_min_max(
                egui::pos2(column.right(), rect.top()),
                egui::pos2(column.right() + RESIZE_HANDLE_SIZE, rect.bottom()),
            );
            left_column = Some(column);
            left_resize = Some(resize);
            left_restore = None;
            middle_left = resize.right();
        }

        let mut middle_right = rect.right();
        let right_column;
        let right_resize;
        let right_restore;

        if layout.right_collapsed {
            let restore = Rect::from_min_max(
                egui::pos2(rect.right() - REOPEN_HANDLE_SIZE, rect.top()),
                rect.right_bottom(),
            );
            right_column = None;
            right_resize = None;
            right_restore = Some(restore);
            middle_right -= REOPEN_HANDLE_SIZE;
        } else {
            let column_left = rect.right() - layout.right_width;
            let column =
                Rect::from_min_max(egui::pos2(column_left, rect.top()), rect.right_bottom());
            let resize = Rect::from_min_max(
                egui::pos2(column.left() - RESIZE_HANDLE_SIZE, rect.top()),
                egui::pos2(column.left(), rect.bottom()),
            );
            right_column = Some(column);
            right_resize = Some(resize);
            right_restore = None;
            middle_right = resize.left();
        }

        let middle_right = middle_right.max(middle_left);
        let middle_rect = Rect::from_min_max(
            egui::pos2(middle_left, rect.top()),
            egui::pos2(middle_right, rect.bottom()),
        );

        let (middle_primary, middle_bottom, middle_bottom_resize, middle_bottom_restore) =
            middle_rows(middle_rect, layout);

        Self {
            left_column,
            left_resize,
            left_restore,
            middle_primary,
            middle_bottom,
            middle_bottom_resize,
            middle_bottom_restore,
            right_column,
            right_resize,
            right_restore,
        }
    }
}

fn middle_rows(
    middle_rect: Rect,
    layout: &LayoutState,
) -> (Rect, Option<Rect>, Option<Rect>, Option<Rect>) {
    if layout.middle_bottom_collapsed {
        let restore_height = REOPEN_HANDLE_SIZE.min(middle_rect.height());
        let restore = Rect::from_min_max(
            egui::pos2(middle_rect.left(), middle_rect.bottom() - restore_height),
            middle_rect.right_bottom(),
        );
        let primary = Rect::from_min_max(
            middle_rect.left_top(),
            egui::pos2(middle_rect.right(), restore.top()),
        );
        return (primary, None, None, Some(restore));
    }

    let bottom_height = layout
        .middle_bottom_height
        .min((middle_rect.height() - RESIZE_HANDLE_SIZE).max(0.0));
    let handle_height = RESIZE_HANDLE_SIZE.min(middle_rect.height());
    let handle_bottom = middle_rect.bottom() - bottom_height;
    let handle_top = (handle_bottom - handle_height).max(middle_rect.top());

    let primary = Rect::from_min_max(
        middle_rect.left_top(),
        egui::pos2(middle_rect.right(), handle_top),
    );
    let resize = Rect::from_min_max(
        egui::pos2(middle_rect.left(), handle_top),
        egui::pos2(middle_rect.right(), handle_bottom),
    );
    let bottom = Rect::from_min_max(
        egui::pos2(middle_rect.left(), handle_bottom),
        middle_rect.right_bottom(),
    );

    (primary, Some(bottom), Some(resize), None)
}

fn panel_slot(
    ui: &mut Ui,
    rect: Rect,
    fill: egui::Color32,
    margin: egui::Vec2,
    contents: impl FnOnce(&mut Ui),
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    ui.painter().rect_filled(rect, 0.0, fill);
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, crate::theme::border_subtle()),
    );

    let inner = rect.shrink2(margin);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.set_min_size(inner.size());
        contents(ui);
    });
}

fn show_column_handles(
    ui: &mut Ui,
    app: &mut HvBibleApp,
    slots: &WorkspaceSlots,
    workspace_rect: Rect,
) {
    if let Some(rect) = slots.left_resize {
        if resize_handle(ui, rect, "left_column_resize", CursorIcon::ResizeHorizontal).dragged() {
            let dx = ui.input(|i| i.pointer.delta().x);
            app.config.layout_state.left_width += dx;
            clamp_layout_state(
                &mut app.config.layout_state,
                workspace_rect.width(),
                workspace_rect.height(),
            );
        }
    }

    if let Some(rect) = slots.right_resize {
        if resize_handle(
            ui,
            rect,
            "right_column_resize",
            CursorIcon::ResizeHorizontal,
        )
        .dragged()
        {
            let dx = ui.input(|i| i.pointer.delta().x);
            app.config.layout_state.right_width -= dx;
            clamp_layout_state(
                &mut app.config.layout_state,
                workspace_rect.width(),
                workspace_rect.height(),
            );
        }
    }

    if let Some(rect) = slots.left_restore {
        if reopen_handle(ui, rect, "left_column_restore", "\u{203A}") {
            app.config.layout_state.left_collapsed = false;
        }
    }

    if let Some(rect) = slots.right_restore {
        if reopen_handle(ui, rect, "right_column_restore", "\u{2039}") {
            app.config.layout_state.right_collapsed = false;
        }
    }
}

fn show_bottom_handle(
    ui: &mut Ui,
    app: &mut HvBibleApp,
    slots: &WorkspaceSlots,
    workspace_rect: Rect,
) {
    if let Some(rect) = slots.middle_bottom_resize {
        if resize_handle(ui, rect, "middle_bottom_resize", CursorIcon::ResizeVertical).dragged() {
            let dy = ui.input(|i| i.pointer.delta().y);
            app.config.layout_state.middle_bottom_height -= dy;
            clamp_layout_state(
                &mut app.config.layout_state,
                workspace_rect.width(),
                workspace_rect.height(),
            );
        }
    }

    if let Some(rect) = slots.middle_bottom_restore {
        if reopen_handle(ui, rect, "middle_bottom_restore", "^") {
            app.config.layout_state.middle_bottom_collapsed = false;
        }
    }
}

fn resize_handle(ui: &mut Ui, rect: Rect, id: &'static str, cursor: CursorIcon) -> egui::Response {
    ui.painter()
        .rect_filled(rect, 0.0, crate::theme::bg_surface_raised());
    let center = rect.center();
    if rect.width() < rect.height() {
        ui.painter().vline(
            center.x,
            rect.top() + 8.0..=rect.bottom() - 8.0,
            egui::Stroke::new(1.0, crate::theme::border_strong()),
        );
    } else {
        ui.painter().hline(
            rect.left() + 8.0..=rect.right() - 8.0,
            center.y,
            egui::Stroke::new(1.0, crate::theme::border_strong()),
        );
    }
    ui.interact(rect, Id::new(id), Sense::click_and_drag())
        .on_hover_cursor(cursor)
}

fn reopen_handle(ui: &mut Ui, rect: Rect, id: &'static str, label: &'static str) -> bool {
    ui.painter()
        .rect_filled(rect, 0.0, crate::theme::accent_muted());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.0),
        crate::theme::text_primary(),
    );
    ui.interact(rect, Id::new(id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
}

fn right_tabs(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    ui.horizontal_wrapped(|ui| {
        for (index, tab) in app.workspace_preset.right_tabs.iter().enumerate() {
            let selected = app.active_right_tab == index;
            if ui.selectable_label(selected, *tab).clicked() {
                app.active_right_tab = index;
            }
        }
        if ui
            .small_button("-")
            .on_hover_text("Collapse right column")
            .clicked()
        {
            app.config.layout_state.right_collapsed = true;
        }
    });
    ui.separator();
    match app.active_right_tab {
        0 => panels::run_sheet_panel::show(ui, app),
        1 => panels::log_panel::show(ui, app),
        2 => panels::queue_panel::show(ui, app),
        _ => panels::run_sheet_panel::show_detected(ui, app),
    }
}

fn bottom_tabs(ui: &mut egui::Ui, app: &mut HvBibleApp) {
    ui.horizontal_wrapped(|ui| {
        for (index, tab) in app.workspace_preset.bottom_tabs.iter().enumerate() {
            let selected = app.active_bottom_tab == index;
            if ui.selectable_label(selected, *tab).clicked() {
                app.active_bottom_tab = index;
            }
        }
        if ui
            .small_button("-")
            .on_hover_text("Collapse bottom row")
            .clicked()
        {
            app.config.layout_state.middle_bottom_collapsed = true;
        }
    });
    ui.separator();
    match app.active_bottom_tab {
        0 => panels::transcript_panel::show(ui, app),
        1 => panels::metrics_panel::show(ui, app),
        _ => panels::metrics_panel::show_events(ui, app),
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::layout_state::LayoutState;

    #[test]
    fn layout_state_clamps_widths_and_bottom_height_to_viewport_rules() {
        let mut state = LayoutState {
            left_collapsed: false,
            left_width: 900.0,
            right_collapsed: false,
            right_width: 900.0,
            middle_bottom_collapsed: false,
            middle_bottom_height: 700.0,
            ribbon_collapsed: false,
        };

        super::clamp_layout_state(&mut state, 1600.0, 900.0);

        assert_eq!(state.left_width, 640.0);
        assert_eq!(state.right_width, 640.0);
        assert_eq!(state.middle_bottom_height, 450.0);
    }

    #[test]
    fn collapsed_layout_keeps_restore_widths_for_reopen_handles() {
        let mut state = LayoutState {
            left_collapsed: true,
            left_width: 340.0,
            right_collapsed: true,
            right_width: 320.0,
            middle_bottom_collapsed: true,
            middle_bottom_height: 172.0,
            ribbon_collapsed: true,
        };

        super::clamp_layout_state(&mut state, 1200.0, 700.0);

        assert_eq!(state.left_width, 340.0);
        assert_eq!(state.right_width, 320.0);
        assert_eq!(state.middle_bottom_height, 172.0);
    }
}
