//! Theme registry with 10+ premium themes.

use super::colors::ThemeColors;
use super::fonts::FontConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ThemeId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub id: ThemeId,
    pub name: String,
    pub description: String,
    pub category: ThemeCategory,
    pub dark_colors: ThemeColors,
    pub light_colors: ThemeColors,
    pub default_font: FontConfig,
    pub is_premium: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeCategory {
    Professional,
    Elegant,
    Vibrant,
    Minimal,
    HighContrast,
    Broadcast,
    Nature,
    Retro,
    Cyberpunk,
    Custom,
}

impl std::fmt::Display for ThemeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeCategory::Professional => write!(f, "Professional"),
            ThemeCategory::Elegant => write!(f, "Elegant"),
            ThemeCategory::Vibrant => write!(f, "Vibrant"),
            ThemeCategory::Minimal => write!(f, "Minimal"),
            ThemeCategory::HighContrast => write!(f, "High Contrast"),
            ThemeCategory::Broadcast => write!(f, "Broadcast"),
            ThemeCategory::Nature => write!(f, "Nature"),
            ThemeCategory::Retro => write!(f, "Retro"),
            ThemeCategory::Cyberpunk => write!(f, "Cyberpunk"),
            ThemeCategory::Custom => write!(f, "Custom"),
        }
    }
}

pub struct ThemeRegistry {
    themes: HashMap<ThemeId, ThemeDefinition>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
        };
        registry.register_premium_themes();
        registry
    }

    pub fn register(&mut self, theme: ThemeDefinition) {
        self.themes.insert(theme.id.clone(), theme);
    }

    pub fn get(&self, id: &str) -> Option<&ThemeDefinition> {
        self.themes.get(id)
    }

    pub fn list_all(&self) -> Vec<&ThemeDefinition> {
        self.themes.values().collect()
    }

    pub fn list_by_category(&self, category: ThemeCategory) -> Vec<&ThemeDefinition> {
        self.themes
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    pub fn list_premium(&self) -> Vec<&ThemeDefinition> {
        self.themes
            .values()
            .filter(|t| t.is_premium)
            .collect()
    }

    fn register_premium_themes(&mut self) {
        // 1. Purple Graphite (Dark) - Professional
        self.register(ThemeDefinition {
            id: "purple_graphite_dark".to_string(),
            name: "Purple Graphite (Dark)".to_string(),
            description: "Professional broadcast theme with purple accents".to_string(),
            category: ThemeCategory::Professional,
            dark_colors: ThemeColors::from_array([
                [0x15, 0x13, 0x1D], // bg_base
                [0x20, 0x1D, 0x2A], // bg_surface
                [0x2A, 0x25, 0x36], // bg_surface_raised
                [0x11, 0x10, 0x18], // bg_surface_sunken
                [0x36, 0x30, 0x46], // border_subtle
                [0x4B, 0x42, 0x61], // border_strong
                [0xEC, 0xE9, 0xF3], // text_primary
                [0xB4, 0xAD, 0xC4], // text_secondary
                [0x77, 0x6F, 0x88], // text_tertiary
                [0x12, 0x10, 0x18], // text_inverse
                [0xA7, 0x78, 0xF2], // accent
                [0xB8, 0x8F, 0xFF], // accent_hover
                [0x4F, 0x39, 0x73], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xF5, 0xF5, 0xF7], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xEA, 0xEA, 0xED], // bg_surface_raised
                [0xE1, 0xE1, 0xE5], // bg_surface_sunken
                [0xD2, 0xD2, 0xD7], // border_subtle
                [0xAD, 0xAD, 0xB5], // border_strong
                [0x1D, 0x1D, 0x1F], // text_primary
                [0x38, 0x38, 0x3D], // text_secondary
                [0x58, 0x58, 0x60], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x8C, 0x52, 0xFF], // accent
                [0x72, 0x37, 0xE6], // accent_hover
                [0xDF, 0xD1, 0xF7], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 2. Studio Light - Professional
        self.register(ThemeDefinition {
            id: "studio_light".to_string(),
            name: "Studio Light".to_string(),
            description: "Clean, professional light theme for studio environments".to_string(),
            category: ThemeCategory::Professional,
            dark_colors: ThemeColors::from_array([
                [0x1A, 0x1A, 0x1E], // bg_base
                [0x25, 0x25, 0x2B], // bg_surface
                [0x30, 0x30, 0x38], // bg_surface_raised
                [0x15, 0x15, 0x18], // bg_surface_sunken
                [0x3E, 0x3E, 0x46], // border_subtle
                [0x50, 0x50, 0x5A], // border_strong
                [0xF0, 0xF0, 0xF5], // text_primary
                [0xB0, 0xB0, 0xB8], // text_secondary
                [0x70, 0x70, 0x78], // text_tertiary
                [0x10, 0x10, 0x12], // text_inverse
                [0x4D, 0xA6, 0xFF], // accent
                [0x3D, 0x8F, 0xD6], // accent_hover
                [0x2A, 0x4A, 0x6A], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xFA, 0xFA, 0xFC], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xF0, 0xF0, 0xF5], // bg_surface_raised
                [0xE8, 0xE8, 0xEC], // bg_surface_sunken
                [0xE0, 0xE0, 0xE5], // border_subtle
                [0xC0, 0xC0, 0xC8], // border_strong
                [0x1A, 0x1A, 0x1E], // text_primary
                [0x40, 0x40, 0x45], // text_secondary
                [0x60, 0x60, 0x65], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x3B, 0x82, 0xF6], // accent
                [0x2B, 0x6F, 0xCB], // accent_hover
                [0xD0, 0xE0, 0xFF], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 3. Lovely Pink - Elegant
        self.register(ThemeDefinition {
            id: "lovely_pink".to_string(),
            name: "Lovely Pink".to_string(),
            description: "Soft, elegant pink theme with warm accents".to_string(),
            category: ThemeCategory::Elegant,
            dark_colors: ThemeColors::from_array([
                [0x1F, 0x15, 0x20], // bg_base
                [0x2A, 0x1E, 0x2C], // bg_surface
                [0x35, 0x28, 0x38], // bg_surface_raised
                [0x18, 0x12, 0x18], // bg_surface_sunken
                [0x40, 0x32, 0x45], // border_subtle
                [0x55, 0x45, 0x5A], // border_strong
                [0xFF, 0xE6, 0xF2], // text_primary
                [0xE0, 0xB8, 0xD0], // text_secondary
                [0xA0, 0x78, 0x90], // text_tertiary
                [0x15, 0x10, 0x15], // text_inverse
                [0xFF, 0x69, 0xB4], // accent
                [0xFF, 0x85, 0xC8], // accent_hover
                [0x8B, 0x3A, 0x62], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xFF, 0xF0, 0xF8], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xF8, 0xE8, 0xF0], // bg_surface_raised
                [0xF0, 0xE0, 0xE8], // bg_surface_sunken
                [0xE8, 0xD8, 0xE0], // border_subtle
                [0xD0, 0xB8, 0xC8], // border_strong
                [0x33, 0x15, 0x25], // text_primary
                [0x55, 0x35, 0x45], // text_secondary
                [0x75, 0x55, 0x65], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0xE9, 0x1E, 0x63], // accent
                [0xC2, 0x18, 0x5B], // accent_hover
                [0xFF, 0xC1, 0xE3], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 4. Ocean Blue - Elegant
        self.register(ThemeDefinition {
            id: "ocean_blue".to_string(),
            name: "Ocean Blue".to_string(),
            description: "Calming ocean blue theme with depth".to_string(),
            category: ThemeCategory::Elegant,
            dark_colors: ThemeColors::from_array([
                [0x0F, 0x1A, 0x20], // bg_base
                [0x18, 0x25, 0x30], // bg_surface
                [0x22, 0x32, 0x40], // bg_surface_raised
                [0x0A, 0x12, 0x18], // bg_surface_sunken
                [0x2A, 0x3A, 0x48], // border_subtle
                [0x38, 0x4A, 0x5A], // border_strong
                [0xE0, 0xF0, 0xFF], // text_primary
                [0xA0, 0xC0, 0xE0], // text_secondary
                [0x60, 0x80, 0xA0], // text_tertiary
                [0x08, 0x10, 0x15], // text_inverse
                [0x00, 0xA8, 0xE8], // accent
                [0x00, 0xB8, 0xF8], // accent_hover
                [0x1A, 0x4A, 0x6A], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xF0, 0xF8, 0xFF], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xE8, 0xF0, 0xF8], // bg_surface_raised
                [0xE0, 0xE8, 0xF0], // bg_surface_sunken
                [0xD8, 0xE0, 0xE8], // border_subtle
                [0xC0, 0xC8, 0xD0], // border_strong
                [0x10, 0x20, 0x30], // text_primary
                [0x30, 0x40, 0x50], // text_secondary
                [0x50, 0x60, 0x70], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x03, 0x89, 0xCC], // accent
                [0x02, 0x79, 0xBC], // accent_hover
                [0xD0, 0xE8, 0xFF], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 5. Sunset Orange - Vibrant
        self.register(ThemeDefinition {
            id: "sunset_orange".to_string(),
            name: "Sunset Orange".to_string(),
            description: "Warm sunset orange theme with energy".to_string(),
            category: ThemeCategory::Vibrant,
            dark_colors: ThemeColors::from_array([
                [0x1F, 0x15, 0x10], // bg_base
                [0x2A, 0x1E, 0x18], // bg_surface
                [0x35, 0x28, 0x20], // bg_surface_raised
                [0x18, 0x12, 0x0A], // bg_surface_sunken
                [0x40, 0x32, 0x28], // border_subtle
                [0x55, 0x45, 0x38], // border_strong
                [0xFF, 0xF0, 0xE6], // text_primary
                [0xE0, 0xC8, 0xB0], // text_secondary
                [0xA0, 0x80, 0x60], // text_tertiary
                [0x15, 0x10, 0x08], // text_inverse
                [0xFF, 0x70, 0x43], // accent
                [0xFF, 0x85, 0x60], // accent_hover
                [0x8B, 0x45, 0x28], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xFF, 0xF5, 0xF0], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xFF, 0xE8, 0xE0], // bg_surface_raised
                [0xFF, 0xE0, 0xD8], // bg_surface_sunken
                [0xFF, 0xD8, 0xD0], // border_subtle
                [0xE8, 0xC8, 0xB8], // border_strong
                [0x2A, 0x15, 0x08], // text_primary
                [0x4A, 0x30, 0x20], // text_secondary
                [0x6A, 0x50, 0x40], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0xF4, 0x5D, 0x05], // accent
                [0xD4, 0x40, 0x04], // accent_hover
                [0xFF, 0xD1, 0x8D], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 6. AMOLED Black - Minimal
        self.register(ThemeDefinition {
            id: "amoled_black".to_string(),
            name: "AMOLED Black".to_string(),
            description: "True black for OLED displays with pure contrast".to_string(),
            category: ThemeCategory::Minimal,
            dark_colors: ThemeColors::from_array([
                [0x00, 0x00, 0x00], // bg_base
                [0x0A, 0x0A, 0x0A], // bg_surface
                [0x12, 0x12, 0x12], // bg_surface_raised
                [0x00, 0x00, 0x00], // bg_surface_sunken
                [0x1A, 0x1A, 0x1A], // border_subtle
                [0x25, 0x25, 0x25], // border_strong
                [0xE8, 0xE8, 0xE8], // text_primary
                [0xA0, 0xA0, 0xA0], // text_secondary
                [0x60, 0x60, 0x60], // text_tertiary
                [0x00, 0x00, 0x00], // text_inverse
                [0x00, 0xFF, 0x88], // accent
                [0x00, 0xFF, 0x95], // accent_hover
                [0x00, 0x4A, 0x28], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xFF, 0xFF, 0xFF], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xF0, 0xF0, 0xF0], // bg_surface_raised
                [0xE8, 0xE8, 0xE8], // bg_surface_sunken
                [0xE0, 0xE0, 0xE0], // border_subtle
                [0xC0, 0xC0, 0xC0], // border_strong
                [0x00, 0x00, 0x00], // text_primary
                [0x30, 0x30, 0x30], // text_secondary
                [0x50, 0x50, 0x50], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x00, 0x9D, 0x72], // accent
                [0x00, 0x7D, 0x5C], // accent_hover
                [0xD0, 0xF0, 0xE8], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 7. Broadcast Red - Broadcast
        self.register(ThemeDefinition {
            id: "broadcast_red".to_string(),
            name: "Broadcast Red".to_string(),
            description: "Professional broadcast theme with red accents".to_string(),
            category: ThemeCategory::Broadcast,
            dark_colors: ThemeColors::from_array([
                [0x1A, 0x0F, 0x12], // bg_base
                [0x22, 0x18, 0x1C], // bg_surface
                [0x2D, 0x22, 0x28], // bg_surface_raised
                [0x12, 0x0A, 0x0D], // bg_surface_sunken
                [0x38, 0x2A, 0x30], // border_subtle
                [0x48, 0x38, 0x40], // border_strong
                [0xFF, 0xE8, 0xEA], // text_primary
                [0xD0, 0xB0, 0xB8], // text_secondary
                [0x90, 0x70, 0x78], // text_tertiary
                [0x10, 0x08, 0x0A], // text_inverse
                [0xFF, 0x44, 0x44], // accent
                [0xFF, 0x60, 0x60], // accent_hover
                [0x8B, 0x25, 0x25], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xFF, 0xF0, 0xF2], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xF8, 0xE8, 0xEA], // bg_surface_raised
                [0xF0, 0xE0, 0xE8], // bg_surface_sunken
                [0xE8, 0xD8, 0xE0], // border_subtle
                [0xD0, 0xB8, 0xC0], // border_strong
                [0x28, 0x08, 0x10], // text_primary
                [0x48, 0x28, 0x30], // text_secondary
                [0x68, 0x48, 0x50], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0xE5, 0x39, 0x35], // accent
                [0xC6, 0x28, 0x28], // accent_hover
                [0xFF, 0xC1, 0xC1], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 8. Forest Green - Nature
        self.register(ThemeDefinition {
            id: "forest_green".to_string(),
            name: "Forest Green".to_string(),
            description: "Natural forest green theme with organic feel".to_string(),
            category: ThemeCategory::Nature,
            dark_colors: ThemeColors::from_array([
                [0x0F, 0x1A, 0x12], // bg_base
                [0x18, 0x25, 0x1C], // bg_surface
                [0x22, 0x32, 0x28], // bg_surface_raised
                [0x0A, 0x12, 0x0A], // bg_surface_sunken
                [0x2A, 0x3A, 0x30], // border_subtle
                [0x38, 0x4A, 0x40], // border_strong
                [0xE0, 0xF0, 0xE8], // text_primary
                [0xA0, 0xC0, 0xB0], // text_secondary
                [0x60, 0x80, 0x70], // text_tertiary
                [0x08, 0x10, 0x08], // text_inverse
                [0x4C, 0xAF, 0x50], // accent
                [0x5C, 0xBF, 0x60], // accent_hover
                [0x2A, 0x5A, 0x30], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xF0, 0xF8, 0xF4], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xE8, 0xF0, 0xEC], // bg_surface_raised
                [0xE0, 0xE8, 0xE4], // bg_surface_sunken
                [0xD8, 0xE0, 0xDC], // border_subtle
                [0xC0, 0xC8, 0xC4], // border_strong
                [0x10, 0x20, 0x15], // text_primary
                [0x30, 0x40, 0x35], // text_secondary
                [0x50, 0x60, 0x55], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x43, 0xA0, 0x47], // accent
                [0x38, 0x8E, 0x3C], // accent_hover
                [0xC8, 0xE6, 0xC9], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 9. Neon Cyberpunk - Cyberpunk
        self.register(ThemeDefinition {
            id: "neon_cyberpunk".to_string(),
            name: "Neon Cyberpunk".to_string(),
            description: "Futuristic cyberpunk theme with neon accents".to_string(),
            category: ThemeCategory::Cyberpunk,
            dark_colors: ThemeColors::from_array([
                [0x0A, 0x00, 0x14], // bg_base
                [0x12, 0x08, 0x20], // bg_surface
                [0x1A, 0x10, 0x30], // bg_surface_raised
                [0x05, 0x00, 0x0A], // bg_surface_sunken
                [0x22, 0x18, 0x38], // border_subtle
                [0x30, 0x25, 0x48], // border_strong
                [0xFF, 0x00, 0xFF], // text_primary
                [0xD0, 0x00, 0xD0], // text_secondary
                [0x80, 0x00, 0x80], // text_tertiary
                [0x00, 0x00, 0x00], // text_inverse
                [0x00, 0xFF, 0xFF], // accent
                [0x00, 0xFF, 0xFF], // accent_hover
                [0x00, 0x6A, 0x6A], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xF0, 0xF8, 0xFF], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xE8, 0xF0, 0xF8], // bg_surface_raised
                [0xE0, 0xE8, 0xF0], // bg_surface_sunken
                [0xD8, 0xE0, 0xE8], // border_subtle
                [0xC0, 0xC8, 0xD0], // border_strong
                [0x15, 0x00, 0x20], // text_primary
                [0x30, 0x20, 0x35], // text_secondary
                [0x50, 0x40, 0x45], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x67, 0x3A, 0xB7], // accent
                [0x5B, 0x21, 0xBD], // accent_hover
                [0xD1, 0xC4, 0xE9], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 10. Retro Terminal - Retro
        self.register(ThemeDefinition {
            id: "retro_terminal".to_string(),
            name: "Retro Terminal".to_string(),
            description: "Classic terminal theme with green phosphor".to_string(),
            category: ThemeCategory::Retro,
            dark_colors: ThemeColors::from_array([
                [0x00, 0x08, 0x00], // bg_base
                [0x00, 0x10, 0x00], // bg_surface
                [0x00, 0x18, 0x00], // bg_surface_raised
                [0x00, 0x05, 0x00], // bg_surface_sunken
                [0x00, 0x20, 0x00], // border_subtle
                [0x00, 0x30, 0x00], // border_strong
                [0x00, 0xFF, 0x00], // text_primary
                [0x00, 0xC0, 0x00], // text_secondary
                [0x00, 0x80, 0x00], // text_tertiary
                [0x00, 0x05, 0x00], // text_inverse
                [0x00, 0xFF, 0x00], // accent
                [0x00, 0xFF, 0x00], // accent_hover
                [0x00, 0x50, 0x00], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xF0, 0xFF, 0xF0], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xE8, 0xFF, 0xE8], // bg_surface_raised
                [0xE0, 0xFF, 0xE0], // bg_surface_sunken
                [0xD8, 0xFF, 0xD8], // border_subtle
                [0xC0, 0xFF, 0xC0], // border_strong
                [0x00, 0x20, 0x00], // text_primary
                [0x00, 0x40, 0x00], // text_secondary
                [0x00, 0x60, 0x00], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x2E, 0x7D, 0x32], // accent
                [0x1E, 0x6D, 0x22], // accent_hover
                [0xC8, 0xE6, 0xC9], // accent_muted
            ]),
            default_font: FontConfig {
                family: super::fonts::FontFamily::JetBrainsMono,
                ..Default::default()
            },
            is_premium: true,
        });

        // 11. High Contrast - High Contrast
        self.register(ThemeDefinition {
            id: "high_contrast".to_string(),
            name: "High Contrast".to_string(),
            description: "Maximum contrast for accessibility".to_string(),
            category: ThemeCategory::HighContrast,
            dark_colors: ThemeColors::from_array([
                [0x00, 0x00, 0x00], // bg_base
                [0x00, 0x00, 0x00], // bg_surface
                [0x00, 0x00, 0x00], // bg_surface_raised
                [0x00, 0x00, 0x00], // bg_surface_sunken
                [0xFF, 0xFF, 0xFF], // border_subtle
                [0xFF, 0xFF, 0xFF], // border_strong
                [0xFF, 0xFF, 0xFF], // text_primary
                [0xFF, 0xFF, 0xFF], // text_secondary
                [0xFF, 0xFF, 0xFF], // text_tertiary
                [0x00, 0x00, 0x00], // text_inverse
                [0xFF, 0xFF, 0x00], // accent
                [0xFF, 0xFF, 0x00], // accent_hover
                [0x80, 0x80, 0x00], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xFF, 0xFF, 0xFF], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xFF, 0xFF, 0xFF], // bg_surface_raised
                [0xFF, 0xFF, 0xFF], // bg_surface_sunken
                [0x00, 0x00, 0x00], // border_subtle
                [0x00, 0x00, 0x00], // border_strong
                [0x00, 0x00, 0x00], // text_primary
                [0x00, 0x00, 0x00], // text_secondary
                [0x00, 0x00, 0x00], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0x00, 0x00, 0x00], // accent
                [0x00, 0x00, 0x00], // accent_hover
                [0x80, 0x80, 0x80], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });

        // 12. Golden Hour - Elegant
        self.register(ThemeDefinition {
            id: "golden_hour".to_string(),
            name: "Golden Hour".to_string(),
            description: "Warm golden hour theme with luxury feel".to_string(),
            category: ThemeCategory::Elegant,
            dark_colors: ThemeColors::from_array([
                [0x1A, 0x15, 0x08], // bg_base
                [0x25, 0x1E, 0x10], // bg_surface
                [0x30, 0x28, 0x18], // bg_surface_raised
                [0x12, 0x0A, 0x05], // bg_surface_sunken
                [0x38, 0x30, 0x20], // border_subtle
                [0x48, 0x40, 0x30], // border_strong
                [0xFF, 0xF0, 0xD6], // text_primary
                [0xE0, 0xC8, 0xA0], // text_secondary
                [0xA0, 0x80, 0x50], // text_tertiary
                [0x10, 0x08, 0x00], // text_inverse
                [0xFF, 0xB3, 0x00], // accent
                [0xFF, 0xC3, 0x00], // accent_hover
                [0x8B, 0x69, 0x14], // accent_muted
            ]),
            light_colors: ThemeColors::from_array([
                [0xFF, 0xFA, 0xF0], // bg_base
                [0xFF, 0xFF, 0xFF], // bg_surface
                [0xFF, 0xF0, 0xE0], // bg_surface_raised
                [0xFF, 0xE8, 0xD8], // bg_surface_sunken
                [0xFF, 0xE0, 0xD0], // border_subtle
                [0xE8, 0xD0, 0xB8], // border_strong
                [0x2A, 0x18, 0x00], // text_primary
                [0x4A, 0x38, 0x20], // text_secondary
                [0x6A, 0x58, 0x40], // text_tertiary
                [0xFF, 0xFF, 0xFF], // text_inverse
                [0xF5, 0x7F, 0x00], // accent
                [0xD4, 0x64, 0x04], // accent_hover
                [0xFF, 0xE0, 0xB3], // accent_muted
            ]),
            default_font: FontConfig::default(),
            is_premium: true,
        });
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
