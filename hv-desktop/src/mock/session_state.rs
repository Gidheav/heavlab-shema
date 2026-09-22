#[derive(Debug, Clone)]
pub struct ServiceBlock {
    pub time: &'static str,
    pub title: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

pub fn mock_run_sheet() -> [ServiceBlock; 6] {
    [
        ServiceBlock {
            time: "09:00",
            title: "Opening Worship",
            status: "Done",
            detail: "Psalm 100 queued",
        },
        ServiceBlock {
            time: "09:18",
            title: "Welcome",
            status: "Done",
            detail: "Lower third clear",
        },
        ServiceBlock {
            time: "09:32",
            title: "Scripture Reading",
            status: "Live",
            detail: "John 3:16 on Program",
        },
        ServiceBlock {
            time: "09:45",
            title: "Message",
            status: "Armed",
            detail: "Auto-detect enabled",
        },
        ServiceBlock {
            time: "10:21",
            title: "Altar Call",
            status: "Next",
            detail: "Romans 8:28 pending",
        },
        ServiceBlock {
            time: "10:35",
            title: "Closing",
            status: "Queued",
            detail: "Benediction slide",
        },
    ]
}

#[derive(Debug, Clone)]
pub struct DetectedReference {
    pub reference: &'static str,
    pub confidence: u8,
    pub source: &'static str,
    pub state: &'static str,
}

pub fn mock_detected_references() -> [DetectedReference; 5] {
    [
        DetectedReference {
            reference: "John 3:16",
            confidence: 94,
            source: "ASR",
            state: "Displayed",
        },
        DetectedReference {
            reference: "Romans 8:28",
            confidence: 91,
            source: "Queue",
            state: "Ready",
        },
        DetectedReference {
            reference: "Psalm 23:4",
            confidence: 68,
            source: "ASR",
            state: "Review",
        },
        DetectedReference {
            reference: "Isaiah 40:31",
            confidence: 88,
            source: "Manual",
            state: "Ready",
        },
        DetectedReference {
            reference: "John 14:6",
            confidence: 97,
            source: "ASR",
            state: "Logged",
        },
    ]
}
