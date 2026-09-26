//! Color palette definitions for themes.

use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub bg_base: [u8; 3],
    pub bg_surface: [u8; 3],
    pub bg_surface_raised: [u8; 3],
    pub bg_surface_sunken: [u8; 3],
    pub border_subtle: [u8; 3],
    pub border_strong: [u8; 3],
    pub text_primary: [u8; 3],
    pub text_secondary: [u8; 3],
    pub text_tertiary: [u8; 3],
    pub text_inverse: [u8; 3],
    pub accent: [u8; 3],
    pub accent_hover: [u8; 3],
    pub accent_muted: [u8; 3],
}

impl ThemeColors {
    pub fn from_array(arr: [[u8; 3]; 13]) -> Self {
        Self {
            bg_base: arr[0],
            bg_surface: arr[1],
            bg_surface_raised: arr[2],
            bg_surface_sunken: arr[3],
            border_subtle: arr[4],
            border_strong: arr[5],
            text_primary: arr[6],
            text_secondary: arr[7],
            text_tertiary: arr[8],
            text_inverse: arr[9],
            accent: arr[10],
            accent_hover: arr[11],
            accent_muted: arr[12],
        }
    }

    pub fn to_color32(&self) -> ColorPalette32 {
        ColorPalette32 {
            bg_base: Color32::from_rgb(self.bg_base[0], self.bg_base[1], self.bg_base[2]),
            bg_surface: Color32::from_rgb(self.bg_surface[0], self.bg_surface[1], self.bg_surface[2]),
            bg_surface_raised: Color32::from_rgb(self.bg_surface_raised[0], self.bg_surface_raised[1], self.bg_surface_raised[2]),
            bg_surface_sunken: Color32::from_rgb(self.bg_surface_sunken[0], self.bg_surface_sunken[1], self.bg_surface_sunken[2]),
            border_subtle: Color32::from_rgb(self.border_subtle[0], self.border_subtle[1], self.border_subtle[2]),
            border_strong: Color32::from_rgb(self.border_strong[0], self.border_strong[1], self.border_strong[2]),
            text_primary: Color32::from_rgb(self.text_primary[0], self.text_primary[1], self.text_primary[2]),
            text_secondary: Color32::from_rgb(self.text_secondary[0], self.text_secondary[1], self.text_secondary[2]),
            text_tertiary: Color32::from_rgb(self.text_tertiary[0], self.text_tertiary[1], self.text_tertiary[2]),
            text_inverse: Color32::from_rgb(self.text_inverse[0], self.text_inverse[1], self.text_inverse[2]),
            accent: Color32::from_rgb(self.accent[0], self.accent[1], self.accent[2]),
            accent_hover: Color32::from_rgb(self.accent_hover[0], self.accent_hover[1], self.accent_hover[2]),
            accent_muted: Color32::from_rgb(self.accent_muted[0], self.accent_muted[1], self.accent_muted[2]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub bg_base: [u8; 3],
    pub bg_surface: [u8; 3],
    pub bg_surface_raised: [u8; 3],
    pub bg_surface_sunken: [u8; 3],
    pub border_subtle: [u8; 3],
    pub border_strong: [u8; 3],
    pub text_primary: [u8; 3],
    pub text_secondary: [u8; 3],
    pub text_tertiary: [u8; 3],
    pub text_inverse: [u8; 3],
    pub accent: [u8; 3],
    pub accent_hover: [u8; 3],
    pub accent_muted: [u8; 3],
}

impl ColorPalette {
    pub fn to_color32(&self) -> ColorPalette32 {
        ColorPalette32 {
            bg_base: Color32::from_rgb(self.bg_base[0], self.bg_base[1], self.bg_base[2]),
            bg_surface: Color32::from_rgb(self.bg_surface[0], self.bg_surface[1], self.bg_surface[2]),
            bg_surface_raised: Color32::from_rgb(self.bg_surface_raised[0], self.bg_surface_raised[1], self.bg_surface_raised[2]),
            bg_surface_sunken: Color32::from_rgb(self.bg_surface_sunken[0], self.bg_surface_sunken[1], self.bg_surface_sunken[2]),
            border_subtle: Color32::from_rgb(self.border_subtle[0], self.border_subtle[1], self.border_subtle[2]),
            border_strong: Color32::from_rgb(self.border_strong[0], self.border_strong[1], self.border_strong[2]),
            text_primary: Color32::from_rgb(self.text_primary[0], self.text_primary[1], self.text_primary[2]),
            text_secondary: Color32::from_rgb(self.text_secondary[0], self.text_secondary[1], self.text_secondary[2]),
            text_tertiary: Color32::from_rgb(self.text_tertiary[0], self.text_tertiary[1], self.text_tertiary[2]),
            text_inverse: Color32::from_rgb(self.text_inverse[0], self.text_inverse[1], self.text_inverse[2]),
            accent: Color32::from_rgb(self.accent[0], self.accent[1], self.accent[2]),
            accent_hover: Color32::from_rgb(self.accent_hover[0], self.accent_hover[1], self.accent_hover[2]),
            accent_muted: Color32::from_rgb(self.accent_muted[0], self.accent_muted[1], self.accent_muted[2]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorPalette32 {
    pub bg_base: Color32,
    pub bg_surface: Color32,
    pub bg_surface_raised: Color32,
    pub bg_surface_sunken: Color32,
    pub border_subtle: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    pub text_inverse: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_muted: Color32,
}

impl Default for ColorPalette {
    fn default() -> Self {
        // Default to Purple Graphite (Dark)
        ColorPalette {
            bg_base: [0x15, 0x13, 0x1D],
            bg_surface: [0x20, 0x1D, 0x2A],
            bg_surface_raised: [0x2A, 0x25, 0x36],
            bg_surface_sunken: [0x11, 0x10, 0x18],
            border_subtle: [0x36, 0x30, 0x46],
            border_strong: [0x4B, 0x42, 0x61],
            text_primary: [0xEC, 0xE9, 0xF3],
            text_secondary: [0xB4, 0xAD, 0xC4],
            text_tertiary: [0x77, 0x6F, 0x88],
            text_inverse: [0x12, 0x10, 0x18],
            accent: [0xA7, 0x78, 0xF2],
            accent_hover: [0xB8, 0x8F, 0xFF],
            accent_muted: [0x4F, 0x39, 0x73],
        }
    }
}
