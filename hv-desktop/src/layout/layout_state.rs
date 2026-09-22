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
        }
    }
}
