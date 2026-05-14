use crate::services::encoder_detector;

#[tauri::command]
pub fn get_supported_encoders() -> Vec<encoder_detector::EncoderInfo> {
    encoder_detector::get_supported_encoders()
}
