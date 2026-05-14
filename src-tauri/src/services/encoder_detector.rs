use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderInfo {
    pub name: String,
    pub label: String,
    pub supported: bool,
}

pub fn get_supported_encoders() -> Vec<EncoderInfo> {
    let platform_encoders = get_platform_encoders();
    vec![
        EncoderInfo {
            name: "libx264".to_string(),
            label: "H.264 (libx264)".to_string(),
            supported: platform_encoders.contains(&"libx264"),
        },
        EncoderInfo {
            name: "h264_nvenc".to_string(),
            label: "H.264 (NVIDIA NVENC)".to_string(),
            supported: platform_encoders.contains(&"h264_nvenc"),
        },
        EncoderInfo {
            name: "h264_amf".to_string(),
            label: "H.264 (AMD AMF)".to_string(),
            supported: platform_encoders.contains(&"h264_amf"),
        },
        EncoderInfo {
            name: "h264_videotoolbox".to_string(),
            label: "H.264 (Apple VideoToolbox)".to_string(),
            supported: platform_encoders.contains(&"h264_videotoolbox"),
        },
    ]
}

fn get_platform_encoders() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    {
        vec!["libx264", "h264_nvenc", "h264_amf"]
    }
    #[cfg(target_os = "macos")]
    {
        vec!["libx264", "h264_videotoolbox"]
    }
    #[cfg(target_os = "linux")]
    {
        vec!["libx264", "h264_nvenc"]
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        vec!["libx264"]
    }
}

pub fn is_encoder_supported(encoder: &str) -> bool {
    get_platform_encoders().contains(&encoder)
}
