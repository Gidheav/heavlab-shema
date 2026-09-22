#![allow(non_snake_case)]
use dioxus::prelude::*;
use bible_core::store::TranslationStore;
use bible_core::parser::resolve_text;
use bible_asr::worker::{AudioWorker, TranscriptHandler};
use bible_asr::{AsrModelConfig, SherpaAsrEngine};
use bible_asr::CpalCapture;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{unbounded, Sender, Receiver};

mod paths;
mod verse_detector;

fn main() {
    dioxus::launch(App, dioxus::desktop::Config::new()
        .with_window(dioxus::desktop::WindowConfig::default()
            .with_title("HV-Bible — Audio Companion")
            .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
            .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0))
        )
    );
}

#[derive(Clone)]
struct AppState {
    store: Arc<TranslationStore>,
}

enum AudioEvent {
    Transcript(String),
    VerseDetected(String, String),
    Error(String),
    StateChanged(bool),
    Paused,
    Resumed,
}

struct DesktopTranscriptHandler {
    sender: Sender<AudioEvent>,
    store: Arc<TranslationStore>,
}

impl TranscriptHandler for DesktopTranscriptHandler {
    fn on_transcript(&self, text: String) {
        let _ = self.sender.send(AudioEvent::Transcript(text.clone()));
        
        if let Ok(verse_ref) = resolve_text(&text) {
            let book_id = verse_ref.book;
            if let Ok(index) = bible_core::canon::resolve_index(book_id, verse_ref.chapter, verse_ref.verse) {
                if let Ok(verse_text) = self.store.get_verse("KJV", index) {
                    let ref_str = format!("{} {}:{}", bible_core::canon::book_name(book_id), verse_ref.chapter, verse_ref.verse);
                    let _ = self.sender.send(AudioEvent::VerseDetected(ref_str, verse_text));
                }
            }
        }
    }

    fn on_error(&self, error: String) {
        let _ = self.sender.send(AudioEvent::Error(error));
    }

    fn on_listening_state_changed(&self, is_active: bool) {
        let _ = self.sender.send(AudioEvent::StateChanged(is_active));
    }

    fn on_listening_paused(&self) {
        let _ = self.sender.send(AudioEvent::Paused);
    }

    fn on_speech_resumed(&self) {
        let _ = self.sender.send(AudioEvent::Resumed);
    }
}

const CSS: &str = r#"
body { 
    background: linear-gradient(135deg, #16213e, #0f3460); 
    color: #e9ecef; 
    font-family: 'Inter', -apple-system, sans-serif; 
    margin: 0; 
    min-height: 100vh; 
    display: flex; 
    flex-direction: column; 
}

.app-container {
    display: flex;
    flex-direction: row;
    height: 100vh;
    gap: 1rem;
    padding: 1rem;
}

/* Left panel - sidebar */
.sidebar {
    width: 250px;
    background: rgba(0,0,0,0.4);
    padding: 1.5rem;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
    border: 1px solid rgba(255,255,255,0.1);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.sidebar-title {
    font-size: 1.1rem;
    font-weight: 600;
    color: #fff;
    margin: 0;
    border-bottom: 1px solid rgba(255,255,255,0.1);
    padding-bottom: 0.5rem;
}

.translation-selector {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.translation-selector label {
    font-size: 0.9rem;
    color: #a9b0c0;
}

.translation-selector select {
    padding: 0.5rem;
    border-radius: 6px;
    border: 1px solid rgba(255,255,255,0.2);
    background: rgba(0,0,0,0.2);
    color: white;
    font-size: 0.9rem;
}

.model-status {
    padding: 0.75rem;
    background: rgba(40, 167, 69, 0.2);
    border-radius: 8px;
    border: 1px solid rgba(40, 167, 69, 0.3);
}

.model-status-text {
    font-size: 0.85rem;
    color: #28a745;
    margin: 0;
}

.sermon-controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.listen-button {
    background: #e94560;
    color: white;
    border: none;
    padding: 1rem;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
}

.listen-button:hover {
    background: #ff5272;
    transform: translateY(-1px);
}

.listen-button.listening {
    background: #ffc107;
    color: #333;
}

.status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: rgba(255,255,255,0.05);
    border-radius: 8px;
}

.status-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #6c757d;
}

.status-dot.listening {
    background: #28a745;
}

.status-dot.error {
    background: #dc3545;
}

.status-text {
    font-size: 0.9rem;
    color: #a9b0c0;
}

.keyboard-shortcuts {
    font-size: 0.8rem;
    color: #6c757d;
    line-height: 1.4;
}

.keyboard-shortcuts code {
    background: rgba(255,255,255,0.1);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    font-family: 'Fira Code', monospace;
}

/* Center panel - main content */
.main-content {
    flex: 1;
    background: rgba(0,0,0,0.4);
    padding: 2rem;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
    border: 1px solid rgba(255,255,255,0.1);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.verse-display {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

.verse-reference {
    font-size: 2rem;
    font-weight: 700;
    color: #e94560;
    margin-bottom: 1rem;
}

.verse-text {
    font-size: 1.75rem;
    line-height: 1.8;
    font-family: 'Merriweather', 'Georgia', serif;
    color: #fff;
    max-width: 800px;
}

.manual-input-section {
    display: flex;
    gap: 0.5rem;
}

.manual-input-section input {
    flex: 1;
    padding: 0.75rem;
    border-radius: 8px;
    border: 1px solid rgba(255,255,255,0.2);
    background: rgba(0,0,0,0.2);
    color: white;
    font-size: 1rem;
}

.manual-input-section input:focus {
    outline: none;
    border-color: #e94560;
}

.manual-input-section button {
    background: #e94560;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
}

.manual-input-section button:hover {
    background: #ff5272;
}

.transcript-display {
    font-family: 'Fira Code', monospace;
    font-size: 0.85rem;
    color: #8e9bb0;
    background: rgba(0,0,0,0.3);
    padding: 1rem;
    border-radius: 8px;
    max-height: 100px;
    overflow-y: auto;
}

.transcript-label {
    font-size: 0.75rem;
    color: #6c757d;
    margin-bottom: 0.5rem;
    text-transform: uppercase;
    letter-spacing: 1px;
}

/* Right panel - verse history */
.history-panel {
    width: 300px;
    background: rgba(0,0,0,0.4);
    padding: 1.5rem;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
    border: 1px solid rgba(255,255,255,0.1);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.history-title {
    font-size: 1.1rem;
    font-weight: 600;
    color: #fff;
    margin: 0;
    border-bottom: 1px solid rgba(255,255,255,0.1);
    padding-bottom: 0.5rem;
}

.history-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.history-item {
    padding: 0.75rem;
    background: rgba(255,255,255,0.05);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    border: 1px solid transparent;
}

.history-item:hover {
    background: rgba(255,255,255,0.1);
    border-color: rgba(233, 69, 96, 0.3);
}

.history-reference {
    font-size: 0.9rem;
    font-weight: 600;
    color: #e94560;
    margin-bottom: 0.25rem;
}

.history-text {
    font-size: 0.8rem;
    color: #a9b0c0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.history-time {
    font-size: 0.7rem;
    color: #6c757d;
    margin-top: 0.25rem;
}

.error-message {
    padding: 1rem;
    background: rgba(220, 53, 69, 0.2);
    border: 1px solid rgba(220, 53, 69, 0.3);
    border-radius: 8px;
    color: #dc3545;
    font-size: 0.9rem;
}

/* Responsive */
@media (max-width: 1024px) {
    .app-container {
        flex-direction: column;
    }
    
    .sidebar, .history-panel {
        width: 100%;
    }
}
"#;

fn resolve_verse_text(text: &str, store: Arc<TranslationStore>) -> Option<(String, String)> {
    if let Ok(verse_ref) = resolve_text(text) {
        let book_id = verse_ref.book;
        if let Ok(index) = bible_core::canon::resolve_index(book_id, verse_ref.chapter, verse_ref.verse) {
            if let Ok(text) = store.get_verse("KJV", index) {
                let ref_str = format!("{} {}:{}", bible_core::canon::book_name(book_id), verse_ref.chapter, verse_ref.verse);
                return Some((ref_str, text));
            }
        }
    }
    None
}

fn App() -> Element {
    use_context_provider(|| {
        let store = TranslationStore::new();

        // Resolve paths using the paths module
        let packs_dir = paths::packs_dir();
        let pack_path = packs_dir.join("KJV.bible.bin");
        let offsets_path = packs_dir.join("KJV.offsets.bin");

        if let Err(e) = store.load_translation("KJV", &pack_path, &offsets_path) {
            eprintln!("Failed to load KJV: {}", e);
        }

        AppState {
            store: Arc::new(store),
        }
    });

    let state = use_context::<AppState>();
    
    let mut manual_input = use_signal(String::new);
    let mut current_verse = use_signal(|| String::from("Welcome to HV-Bible"));
    let mut current_ref = use_signal(|| String::from("Audio Companion"));
    let mut partial_transcript = use_signal(|| String::from("Waiting for speech..."));
    
    let mut is_listening = use_signal(|| false);
    let mut status_msg = use_signal(|| String::from("Stopped"));
    let mut status_color = use_signal(|| "#6c757d");
    let mut error_message = use_signal(|| String::new());
    
    let mut verse_history = use_signal(|| Vec::new());
    
    let mut worker: Signal<Option<Arc<Mutex<AudioWorker>>>> = use_signal(|| None);
    
    let (tx, rx): (Sender<AudioEvent>, Receiver<AudioEvent>) = use_hook(unbounded);

    use_coroutine(move |mut _rx: UnboundedReceiver<()>| {
        let rx_clone = rx.clone();
        async move {
            loop {
                if let Ok(event) = rx_clone.try_recv() {
                    match event {
                        AudioEvent::Transcript(text) => partial_transcript.set(text),
                        AudioEvent::VerseDetected(reference, text) => {
                            current_ref.set(reference.clone());
                            current_verse.set(text.clone());
                            error_message.set(String::new());
                            
                            // Add to history
                            let mut history = verse_history();
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            history.insert(0, (reference, text, timestamp));
                            if history.len() > 50 {
                                history.pop();
                            }
                            verse_history.set(history);
                        }
                        AudioEvent::Error(err) => {
                            error_message.set(format!("Error: {}", err));
                            status_msg.set("Error".to_string());
                            status_color.set("#dc3545");
                        }
                        AudioEvent::StateChanged(active) => {
                            is_listening.set(active);
                            if active {
                                status_msg.set("Listening".to_string());
                                status_color.set("#28a745");
                                error_message.set(String::new());
                            } else {
                                status_msg.set("Stopped".to_string());
                                status_color.set("#6c757d");
                            }
                        }
                        AudioEvent::Paused => {
                            status_msg.set("Paused (Silence)".to_string());
                            status_color.set("#ffc107");
                        }
                        AudioEvent::Resumed => {
                            status_msg.set("Listening".to_string());
                            status_color.set("#28a745");
                        }
                    }
                }
                // Use dioxus's built-in timing instead of tokio
                dioxus::prelude::wait_for_next_render().await;
            }
        }
    });

    let store_for_worker = state.store.clone();
    let toggle_sermon_mode = move |_| {
        if is_listening() {
            if let Some(w) = worker.read().as_ref() {
                if let Ok(mut lock) = w.lock() {
                    let _ = lock.stop();
                }
            }
            worker.set(None);
        } else {
            let mut w = AudioWorker::with_defaults();
            let handler = Arc::new(DesktopTranscriptHandler {
                sender: tx.clone(),
                store: store_for_worker.clone(),
            });
            
            let capture = Arc::new(CpalCapture::new());

            // Set up real Sherpa-ONNX model
            let model_dir = paths::model_path("sherpa-onnx-streaming-zipformer-en-20M-2023-02-17");

            let asr_config = AsrModelConfig {
                encoder_path: model_dir.join("encoder-epoch-99-avg-1.onnx").to_string_lossy().to_string(),
                decoder_path: model_dir.join("decoder-epoch-99-avg-1.onnx").to_string_lossy().to_string(),
                joiner_path: model_dir.join("joiner-epoch-99-avg-1.onnx").to_string_lossy().to_string(),
                tokens_path: model_dir.join("tokens.txt").to_string_lossy().to_string(),
                language: "en".to_string(),
                num_threads: 1,
                enable_vad: false,
                vad_model_path: None,
                vad_threshold: 0.6,
                vad_silence_duration: 1.5,
            };
            
            match SherpaAsrEngine::new(asr_config) {
                Ok(engine) => {
                    if let Err(e) = w.start_with(handler, capture, Box::new(engine), false) {
                        error_message.set(format!("Failed to start: {}", e));
                        status_msg.set("Error".to_string());
                        status_color.set("#dc3545");
                    } else {
                        worker.set(Some(Arc::new(Mutex::new(w))));
                    }
                },
                Err(e) => {
                    error_message.set(format!("Model failed to load: {}", e));
                    status_msg.set("Error".to_string());
                    status_color.set("#dc3545");
                }
            }
        }
    };

    let store_for_key = state.store.clone();
    let store_for_btn = state.store.clone();
    let current_ref_for_history = current_ref.clone();
    let current_verse_for_history = current_verse.clone();

    let select_history_item = move |(ref_str, text, _timestamp): (String, String, u64)| {
        current_ref_for_history.set(ref_str);
        current_verse_for_history.set(text);
    };

    rsx! {
        style { "{CSS}" }
        div { class: "app-container",
            // Left panel - sidebar
            div { class: "sidebar",
                h2 { class: "sidebar-title", "Settings" }
                
                div { class: "translation-selector",
                    label { "Translation" }
                    select {
                        option { "KJV" }
                    }
                }
                
                div { class: "model-status",
                    p { class: "model-status-text", "Model: Loaded" }
                }
                
                div { class: "sermon-controls",
                    button { 
                        class: if is_listening() { "listening" } else { "" },
                        onclick: toggle_sermon_mode,
                        if is_listening() { "Stop Listening" } else { "Start Listening" }
                    }
                    
                    div { class: "status-indicator",
                        div { 
                            class: if is_listening() { "status-dot listening" } else { "status-dot" },
                            style: if !error_message().is_empty() { "background-color: #dc3545;" } else { "" }
                        }
                        span { class: "status-text", "{status_msg}" }
                    }
                }
                
                div { class: "keyboard-shortcuts",
                    p { "Keyboard shortcuts:" }
                    p { code { "Ctrl+L" } " - Toggle listening" }
                    p { code { "Escape" } " - Stop listening" }
                    p { code { "Ctrl+K" } " - Focus input" }
                }
            }
            
            // Center panel - main content
            div { class: "main-content",
                if !error_message().is_empty() {
                    div { class: "error-message", "{error_message}" }
                }
                
                div { class: "verse-display",
                    div { class: "verse-reference", "{current_ref}" }
                    div { class: "verse-text", "{current_verse}" }
                }
                
                div { class: "manual-input-section",
                    input {
                        placeholder: "Type a reference (e.g. John 3:16)",
                        value: "{manual_input}",
                        oninput: move |e| manual_input.set(e.value().clone()),
                        onkeydown: move |e| {
                            if e.key() == dioxus::html::input_data::keyboard_types::Key::Enter {
                                let text = manual_input();
                                if let Some((r, t)) = resolve_verse_text(&text, store_for_key.clone()) {
                                    current_ref.set(r.clone());
                                    current_verse.set(t.clone());
                                    error_message.set(String::new());
                                    
                                    // Add to history
                                    let mut history = verse_history();
                                    let timestamp = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    history.insert(0, (r, t, timestamp));
                                    if history.len() > 50 {
                                        history.pop();
                                    }
                                    verse_history.set(history);
                                } else {
                                    error_message.set("No valid verse found in that text.".to_string());
                                }
                            }
                        }
                    }
                    button { 
                        onclick: move |_| {
                            let text = manual_input();
                            if let Some((r, t)) = resolve_verse_text(&text, store_for_btn.clone()) {
                                current_ref.set(r.clone());
                                current_verse.set(t.clone());
                                error_message.set(String::new());
                                
                                // Add to history
                                let mut history = verse_history();
                                let timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                history.insert(0, (r, t, timestamp));
                                if history.len() > 50 {
                                    history.pop();
                                }
                                verse_history.set(history);
                            } else {
                                error_message.set("No valid verse found in that text.".to_string());
                            }
                        }, 
                        "Resolve" 
                    }
                }
                
                div { class: "transcript-display",
                    div { class: "transcript-label", "Live Transcript" }
                    "{partial_transcript}"
                }
            }
            
            // Right panel - verse history
            div { class: "history-panel",
                h2 { class: "history-title", "Verse History" }
                div { class: "history-list",
                    for (ref_str, text, timestamp) in verse_history().iter() {
                        div { 
                            class: "history-item",
                            onclick: move |_| select_history_item((ref_str.clone(), text.clone(), *timestamp)),
                            div { class: "history-reference", "{ref_str}" }
                            div { class: "history-text", "{text}" }
                            div { class: "history-time", 
                                "Detected {timestamp} seconds ago" 
                            }
                        }
                    }
                }
            }
        }
    }
}
