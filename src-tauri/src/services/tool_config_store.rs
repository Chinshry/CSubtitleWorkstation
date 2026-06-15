use crate::models::tool_config::{CcSubtitleConfig, ProofreadConfig, TextConversionConfig};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

const TEXT_CONVERSION_SECTION: &str = "textConversion";
const PROOFREAD_SECTION: &str = "proofread";
const CC_SUBTITLE_SECTION: &str = "ccSubtitle";

#[derive(Default)]
struct ToolboxConfig {
    text_conversion: TextConversionConfig,
    proofread: ProofreadConfig,
    cc_subtitle: CcSubtitleConfig,
}

fn tool_config_path(app: &AppHandle, file_name: &str) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("failed to get config directory: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| format!("failed to create config directory: {err}"))?;
    Ok(dir.join(file_name))
}

pub fn load_text_conversion(
    app: &AppHandle,
    file_name: &str,
) -> Result<TextConversionConfig, String> {
    Ok(load_toolbox(app, file_name)?.text_conversion)
}

pub fn save_text_conversion(
    app: &AppHandle,
    file_name: &str,
    config: &TextConversionConfig,
) -> Result<(), String> {
    let mut toolbox = load_toolbox(app, file_name)?;
    toolbox.text_conversion = config.clone();
    save_toolbox(app, file_name, &toolbox)
}

pub fn load_proofread(app: &AppHandle, file_name: &str) -> Result<ProofreadConfig, String> {
    Ok(load_toolbox(app, file_name)?.proofread)
}

pub fn save_proofread(
    app: &AppHandle,
    file_name: &str,
    config: &ProofreadConfig,
) -> Result<(), String> {
    let mut toolbox = load_toolbox(app, file_name)?;
    toolbox.proofread = config.clone();
    save_toolbox(app, file_name, &toolbox)
}

pub fn load_cc_subtitle(app: &AppHandle, file_name: &str) -> Result<CcSubtitleConfig, String> {
    Ok(load_toolbox(app, file_name)?.cc_subtitle)
}

pub fn save_cc_subtitle(
    app: &AppHandle,
    file_name: &str,
    config: &CcSubtitleConfig,
) -> Result<(), String> {
    let mut toolbox = load_toolbox(app, file_name)?;
    toolbox.cc_subtitle = config.clone();
    save_toolbox(app, file_name, &toolbox)
}

fn load_toolbox(app: &AppHandle, file_name: &str) -> Result<ToolboxConfig, String> {
    let path = tool_config_path(app, file_name)?;
    if !path.exists() {
        return Ok(ToolboxConfig::default());
    }

    let raw = fs::read_to_string(&path).map_err(|err| format!("failed to read toolbox config: {err}"))?;
    Ok(parse_toolbox(&raw))
}

fn save_toolbox(app: &AppHandle, file_name: &str, config: &ToolboxConfig) -> Result<(), String> {
    let path = tool_config_path(app, file_name)?;
    fs::write(path, format_toolbox(config))
        .map_err(|err| format!("failed to save toolbox config: {err}"))
}

fn parse_toolbox(raw: &str) -> ToolboxConfig {
    let text_conversion = parse_section(raw, TEXT_CONVERSION_SECTION)
        .map(|section| TextConversionConfig {
            custom_dictionary: parse_block(&section, "customDictionary"),
        })
        .unwrap_or_default();
    let proofread = parse_section(raw, PROOFREAD_SECTION)
        .map(|section| ProofreadConfig {
            term_dictionary: parse_block(&section, "termDictionary"),
        })
        .unwrap_or_default();
    let cc_subtitle = parse_section(raw, CC_SUBTITLE_SECTION)
        .map(|section| CcSubtitleConfig {
            replacement_dictionary: parse_block(&section, "replacementDictionary"),
            ass_header: parse_block(&section, "assHeader"),
            screen_style_name: parse_scalar(&section, "screenStyleName"),
            speak_style_name: parse_scalar(&section, "speakStyleName"),
        })
        .unwrap_or_default();

    ToolboxConfig {
        text_conversion,
        proofread,
        cc_subtitle,
    }
}

fn format_toolbox(config: &ToolboxConfig) -> String {
    format!(
        "{}{}{}",
        format_section(
            TEXT_CONVERSION_SECTION,
            &format_block("customDictionary", &config.text_conversion.custom_dictionary),
        ),
        format_section(
            PROOFREAD_SECTION,
            &format_block("termDictionary", &config.proofread.term_dictionary),
        ),
        format_section(
            CC_SUBTITLE_SECTION,
            &format!(
                "{}{}{}{}",
                format_block("replacementDictionary", &config.cc_subtitle.replacement_dictionary),
                format_block("assHeader", &config.cc_subtitle.ass_header),
                format_scalar("screenStyleName", &config.cc_subtitle.screen_style_name),
                format_scalar("speakStyleName", &config.cc_subtitle.speak_style_name),
            ),
        ),
    )
}

fn format_section(key: &str, body: &str) -> String {
    let mut out = format!("{key}:\n");
    for line in body.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn format_block(key: &str, value: &str) -> String {
    let mut out = format!("{key}: |\n");
    for line in value.replace("\r\n", "\n").replace('\r', "\n").split('\n') {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn format_scalar(key: &str, value: &str) -> String {
    format!("{key}: {}\n", quote_scalar(value))
}

fn quote_scalar(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn parse_section(raw: &str, key: &str) -> Option<String> {
    let start = find_key_line(raw, key)?;
    let lines: Vec<&str> = raw.lines().collect();
    let mut section = Vec::new();
    for line in lines.into_iter().skip(start + 1) {
        if is_top_level_key(line) {
            break;
        }
        if let Some(rest) = line.strip_prefix("  ") {
            section.push(rest);
        } else if line.trim().is_empty() {
            section.push("");
        }
    }
    Some(section.join("\n"))
}

fn parse_block(raw: &str, key: &str) -> String {
    let Some(start) = find_key_line(raw, key) else {
        return String::new();
    };
    let lines: Vec<&str> = raw.lines().collect();
    let value = lines[start]
        .split_once(':')
        .map(|(_, rest)| rest.trim())
        .unwrap_or_default();
    if !value.starts_with('|') {
        return unquote_scalar(value);
    }

    let mut block = Vec::new();
    for line in lines.into_iter().skip(start + 1) {
        if is_top_level_key(line) {
            break;
        }
        if let Some(rest) = line.strip_prefix("  ") {
            block.push(rest);
        } else if line.trim().is_empty() {
            block.push("");
        }
    }
    trim_trailing_empty_lines(&mut block);
    block.join("\n")
}

fn parse_scalar(raw: &str, key: &str) -> String {
    let Some(start) = find_key_line(raw, key) else {
        return String::new();
    };
    raw.lines()
        .nth(start)
        .and_then(|line| line.split_once(':').map(|(_, rest)| unquote_scalar(rest.trim())))
        .unwrap_or_default()
}

fn find_key_line(raw: &str, key: &str) -> Option<usize> {
    raw.lines().position(|line| {
        line.starts_with(key)
            && line
                .as_bytes()
                .get(key.len())
                .is_some_and(|ch| *ch == b':')
    })
}

fn is_top_level_key(line: &str) -> bool {
    if line.starts_with(' ') || line.trim().is_empty() {
        return false;
    }
    line.split_once(':')
        .map(|(key, _)| {
            !key.is_empty()
                && key
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        })
        .unwrap_or(false)
}

fn unquote_scalar(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        return trimmed[1..trimmed.len() - 1].replace("''", "'");
    }
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return serde_json::from_str(trimmed)
            .unwrap_or_else(|_| trimmed[1..trimmed.len() - 1].to_string());
    }
    trimmed.to_string()
}

fn trim_trailing_empty_lines(lines: &mut Vec<&str>) {
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
}
