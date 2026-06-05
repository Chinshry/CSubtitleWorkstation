use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use zhconv::{zhconv, Variant};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomConversionRule {
    from: String,
    to: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedTextConversion {
    output_path: String,
}

#[tauri::command]
pub fn convert_chinese_text(
    text: String,
    mode: String,
    custom_rules: Option<Vec<CustomConversionRule>>,
) -> Result<String, String> {
    let variant = match mode.as_str() {
        "t2s" => Variant::ZhCN,
        "s2t" => Variant::ZhTW,
        _ => return Err(format!("Unknown conversion mode: {mode}")),
    };

    let rules = normalize_custom_rules(custom_rules.unwrap_or_default());
    let (protected_text, placeholders) = protect_custom_rule_sources(&text, &rules);
    let converted = zhconv(&protected_text, variant);
    Ok(restore_custom_rule_targets(converted, placeholders))
}

#[tauri::command]
pub fn read_plain_text_file(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|err| format!("Failed to read text file: {err}"))?;
    decode_text(&bytes)
}

#[tauri::command]
pub fn save_converted_text_file(
    path: String,
    text: String,
    suffix: String,
    overwrite: bool,
) -> Result<SavedTextConversion, String> {
    let input_path = PathBuf::from(path);
    let output_path = output_path_with_suffix(&input_path, &suffix)?;
    if output_path.exists() && !overwrite {
        return Err(format!(
            "OUTPUT_EXISTS:{}",
            output_path.to_string_lossy()
        ));
    }
    fs::write(&output_path, text)
        .map_err(|err| format!("Failed to write converted text: {err}"))?;
    Ok(SavedTextConversion {
        output_path: output_path.to_string_lossy().to_string(),
    })
}

fn normalize_custom_rules(mut rules: Vec<CustomConversionRule>) -> Vec<CustomConversionRule> {
    rules.retain(|rule| !rule.from.is_empty());
    rules.sort_by(|a, b| b.from.chars().count().cmp(&a.from.chars().count()));
    rules
}

fn protect_custom_rule_sources(
    text: &str,
    rules: &[CustomConversionRule],
) -> (String, Vec<(String, String)>) {
    let mut protected = text.to_string();
    let placeholders = rules
        .iter()
        .enumerate()
        .map(|(index, rule)| {
            let placeholder = format!("\u{E000}CSW{index}\u{E001}");
            protected = protected.replace(&rule.from, &placeholder);
            (placeholder, rule.to.clone())
        })
        .collect();
    (protected, placeholders)
}

fn restore_custom_rule_targets(text: String, placeholders: Vec<(String, String)>) -> String {
    placeholders
        .into_iter()
        .fold(text, |next, (placeholder, target)| next.replace(&placeholder, &target))
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
    use super::{convert_chinese_text, CustomConversionRule};

    #[test]
    fn converts_taiwan_phrases_to_mainland_simplified() {
        let input = "俐落\n急遽\n哈囉\n彷彿\n反覆\n回覆\n軟體\n記憶體\n腳踏車";
        let output = convert_chinese_text(input.to_string(), "t2s".to_string(), None).unwrap();
        assert_eq!(output, "俐落\n急遽\n哈啰\n仿佛\n反复\n回复\n软件\n内存\n自行车");
    }

    #[test]
    fn applies_custom_rules_after_base_conversion() {
        let input = "俐落\n急遽";
        let output = convert_chinese_text(
            input.to_string(),
            "t2s".to_string(),
            Some(vec![
                CustomConversionRule {
                    from: "俐落".to_string(),
                    to: "利落".to_string(),
                },
                CustomConversionRule {
                    from: "急遽".to_string(),
                    to: "急剧".to_string(),
                },
            ]),
        )
        .unwrap();
        assert_eq!(output, "利落\n急剧");
    }

    #[test]
    fn custom_rules_have_priority_over_base_conversion() {
        let input = "朴乾旭";
        let output = convert_chinese_text(
            input.to_string(),
            "t2s".to_string(),
            Some(vec![CustomConversionRule {
                from: "朴乾旭".to_string(),
                to: "朴乾旭".to_string(),
            }]),
        )
        .unwrap();
        assert_eq!(output, "朴乾旭");
    }
}
