#[derive(Debug, Clone)]
pub struct BroadcastState {
    pub program_output: &'static str,
    pub clean_feed: bool,
    pub ndi_enabled: bool,
    pub lower_third: bool,
    pub confidence_gate: u8,
    pub latency_ms: u16,
}

impl BroadcastState {
    pub fn mock_live() -> Self {
        Self {
            program_output: "Second Display 1920x1080",
            clean_feed: true,
            ndi_enabled: false,
            lower_third: false,
            confidence_gate: 70,
            latency_ms: 142,
        }
    }
}
