use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const STYLE_NAME_SCREEN: &str = "花字";
const STYLE_NAME_SPEAK: &str = "听轴";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CcSubtitleResult {
    text: String,
    changed_lines: usize,
    inserted_lines: usize,
    replacement_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedCcSubtitleFile {
    output_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CcReplacementRule {
    replacement: String,
    pattern: String,
}

struct CompiledReplacementRule {
    replacement: String,
    regex: Regex,
}

#[tauri::command]
pub fn organize_cc_subtitle_text(
    text: String,
    replacement_rules: Option<Vec<CcReplacementRule>>,
) -> Result<CcSubtitleResult, String> {
    let rules = compile_replacement_rules(replacement_rules.unwrap_or_default())?;
    Ok(organize_cc_subtitle(&text, &rules))
}

#[tauri::command]
pub fn read_cc_subtitle_file(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|err| format!("Failed to read subtitle file: {err}"))?;
    decode_text(&bytes)
}

#[tauri::command]
pub fn save_cc_subtitle_file(
    path: String,
    text: String,
    suffix: String,
    overwrite: bool,
) -> Result<SavedCcSubtitleFile, String> {
    let input_path = PathBuf::from(path);
    let output_path = output_path_with_suffix(&input_path, &suffix)?;
    if output_path.exists() && !overwrite {
        return Err(format!("OUTPUT_EXISTS:{}", output_path.to_string_lossy()));
    }
    fs::write(&output_path, text).map_err(|err| format!("Failed to write subtitle file: {err}"))?;
    Ok(SavedCcSubtitleFile {
        output_path: output_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn save_cc_subtitle_to_path(path: String, text: String) -> Result<SavedCcSubtitleFile, String> {
    let output_path = PathBuf::from(path);
    fs::write(&output_path, text).map_err(|err| format!("Failed to write subtitle file: {err}"))?;
    Ok(SavedCcSubtitleFile {
        output_path: output_path.to_string_lossy().to_string(),
    })
}

fn organize_cc_subtitle(text: &str, replacement_rules: &[CompiledReplacementRule]) -> CcSubtitleResult {
    let mut event_format = default_event_format();
    let mut in_events = false;
    let mut output = String::with_capacity(text.len());
    let mut changed_lines = 0;
    let mut inserted_lines = 0;
    let mut replacement_count = 0;

    for raw_line in text.split_inclusive('\n') {
        let (line, eol) = split_line_ending(raw_line);
        let trimmed = line.trim();
        if is_section_header(trimmed) {
            in_events = trimmed.eq_ignore_ascii_case("[Events]");
            output.push_str(line);
            output.push_str(eol);
            continue;
        }

        if in_events {
            if let Some(format_value) = trimmed.strip_prefix("Format:") {
                event_format = parse_ass_format(format_value);
            }

            if let Some((next_line, inserted, changed, replacements)) =
                process_event_line(line, &event_format, replacement_rules)
            {
                output.push_str(&next_line);
                output.push_str(eol);
                if let Some(inserted_line) = inserted {
                    output.push_str(&inserted_line);
                    output.push_str(eol);
                    inserted_lines += 1;
                }
                if changed {
                    changed_lines += 1;
                }
                replacement_count += replacements;
                continue;
            }
        }

        output.push_str(line);
        output.push_str(eol);
    }

    CcSubtitleResult {
        text: output,
        changed_lines,
        inserted_lines,
        replacement_count,
    }
}

fn process_event_line(
    line: &str,
    format: &[String],
    replacement_rules: &[CompiledReplacementRule],
) -> Option<(String, Option<String>, bool, usize)> {
    let (kind, value) = event_value(line)?;
    let mut fields = split_ass_fields(value.trim_start(), format.len().max(1));
    let style_index = field_index(format, "style")?;
    let text_index = field_index(format, "text")?;
    if style_index >= fields.len() || text_index >= fields.len() {
        return None;
    }

    let original_fields = fields.clone();
    let (text, mut replacements) = apply_replacement_rules(&fields[text_index], replacement_rules);

    let inserted = if let Some(end_index) = text.find(']') {
        let speaker = text[..=end_index].replace(['[', ']'], "");
        let dialogue = text[end_index + 1..].to_string();
        fields[style_index] = STYLE_NAME_SCREEN.to_string();
        fields[text_index] = speaker;
        if dialogue.is_empty() {
            None
        } else {
            let mut inserted_fields = original_fields.clone();
            let inserted_text = clean_dialogue_text(&dialogue);
            let (final_inserted_text, inserted_replacements) =
                apply_replacement_rules(&inserted_text, replacement_rules);
            replacements += inserted_replacements;
            inserted_fields[style_index] = STYLE_NAME_SPEAK.to_string();
            inserted_fields[text_index] = final_inserted_text;
            Some(format_event_line(kind, &inserted_fields))
        }
    } else {
        fields[style_index] = STYLE_NAME_SPEAK.to_string();
        fields[text_index] = clean_dialogue_text(&text);
        None
    };

    let changed = fields != original_fields || inserted.is_some();
    Some((
        format_event_line(kind, &fields),
        inserted,
        changed,
        replacements,
    ))
}

fn event_value(line: &str) -> Option<(&str, &str)> {
    for kind in ["Dialogue", "Comment"] {
        if let Some(value) = line
            .strip_prefix(kind)
            .and_then(|rest| rest.strip_prefix(':'))
        {
            return Some((kind, value));
        }
    }
    None
}

fn format_event_line(kind: &str, fields: &[String]) -> String {
    format!("{kind}: {}", fields.join(","))
}

fn clean_dialogue_text(text: &str) -> String {
    let cleaned = text
        .replace("...", "")
        .replace("\\N", " ")
        .trim_start()
        .to_string();
    whitespace_regex().replace_all(&cleaned, " ").to_string()
}

fn compile_replacement_rules(
    rules: Vec<CcReplacementRule>,
) -> Result<Vec<CompiledReplacementRule>, String> {
    rules
        .into_iter()
        .filter(|rule| !rule.replacement.trim().is_empty() && !rule.pattern.trim().is_empty())
        .map(|rule| {
            Regex::new(rule.pattern.trim())
                .map(|regex| CompiledReplacementRule {
                    replacement: rule.replacement.trim().to_string(),
                    regex,
                })
                .map_err(|err| format!("Invalid replacement regex `{}`: {err}", rule.pattern))
        })
        .collect()
}

fn apply_replacement_rules(text: &str, rules: &[CompiledReplacementRule]) -> (String, usize) {
    let mut next = text.to_string();
    let mut count = 0;
    for rule in rules {
        let mut local_count = 0;
        next = rule
            .regex
            .replace_all(&next, |captures: &Captures<'_>| {
                local_count += 1;
                apply_capture_placeholders(&rule.replacement, captures)
            })
            .to_string();
        count += local_count;
    }
    (next, count)
}

fn apply_capture_placeholders(replacement: &str, captures: &Captures<'_>) -> String {
    let mut next = replacement.to_string();
    for index in 1..captures.len() {
        let Some(value) = captures.get(index) else {
            continue;
        };
        next = next.replace(&format!("%{index}"), value.as_str());
    }
    next
}

fn split_ass_fields(value: &str, expected_fields: usize) -> Vec<String> {
    if expected_fields <= 1 {
        return vec![value.to_string()];
    }
    value
        .splitn(expected_fields, ',')
        .map(ToString::to_string)
        .collect()
}

fn parse_ass_format(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| normalize_ass_field(part.trim()))
        .collect()
}

fn default_event_format() -> Vec<String> {
    [
        "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
        "Text",
    ]
    .into_iter()
    .map(normalize_ass_field)
    .collect()
}

fn field_index(format: &[String], field: &str) -> Option<usize> {
    let normalized = normalize_ass_field(field);
    format.iter().position(|value| value == &normalized)
}

fn normalize_ass_field(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '_'], "")
}

fn is_section_header(value: &str) -> bool {
    value.starts_with('[') && value.ends_with(']')
}

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(stripped) = line.strip_suffix("\r\n") {
        (stripped, "\r\n")
    } else if let Some(stripped) = line.strip_suffix('\n') {
        (stripped, "\n")
    } else if let Some(stripped) = line.strip_suffix('\r') {
        (stripped, "\r")
    } else {
        (line, "")
    }
}

fn whitespace_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s+").expect("whitespace regex should be valid"))
}

fn decode_text(bytes: &[u8]) -> Result<String, String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(bytes[3..].to_vec())
            .map_err(|err| format!("Text is not valid UTF-8: {err}"));
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units)
            .map_err(|err| format!("Text is not valid UTF-16 LE: {err}"));
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units)
            .map_err(|err| format!("Text is not valid UTF-16 BE: {err}"));
    }

    String::from_utf8(bytes.to_vec()).map_err(|err| format!("Text is not valid UTF-8: {err}"))
}

fn output_path_with_suffix(input_path: &Path, suffix: &str) -> Result<PathBuf, String> {
    let parent = input_path
        .parent()
        .ok_or_else(|| "Cannot identify input file directory".to_string())?;
    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Cannot identify input file name".to_string())?;
    let extension = input_path.extension().and_then(|value| value.to_str());
    let file_name = match extension {
        Some(ext) if !ext.is_empty() => format!("{stem}{suffix}.{ext}"),
        _ => format!("{stem}{suffix}"),
    };
    Ok(parent.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::{organize_cc_subtitle, CcReplacementRule};

    fn rules() -> Vec<super::CompiledReplacementRule> {
        super::compile_replacement_rules(vec![CcReplacementRule {
            replacement: "章昊".to_string(),
            pattern: r"(?i)ZHANG\s*HAO".to_string(),
        }])
        .unwrap()
    }

    #[test]
    fn splits_cc_tag_into_screen_and_speak_lines() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:01.00,Default,[ZHANG HAO]Hello...\\Nworld\n";
        let output = organize_cc_subtitle(input, &rules());
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,花字,章昊"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,听轴,Hello world"));
        assert_eq!(output.inserted_lines, 1);
        assert_eq!(output.replacement_count, 1);
    }

    #[test]
    fn converts_plain_dialogue_to_speak_style() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:01.00,Default,Hi...\\N there\n";
        let output = organize_cc_subtitle(input, &[]);
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,听轴,Hi there"));
        assert_eq!(output.inserted_lines, 0);
    }

    #[test]
    fn applies_capture_placeholder_in_replacement_rule() {
        let compiled = super::compile_replacement_rules(vec![CcReplacementRule {
            replacement: "%1ense".to_string(),
            pattern: "([sS])ence".to_string(),
        }])
        .unwrap();
        let input = "[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:01.00,Default,Sence\n";
        let output = organize_cc_subtitle(input, &compiled);
        assert!(output.text.contains("听轴,Sense"));
        assert_eq!(output.replacement_count, 1);
    }
}
