//! Filesystem locations for config, packs, and models.

use std::path::PathBuf;

/// Application data directory.
/// this is the codes thats handles the data directory of the application on release mode and debug mode, other
///
/// Debug builds use the workspace `data/` folder. Release builds use
/// `<exe_dir>/data`.
pub fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data")
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

#[allow(dead_code)]
pub fn packs_dir() -> PathBuf {
    data_dir().join("packs")
}

#[allow(dead_code)]
pub fn models_dir() -> PathBuf {
    data_dir().join("models")
}
