#[derive(Debug, Clone)]
pub struct WorkspacePreset {
    pub name: &'static str,
    pub left_weight: f32,
    pub center_weight: f32,
    pub right_weight: f32,
    pub bottom_height: f32,
    pub right_tabs: [&'static str; 4],
    pub bottom_tabs: [&'static str; 3],
}

impl WorkspacePreset {
    pub fn default_broadcast() -> Self {
        Self {
            name: "Operator Broadcast",
            left_weight: 0.22,
            center_weight: 0.54,
            right_weight: 0.24,
            bottom_height: 172.0,
            right_tabs: ["Run Sheet", "Log", "Queue", "Detected"],
            bottom_tabs: ["Transcript", "Metrics", "Events"],
        }
    }
}
