//! Font customization system.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FontFamily {
    Inter,
    JetBrainsMono,
    FiraCode,
    Roboto,
    OpenSans,
    System,
    Custom(String),
}

impl fmt::Display for FontFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontFamily::Inter => write!(f, "Inter"),
            FontFamily::JetBrainsMono => write!(f, "JetBrains Mono"),
            FontFamily::FiraCode => write!(f, "Fira Code"),
            FontFamily::Roboto => write!(f, "Roboto"),
            FontFamily::OpenSans => write!(f, "Open Sans"),
            FontFamily::System => write!(f, "System"),
            FontFamily::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
    Bold,
    BoldItalic,
}

impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontStyle::Normal => write!(f, "Normal"),
            FontStyle::Italic => write!(f, "Italic"),
            FontStyle::Bold => write!(f, "Bold"),
            FontStyle::BoldItalic => write!(f, "Bold Italic"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FontSize {
    XSmall,   // 10px
    Small,    // 12px
    Medium,   // 14px
    Large,    // 16px
    XLarge,   // 18px
    XXLarge,  // 20px
    XXXLarge, // 24px
    Custom(f32),
}

impl PartialEq for FontSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FontSize::Custom(a), FontSize::Custom(b)) => (a - b).abs() < 0.001,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

impl FontSize {
    pub fn to_pixels(&self) -> f32 {
        match self {
            FontSize::XSmall => 10.0,
            FontSize::Small => 12.0,
            FontSize::Medium => 14.0,
            FontSize::Large => 16.0,
            FontSize::XLarge => 18.0,
            FontSize::XXLarge => 20.0,
            FontSize::XXXLarge => 24.0,
            FontSize::Custom(size) => *size,
        }
    }

    pub fn from_pixels(size: f32) -> Self {
        match size {
            s if s <= 11.0 => FontSize::XSmall,
            s if s <= 13.0 => FontSize::Small,
            s if s <= 15.0 => FontSize::Medium,
            s if s <= 17.0 => FontSize::Large,
            s if s <= 19.0 => FontSize::XLarge,
            s if s <= 22.0 => FontSize::XXLarge,
            s if s <= 26.0 => FontSize::XXXLarge,
            _ => FontSize::Custom(size),
        }
    }
}

impl fmt::Display for FontSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontSize::XSmall => write!(f, "10px"),
            FontSize::Small => write!(f, "12px"),
            FontSize::Medium => write!(f, "14px"),
            FontSize::Large => write!(f, "16px"),
            FontSize::XLarge => write!(f, "18px"),
            FontSize::XXLarge => write!(f, "20px"),
            FontSize::XXXLarge => write!(f, "24px"),
            FontSize::Custom(size) => write!(f, "{:.1}px", size),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: FontFamily,
    pub size: FontSize,
    pub style: FontStyle,
    pub ui_scale: f32, // 0.5 - 2.0
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: FontFamily::Inter,
            size: FontSize::Medium,
            style: FontStyle::Normal,
            ui_scale: 1.0,
        }
    }
}

impl FontConfig {
    pub fn with_family(mut self, family: FontFamily) -> Self {
        self.family = family;
        self
    }

    pub fn with_size(mut self, size: FontSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.ui_scale = scale.clamp(0.5, 2.0);
        self
    }
}
