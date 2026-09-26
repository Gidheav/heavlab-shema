//! Theme management and application logic.

use super::colors::{ColorPalette, ColorPalette32};
use super::fonts::FontConfig;
use super::registry::{ThemeDefinition, ThemeId};
use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
    FollowSystem,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: ThemeId,
    pub mode: ThemeMode,
    pub custom_colors: Option<ColorPalette>,
    pub font_config: FontConfig,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            id: "purple_graphite_dark".to_string(),
            mode: ThemeMode::Dark,
            custom_colors: None,
            font_config: FontConfig::default(),
        }
    }
}

impl Theme {
    pub fn new(id: ThemeId, mode: ThemeMode) -> Self {
        Self {
            id,
            mode,
            custom_colors: None,
            font_config: FontConfig::default(),
        }
    }

    pub fn with_colors(mut self, colors: ColorPalette) -> Self {
        self.custom_colors = Some(colors);
        self
    }

    pub fn with_font(mut self, font_config: FontConfig) -> Self {
        self.font_config = font_config;
        self
    }

    pub fn apply(&self, ctx: &egui::Context, definition: &ThemeDefinition) {
        let colors = if let Some(custom) = &self.custom_colors {
            custom.to_color32()
        } else {
            match self.mode {
                ThemeMode::Dark => definition.dark_colors.to_color32(),
                ThemeMode::Light => definition.light_colors.to_color32(),
                ThemeMode::FollowSystem => {
                    // Default to dark if system detection fails
                    if super::IS_DARK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                        definition.dark_colors.to_color32()
                    } else {
                        definition.light_colors.to_color32()
                    }
                }
            }
        };

        let mut visuals = match self.mode {
            ThemeMode::Dark => egui::Visuals::dark(),
            ThemeMode::Light => egui::Visuals::light(),
            ThemeMode::FollowSystem => {
                if super::IS_DARK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                }
            }
        };

        // Apply theme colors
        visuals.override_text_color = Some(colors.text_primary);
        visuals.panel_fill = colors.bg_base;
        visuals.window_fill = colors.bg_surface;
        visuals.extreme_bg_color = colors.bg_surface_sunken;
        visuals.faint_bg_color = colors.bg_surface;
        visuals.widgets.inactive.bg_fill = colors.bg_surface_raised;
        visuals.widgets.hovered.bg_fill = if matches!(self.mode, ThemeMode::Dark) {
            colors.bg_surface_raised.linear_multiply(1.08)
        } else {
            colors.bg_surface_raised.linear_multiply(0.92)
        };
        visuals.widgets.active.bg_fill = colors.accent_muted;
        visuals.widgets.open.bg_fill = colors.bg_surface_raised;
        visuals.widgets.noninteractive.bg_fill = colors.bg_surface;
        visuals.selection.bg_fill = colors.accent_muted;
        visuals.hyperlink_color = colors.accent;

        // Apply text colors
        let tp = colors.text_primary;
        visuals.widgets.inactive.fg_stroke.color = tp;
        visuals.widgets.hovered.fg_stroke.color = tp;
        visuals.widgets.active.fg_stroke.color = tp;

        // Update dark mode state
        super::IS_DARK_MODE.store(matches!(self.mode, ThemeMode::Dark), std::sync::atomic::Ordering::Relaxed);

        ctx.set_visuals(visuals);
    }

    pub fn get_current_colors(&self, definition: &ThemeDefinition) -> ColorPalette32 {
        if let Some(custom) = &self.custom_colors {
            custom.to_color32()
        } else {
            match self.mode {
                ThemeMode::Dark => definition.dark_colors.to_color32(),
                ThemeMode::Light => definition.light_colors.to_color32(),
                ThemeMode::FollowSystem => {
                    if super::IS_DARK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                        definition.dark_colors.to_color32()
                    } else {
                        definition.light_colors.to_color32()
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePreview {
    pub theme: Theme,
    pub colors: ColorPalette,
}

impl ThemePreview {
    pub fn new(theme: Theme, definition: &ThemeDefinition) -> Self {
        let colors = if let Some(custom) = &theme.custom_colors {
            custom.clone()
        } else {
            let theme_colors = match theme.mode {
                ThemeMode::Dark => &definition.dark_colors,
                ThemeMode::Light => &definition.light_colors,
                ThemeMode::FollowSystem => {
                    if super::IS_DARK_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                        &definition.dark_colors
                    } else {
                        &definition.light_colors
                    }
                }
            };
            ColorPalette {
                bg_base: theme_colors.bg_base,
                bg_surface: theme_colors.bg_surface,
                bg_surface_raised: theme_colors.bg_surface_raised,
                bg_surface_sunken: theme_colors.bg_surface_sunken,
                border_subtle: theme_colors.border_subtle,
                border_strong: theme_colors.border_strong,
                text_primary: theme_colors.text_primary,
                text_secondary: theme_colors.text_secondary,
                text_tertiary: theme_colors.text_tertiary,
                text_inverse: theme_colors.text_inverse,
                accent: theme_colors.accent,
                accent_hover: theme_colors.accent_hover,
                accent_muted: theme_colors.accent_muted,
            }
        };
        Self { theme, colors }
    }

    pub fn apply_preview(&self, ctx: &egui::Context) {
        let colors = self.colors.to_color32();
        let mut visuals = if matches!(self.theme.mode, ThemeMode::Dark) {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.override_text_color = Some(colors.text_primary);
        visuals.panel_fill = colors.bg_base;
        visuals.window_fill = colors.bg_surface;
        visuals.extreme_bg_color = colors.bg_surface_sunken;
        visuals.faint_bg_color = colors.bg_surface;
        visuals.widgets.inactive.bg_fill = colors.bg_surface_raised;
        visuals.widgets.hovered.bg_fill = if matches!(self.theme.mode, ThemeMode::Dark) {
            colors.bg_surface_raised.linear_multiply(1.08)
        } else {
            colors.bg_surface_raised.linear_multiply(0.92)
        };
        visuals.widgets.active.bg_fill = colors.accent_muted;
        visuals.widgets.open.bg_fill = colors.bg_surface_raised;
        visuals.widgets.noninteractive.bg_fill = colors.bg_surface;
        visuals.selection.bg_fill = colors.accent_muted;
        visuals.hyperlink_color = colors.accent;

        let tp = colors.text_primary;
        visuals.widgets.inactive.fg_stroke.color = tp;
        visuals.widgets.hovered.fg_stroke.color = tp;
        visuals.widgets.active.fg_stroke.color = tp;

        super::IS_DARK_MODE.store(matches!(self.theme.mode, ThemeMode::Dark), std::sync::atomic::Ordering::Relaxed);
        ctx.set_visuals(visuals);
    }
}
