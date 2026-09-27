use serde::{Deserialize, Serialize};

pub const DEFAULT_LEFT_WIDTH: f32 = 340.0;
pub const DEFAULT_RIGHT_WIDTH: f32 = 320.0;
pub const DEFAULT_MIDDLE_BOTTOM_HEIGHT: f32 = 172.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutState {
    pub left_collapsed: bool,
    pub left_width: f32,
    pub right_collapsed: bool,
    pub right_width: f32,
    pub middle_bottom_collapsed: bool,
    pub middle_bottom_height: f32,
    pub ribbon_collapsed: bool,
    #[serde(default)]
    pub settings_open: bool,
    #[serde(default)]
    pub settings_tab: usize,
    /// Open/closed state for every section in the left audio column, so the
    /// operator's arrangement survives a restart. Persisted because it is a
    /// workspace preference, not a transient view state.
    #[serde(default)]
    pub audio_sections: AudioSections,
}

/// Disclosure state for the left column's six sections.
///
/// The operator sets up a console the way they like it and expects to find it
/// that way on Sunday morning, exactly like a mix window. `Copy` because the
/// column mirrors it into locals each frame and writes it back.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AudioSections {
    pub input_device: bool,
    pub input_level: bool,
    pub gain: bool,
    pub voice: bool,
    pub preset: bool,
    pub led: bool,
}

impl Default for AudioSections {
    /// The two things an operator checks mid-service start open: which input is
    /// live, and how hot it is running. The rest are deliberate.
    fn default() -> Self {
        Self {
            input_device: true,
            input_level: true,
            gain: false,
            voice: false,
            preset: false,
            led: false,
        }
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            left_collapsed: false,
            left_width: DEFAULT_LEFT_WIDTH,
            right_collapsed: false,
            right_width: DEFAULT_RIGHT_WIDTH,
            middle_bottom_collapsed: false,
            middle_bottom_height: DEFAULT_MIDDLE_BOTTOM_HEIGHT,
            ribbon_collapsed: false,
            settings_open: false,
            settings_tab: 0,
            audio_sections: AudioSections::default(),
        }
    }
}
