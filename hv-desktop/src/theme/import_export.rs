//! Theme import/export functionality.

use super::colors::ThemeColors;
use super::fonts::FontConfig;
use super::registry::{ThemeDefinition, ThemeId};
use super::theme::{Theme, ThemeMode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeExport {
    pub name: String,
    pub description: String,
    pub category: String,
    pub dark_colors: ThemeColors,
    pub light_colors: ThemeColors,
    pub font_config: FontConfig,
}

impl ThemeExport {
    pub fn from_definition(definition: &ThemeDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            category: format!("{:?}", definition.category),
            dark_colors: definition.dark_colors.clone(),
            light_colors: definition.light_colors.clone(),
            font_config: definition.default_font.clone(),
        }
    }

    pub fn to_definition(&self, id: ThemeId) -> ThemeDefinition {
        let category = match self.category.as_str() {
            "Professional" => super::registry::ThemeCategory::Professional,
            "Elegant" => super::registry::ThemeCategory::Elegant,
            "Vibrant" => super::registry::ThemeCategory::Vibrant,
            "Minimal" => super::registry::ThemeCategory::Minimal,
            "HighContrast" => super::registry::ThemeCategory::HighContrast,
            "Broadcast" => super::registry::ThemeCategory::Broadcast,
            "Nature" => super::registry::ThemeCategory::Nature,
            "Retro" => super::registry::ThemeCategory::Retro,
            "Cyberpunk" => super::registry::ThemeCategory::Cyberpunk,
            "Custom" => super::registry::ThemeCategory::Custom,
            _ => super::registry::ThemeCategory::Custom,
        };

        ThemeDefinition {
            id,
            name: self.name.clone(),
            description: self.description.clone(),
            category,
            dark_colors: self.dark_colors.clone(),
            light_colors: self.light_colors.clone(),
            default_font: self.font_config.clone(),
            is_premium: false, // Imported themes are not premium
        }
    }

    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let theme_export: Self = serde_json::from_str(&json)?;
        Ok(theme_export)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserThemeConfig {
    pub current_theme_id: ThemeId,
    pub theme_mode: ThemeMode,
    pub custom_themes: Vec<ThemeExport>,
}

impl Default for UserThemeConfig {
    fn default() -> Self {
        Self {
            current_theme_id: "purple_graphite_dark".to_string(),
            theme_mode: ThemeMode::Dark,
            custom_themes: Vec::new(),
        }
    }
}

impl UserThemeConfig {
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let json = fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&json)?;
        Ok(config)
    }

    pub fn add_custom_theme(&mut self, theme: ThemeExport) {
        // Check if theme with same name already exists
        if let Some(pos) = self.custom_themes.iter().position(|t| t.name == theme.name) {
            self.custom_themes[pos] = theme;
        } else {
            self.custom_themes.push(theme);
        }
    }

    pub fn remove_custom_theme(&mut self, name: &str) {
        self.custom_themes.retain(|t| t.name != name);
    }

    pub fn get_custom_theme(&self, name: &str) -> Option<&ThemeExport> {
        self.custom_themes.iter().find(|t| t.name == name)
    }
}
