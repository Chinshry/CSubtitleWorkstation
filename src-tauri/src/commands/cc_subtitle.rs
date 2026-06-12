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

struct CcStyleNames {
    screen: String,
    speak: String,
}

#[tauri::command]
pub async fn organize_cc_subtitle_text(
    text: String,
    replacement_rules: Option<Vec<CcReplacementRule>>,
    screen_style_name: Option<String>,
    speak_style_name: Option<String>,
    ass_header: Option<String>,
) -> Result<CcSubtitleResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let rules = compile_replacement_rules(replacement_rules.unwrap_or_default())?;
        let style_names = CcStyleNames::new(screen_style_name, speak_style_name);
        Ok(organize_cc_subtitle(&text, &rules, &style_names, ass_header.as_deref()))
    })
    .await
    .map_err(|err| format!("Failed to run CC subtitle organizer: {err}"))?
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

impl CcStyleNames {
    fn new(screen: Option<String>, speak: Option<String>) -> Self {
        Self {
            screen: sanitize_style_name(screen.as_deref(), STYLE_NAME_SCREEN),
            speak: sanitize_style_name(speak.as_deref(), STYLE_NAME_SPEAK),
        }
    }
}

fn organize_cc_subtitle(
    text: &str,
    replacement_rules: &[CompiledReplacementRule],
    style_names: &CcStyleNames,
    ass_header: Option<&str>,
) -> CcSubtitleResult {
    if looks_like_srt(text) {
        return organize_srt_as_ass(text, replacement_rules, style_names, ass_header);
    }

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
                process_event_line(line, &event_format, replacement_rules, style_names)
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
    style_names: &CcStyleNames,
) -> Option<(String, Option<String>, bool, usize)> {
    let (kind, value) = event_value(line)?;
    let mut fields = split_ass_fields(value.trim_start(), format.len().max(1));
    let style_index = field_index(format, "style")?;
    let text_index = field_index(format, "text")?;
    if style_index >= fields.len() || text_index >= fields.len() {
        return None;
    }

    process_event_fields(kind, &mut fields, style_index, text_index, replacement_rules, style_names)
}

fn process_event_fields(
    kind: &str,
    fields: &mut [String],
    style_index: usize,
    text_index: usize,
    replacement_rules: &[CompiledReplacementRule],
    style_names: &CcStyleNames,
) -> Option<(String, Option<String>, bool, usize)> {
    let original_fields = fields.to_vec();
    let (text, mut replacements) = apply_replacement_rules(&fields[text_index], replacement_rules);

    let inserted = if let Some(end_index) = text.find(']') {
        let speaker = text[..=end_index].replace(['[', ']'], "");
        let dialogue = text[end_index + 1..].to_string();
        fields[style_index] = style_names.screen.clone();
        fields[text_index] = speaker;
        if dialogue.is_empty() {
            None
        } else {
            let mut inserted_fields = original_fields.clone();
            let inserted_text = clean_dialogue_text(&dialogue);
            let (final_inserted_text, inserted_replacements) =
                apply_replacement_rules(&inserted_text, replacement_rules);
            replacements += inserted_replacements;
            inserted_fields[style_index] = style_names.speak.clone();
            inserted_fields[text_index] = final_inserted_text;
            Some(format_event_line(kind, &inserted_fields))
        }
    } else {
        fields[style_index] = style_names.speak.clone();
        fields[text_index] = clean_dialogue_text(&text);
        None
    };

    let changed = fields != original_fields || inserted.is_some();
    Some((
        format_event_line(kind, fields),
        inserted,
        changed,
        replacements,
    ))
}

fn organize_srt_as_ass(
    text: &str,
    replacement_rules: &[CompiledReplacementRule],
    style_names: &CcStyleNames,
    ass_header: Option<&str>,
) -> CcSubtitleResult {
    let mut output = ass_document_header(style_names, ass_header);
    let mut changed_lines = 0;
    let mut inserted_lines = 0;
    let mut replacement_count = 0;

    for cue in parse_srt_cues(text) {
        let mut fields = vec![
            "0".to_string(),
            cue.start,
            cue.end,
            "Default".to_string(),
            String::new(),
            "0".to_string(),
            "0".to_string(),
            "0".to_string(),
            String::new(),
            cue.text,
        ];

        if let Some((line, inserted, changed, replacements)) =
            process_event_fields("Dialogue", &mut fields, 3, 9, replacement_rules, style_names)
        {
            output.push_str(&line);
            output.push('\n');
            if let Some(inserted_line) = inserted {
                output.push_str(&inserted_line);
                output.push('\n');
                inserted_lines += 1;
            }
            if changed {
                changed_lines += 1;
            }
            replacement_count += replacements;
        }
    }

    CcSubtitleResult {
        text: output,
        changed_lines,
        inserted_lines,
        replacement_count,
    }
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

struct SrtCue {
    start: String,
    end: String,
    text: String,
}

fn looks_like_srt(text: &str) -> bool {
    text.lines()
        .take(12)
        .any(|line| parse_srt_timing(line.trim()).is_some())
}

fn parse_srt_cues(text: &str) -> Vec<SrtCue> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    normalized
        .split("\n\n")
        .filter_map(parse_srt_cue)
        .collect()
}

fn parse_srt_cue(block: &str) -> Option<SrtCue> {
    let mut lines = block.lines().map(str::trim_end).filter(|line| !line.trim().is_empty());
    let first = lines.next()?;
    let timing_line = if first.trim().chars().all(|value| value.is_ascii_digit()) {
        lines.next()?
    } else {
        first
    };
    let (start, end) = parse_srt_timing(timing_line.trim())?;
    let text = lines.collect::<Vec<_>>().join("\\N");
    if text.trim().is_empty() {
        return None;
    }

    Some(SrtCue {
        start,
        end,
        text,
    })
}

fn parse_srt_timing(line: &str) -> Option<(String, String)> {
    let (start, end) = line.split_once("-->")?;
    Some((srt_time_to_ass(start.trim())?, srt_time_to_ass(end.trim())?))
}

fn srt_time_to_ass(value: &str) -> Option<String> {
    let time = value.split_whitespace().next()?;
    let mut parts = time.split([':', ',']);
    let hours = parts.next()?.parse::<u32>().ok()?;
    let minutes = parts.next()?.parse::<u32>().ok()?;
    let seconds = parts.next()?.parse::<u32>().ok()?;
    let millis = parts.next()?.parse::<u32>().ok()?;
    let total_centis =
        (((hours * 60 + minutes) * 60 + seconds) * 1000 + millis + 5) / 10;
    let centis = total_centis % 100;
    let total_seconds = total_centis / 100;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;
    Some(format!(
        "{}:{:02}:{:02}.{:02}",
        hours,
        minutes,
        seconds,
        centis
    ))
}

fn sanitize_style_name(value: Option<&str>, fallback: &str) -> String {
    let style = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback);
    style.replace(',', " ")
}

fn ass_document_header(style_names: &CcStyleNames, ass_header: Option<&str>) -> String {
    if let Some(header) = ass_header.map(str::trim).filter(|value| !value.is_empty()) {
        let mut output = header.replace("\r\n", "\n").replace('\r', "\n");
        while output.ends_with('\n') {
            output.pop();
        }
        output.push_str("\n\n[Events]\n");
        output.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\n");
        return output;
    }

    default_ass_document(style_names)
}

fn default_ass_document(style_names: &CcStyleNames) -> String {
    let style_screen = format!(
        "Style: {},Microsoft YaHei,36,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,8,20,20,24,1",
        style_names.screen
    );
    let style_speak = format!(
        "Style: {},Microsoft YaHei,36,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,20,20,36,1",
        style_names.speak
    );

    let lines = [
        "[Script Info]",
        "ScriptType: v4.00+",
        "WrapStyle: 0",
        "ScaledBorderAndShadow: yes",
        "",
        "[V4+ Styles]",
        "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding",
        &style_screen,
        &style_speak,
        "",
        "[Events]",
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text",
        "",
    ];
    lines.join("\n")
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

    fn style_names() -> super::CcStyleNames {
        super::CcStyleNames::new(None, None)
    }

    #[test]
    fn splits_cc_tag_into_screen_and_speak_lines() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:01.00,Default,[ZHANG HAO]Hello...\\Nworld\n";
        let output = organize_cc_subtitle(input, &rules(), &style_names(), None);
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,花字,章昊"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,听轴,Hello... world"));
        assert_eq!(output.inserted_lines, 1);
        assert_eq!(output.replacement_count, 1);
    }

    #[test]
    fn converts_plain_dialogue_to_speak_style() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:01.00,Default,Hi...\\N there\n";
        let output = organize_cc_subtitle(input, &[], &style_names(), None);
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,听轴,Hi... there"));
        assert_eq!(output.inserted_lines, 0);
    }

    #[test]
    fn keeps_ascii_ellipsis_and_cleans_newline_spacing() {
        assert_eq!(super::clean_dialogue_text("Wait...\\Nnow"), "Wait... now");
    }

    #[test]
    fn applies_capture_placeholder_in_replacement_rule() {
        let compiled = super::compile_replacement_rules(vec![CcReplacementRule {
            replacement: "%1ense".to_string(),
            pattern: "([sS])ence".to_string(),
        }])
        .unwrap();
        let input = "[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:01.00,Default,Sence\n";
        let output = organize_cc_subtitle(input, &compiled, &style_names(), None);
        assert!(output.text.contains("听轴,Sense"));
        assert_eq!(output.replacement_count, 1);
    }

    #[test]
    fn converts_srt_to_ass_and_splits_speaker_tag() {
        let input = "1\n00:00:00,000 --> 00:00:01,277\n[오늘의 수업 미리보기]\n빠바바바바바바밤\n\n2\n00:00:01,278 --> 00:00:02,348\n하고 갑자기 없어졌다가\n";
        let output = organize_cc_subtitle(input, &[], &style_names(), None);

        assert!(output.text.contains("[Script Info]"));
        assert!(output.text.contains("[Events]"));
        assert!(output.text.contains("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.28,花字,,0,0,0,,오늘의 수업 미리보기"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.28,听轴,,0,0,0,,빠바바바바바바밤"));
        assert!(output.text.contains("Dialogue: 0,0:00:01.28,0:00:02.35,听轴,,0,0,0,,하고 갑자기 없어졌다가"));
        assert_eq!(output.inserted_lines, 1);
    }

    #[test]
    fn uses_custom_style_names_for_srt_output() {
        let styles = super::CcStyleNames::new(Some("上屏".to_string()), Some("正文".to_string()));
        let input = "1\n00:00:00,000 --> 00:00:01,000\n[Speaker]\nHello\n";
        let output = organize_cc_subtitle(input, &[], &styles, None);

        assert!(output.text.contains("Style: 上屏,Microsoft YaHei"));
        assert!(output.text.contains("Style: 正文,Microsoft YaHei"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,上屏,,0,0,0,,Speaker"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,正文,,0,0,0,,Hello"));
    }

    #[test]
    fn uses_imported_ass_header_for_srt_output() {
        let styles = super::CcStyleNames::new(Some("屏幕字_65".to_string()), Some("1080_横_听轴".to_string()));
        let header = "[Script Info]\nScriptType: v4.00+\nPlayResX: 1080\nPlayResY: 1920\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour\nStyle: 屏幕字_65,Arial,65,&H00FFFFFF\nStyle: 1080_横_听轴,Arial,70,&H00FFFFFF";
        let input = "1\n00:00:00,000 --> 00:00:01,000\n[Caption]\nLine\n";
        let output = organize_cc_subtitle(input, &[], &styles, Some(header));

        assert!(output.text.starts_with("[Script Info]\nScriptType: v4.00+\nPlayResX: 1080\nPlayResY: 1920"));
        assert!(output.text.contains("Style: 屏幕字_65,Arial,65,&H00FFFFFF"));
        assert!(output.text.contains("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,屏幕字_65,,0,0,0,,Caption"));
        assert!(output.text.contains("Dialogue: 0,0:00:00.00,0:00:01.00,1080_横_听轴,,0,0,0,,Line"));
        assert!(!output.text.contains("[Aegisub Project Garbage]"));
    }
}
