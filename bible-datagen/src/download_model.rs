use std::fs;
use std::io;
use std::path::Path;

// ── download-model ───────────────────────────────────────────────────

pub fn run_download_model() -> Result<(), io::Error> {
    let url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17.tar.bz2";
    let model_dir_name = "sherpa-onnx-streaming-zipformer-en-20M-2023-02-17";
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/models");
    let model_dir = output_dir.join(model_dir_name);

    if model_dir.exists() && model_dir.join("encoder-epoch-99-avg-1.onnx").exists() {
        println!("✓ Model already downloaded in {:?}", model_dir);
        return Ok(());
    }

    fs::create_dir_all(&output_dir)?;
    println!("Downloading model from {} ...", url);

    let response = ureq::get(url).call().map_err(|e| {
        io::Error::other(format!("Download failed: {}", e))
    })?;

    let reader = response.into_reader();
    
    println!("Extracting archive...");
    let bz2 = bzip2::read::BzDecoder::new(reader);
    let mut archive = tar::Archive::new(bz2);
    
    archive.unpack(&output_dir)?;

    println!("✓ Model downloaded and extracted to {:?}", model_dir);
    Ok(())
}
