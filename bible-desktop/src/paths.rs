use std::path::PathBuf;

/// Get the data directory for the application.
///
/// In debug builds, this resolves to CARGO_MANIFEST_DIR/../data
/// In release builds, this resolves to the executable's directory + /data
pub fn data_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        // Debug build: use workspace relative path
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data")
    } else {
        // Release build: use executable directory
        let exe_path = std::env::current_exe()
            .expect("Failed to get executable path");
        let exe_dir = exe_path.parent()
            .expect("Failed to get executable directory");
        exe_dir.join("data")
    }
}

/// Get the packs directory containing Bible translation data files.
pub fn packs_dir() -> PathBuf {
    data_dir().join("packs")
}

/// Get the models directory containing ASR model files.
pub fn models_dir() -> PathBuf {
    data_dir().join("models")
}

/// Get the path to a specific model file or directory.
pub fn model_path(model_name: &str) -> PathBuf {
    models_dir().join(model_name)
}