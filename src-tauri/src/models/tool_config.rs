use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct TextConversionConfig {
    #[serde(default)]
    pub custom_dictionary: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ProofreadConfig {
    #[serde(default)]
    pub term_dictionary: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CcSubtitleConfig {
    #[serde(default)]
    pub replacement_dictionary: String,
    #[serde(default)]
    pub ass_header: String,
    #[serde(default)]
    pub screen_style_name: String,
    #[serde(default)]
    pub speak_style_name: String,
}
