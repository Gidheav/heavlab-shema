use eframe::egui::{self, ComboBox, Ui};

use crate::app::HvBibleApp;
use crate::panels::audio::labeled_slider;

pub fn show(ui: &mut Ui, app: &mut HvBibleApp) {
    labeled_slider(
        ui,
        "Input Gain",
        &mut app.gain,
        -20.0,
        20.0,
        "dB",
    );
    labeled_slider(
        ui,
        "Digital Trim",
        &mut app.audio_mock.gain.digital_trim_db,
        -12.0,
        12.0,
        "dB",
    );



    // Send gain to pipeline
    app.pipeline
        .send_command(hv_pipeline::PipelineCommand::SetGain(app.gain));
}