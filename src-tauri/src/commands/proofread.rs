use jieba_rs::{Jieba, Tag};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofreadIssue {
    id: String,
    line: usize,
    start: usize,
    end: usize,
    original: String,
    suggestion: String,
    message: String,
    reason: String,
    context: String,
    confidence: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofreadTermRule {
    canonical: String,
    pattern: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedProofreadFile {
    output_path: String,
}

#[tauri::command]
pub fn proofread_text(
    text: String,
    term_rules: Option<Vec<ProofreadTermRule>>,
) -> Result<Vec<ProofreadIssue>, String> {
    let mut issues = check_de_di_de_with_jieba(&text);
    collect_term_rule_issues(&text, term_rules.unwrap_or_default(), &mut issues)?;
    issues.sort_by_key(|issue| (issue.start, issue.end));
    for (index, issue) in issues.iter_mut().enumerate() {
        issue.id = format!("proofread-{}", index + 1);
    }
    Ok(issues)
}

#[tauri::command]
pub fn read_proofread_file(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|err| format!("Failed to read text file: {err}"))?;
    decode_text(&bytes)
}

#[tauri::command]
pub fn save_proofread_file(
    path: String,
    text: String,
    suffix: String,
    overwrite: bool,
) -> Result<SavedProofreadFile, String> {
    let input_path = PathBuf::from(path);
    let output_path = output_path_with_suffix(&input_path, &suffix)?;
    if output_path.exists() && !overwrite {
        return Err(format!("OUTPUT_EXISTS:{}", output_path.to_string_lossy()));
    }
    fs::write(&output_path, text).map_err(|err| format!("Failed to write proofread text: {err}"))?;
    Ok(SavedProofreadFile {
        output_path: output_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn save_proofread_to_path(path: String, text: String) -> Result<SavedProofreadFile, String> {
    let output_path = PathBuf::from(path);
    fs::write(&output_path, text).map_err(|err| format!("Failed to write proofread text: {err}"))?;
    Ok(SavedProofreadFile {
        output_path: output_path.to_string_lossy().to_string(),
    })
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

fn collect_term_rule_issues(
    text: &str,
    rules: Vec<ProofreadTermRule>,
    issues: &mut Vec<ProofreadIssue>,
) -> Result<(), String> {
    let compiled_rules = rules
        .into_iter()
        .filter(|rule| !rule.canonical.trim().is_empty() && !rule.pattern.trim().is_empty())
        .map(|rule| {
            Regex::new(rule.pattern.trim())
                .map(|regex| (rule.canonical.trim().to_string(), regex))
                .map_err(|err| format!("?????????{} ({err})", rule.pattern))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if compiled_rules.is_empty() {
        return Ok(());
    }

    let mut line_start = 0;
    for (line_index, raw_line) in text.split_inclusive('\n').enumerate() {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        collect_line_term_issues(line, line_index + 1, line_start, &compiled_rules, issues);
        line_start += raw_line.chars().count();
    }

    Ok(())
}

fn collect_line_term_issues(
    line: &str,
    line_number: usize,
    line_start: usize,
    rules: &[(String, Regex)],
    issues: &mut Vec<ProofreadIssue>,
) {
    if line.trim().is_empty() {
        return;
    }

    for (canonical, regex) in rules {
        for captures in regex.captures_iter(line) {
            let Some(matched) = captures.get(0) else {
                continue;
            };
            let suggestion = apply_capture_placeholders(canonical, &captures);
            if matched.as_str() == suggestion {
                continue;
            }

            let start = line_start + line[..matched.start()].chars().count();
            let end = line_start + line[..matched.end()].chars().count();
            issues.push(ProofreadIssue {
                id: format!("term-{}", issues.len() + 1),
                line: line_number,
                start,
                end,
                original: matched.as_str().to_string(),
                suggestion,
                message: "?????????".to_string(),
                reason: "???????????????????".to_string(),
                context: line.trim().to_string(),
                confidence: "high".to_string(),
            });
        }
    }
}

fn apply_capture_placeholders(canonical: &str, captures: &regex::Captures<'_>) -> String {
    let mut suggestion = canonical.to_string();
    for index in 1..captures.len() {
        let Some(value) = captures.get(index) else {
            continue;
        };
        suggestion = suggestion.replace(&format!("%{index}"), value.as_str());
    }
    suggestion
}

fn check_de_di_de_with_jieba(text: &str) -> Vec<ProofreadIssue> {
    let mut issues = Vec::new();
    let mut line_start = 0;

    for (line_index, raw_line) in text.split_inclusive('\n').enumerate() {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        collect_line_issues(line, line_index + 1, line_start, &mut issues);
        line_start += raw_line.chars().count();
    }

    if !text.ends_with('\n') && text.is_empty() {
        collect_line_issues("", 1, 0, &mut issues);
    }

    issues
}

fn collect_line_issues(
    line: &str,
    line_number: usize,
    line_start: usize,
    issues: &mut Vec<ProofreadIssue>,
) {
    if line.trim().is_empty() {
        return;
    }

    let tags = jieba().tag(line, true);
    for (index, tag) in tags.iter().enumerate() {
        let Some((suggestion, message, reason, confidence)) = suggest_with_tags(&tags, index) else {
            continue;
        };
        let start = line_start + tag.start;
        let end = line_start + tag.end;
        issues.push(ProofreadIssue {
            id: format!("de-di-de-{}", issues.len() + 1),
            line: line_number,
            start,
            end,
            original: tag.word.to_string(),
            suggestion: suggestion.to_string(),
            message: message.to_string(),
            reason: reason.to_string(),
            context: line.trim().to_string(),
            confidence: confidence.to_string(),
        });
    }
}

fn suggest_with_tags<'a>(
    tags: &[Tag<'a>],
    index: usize,
) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
    let current = tags[index].word;
    if !matches!(current, "的" | "地" | "得") {
        return None;
    }

    let prev = previous_content_tag(tags, index);
    let next = next_content_tag(tags, index);
    let after_next = next.and_then(|tag| next_content_tag(tags, tag.0));

    match current {
        "的" => {
            if prev.is_some_and(|(_, tag)| is_verb_or_adjective(tag))
                && next.is_some_and(|(_, tag)| is_degree_or_adjective(tag))
            {
                return Some(("得", "疑似应使用“得”", "动词或形容词后接程度、结果补语时通常使用“得”。", "medium"));
            }
            if prev.is_some_and(|(_, tag)| is_adverbial_modifier(tag))
                && next.is_some_and(|(_, tag)| is_verb(tag))
            {
                return Some(("地", "疑似应使用“地”", "副词或状态词修饰动作时通常使用“地”。", "medium"));
            }
        }
        "地" => {
            if next.is_some_and(|(_, tag)| is_noun(tag)) {
                return Some(("的", "疑似应使用“的”", "修饰名词时通常使用“的”。", "medium"));
            }
            if prev.is_some_and(|(_, tag)| is_verb_or_adjective(tag))
                && next.is_some_and(|(_, tag)| is_degree_or_adjective(tag))
            {
                return Some(("得", "疑似应使用“得”", "后面接程度或结果补语时通常使用“得”。", "medium"));
            }
        }
        "得" => {
            if prev.is_some_and(|(_, tag)| is_adverbial_modifier(tag))
                && next.is_some_and(|(_, tag)| is_verb(tag))
            {
                return Some(("地", "疑似应使用“地”", "副词或状态词修饰动作时通常使用“地”。", "medium"));
            }
            if next.is_some_and(|(_, tag)| is_noun(tag))
                || after_next.is_some_and(|(_, tag)| is_noun(tag))
            {
                return Some(("的", "疑似应使用“的”", "修饰名词时通常使用“的”。", "low"));
            }
        }
        _ => {}
    }

    None
}

fn previous_content_tag<'a>(tags: &'a [Tag<'a>], index: usize) -> Option<(usize, &'a str)> {
    tags[..index]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, tag)| !is_punctuation(tag.tag))
        .map(|(idx, tag)| (idx, tag.tag))
}

fn next_content_tag<'a>(tags: &'a [Tag<'a>], index: usize) -> Option<(usize, &'a str)> {
    tags[index + 1..]
        .iter()
        .enumerate()
        .find(|(_, tag)| !is_punctuation(tag.tag))
        .map(|(offset, tag)| (index + 1 + offset, tag.tag))
}

fn jieba() -> &'static Jieba {
    static JIEBA: OnceLock<Jieba> = OnceLock::new();
    JIEBA.get_or_init(Jieba::new)
}

fn is_punctuation(tag: &str) -> bool {
    tag == "x"
}

fn is_verb(tag: &str) -> bool {
    tag.starts_with('v')
}

fn is_noun(tag: &str) -> bool {
    tag.starts_with('n') || matches!(tag, "r" | "s" | "t")
}

fn is_adjective(tag: &str) -> bool {
    tag.starts_with('a')
}

fn is_adverbial_modifier(tag: &str) -> bool {
    tag.starts_with('d') || is_adjective(tag)
}

fn is_degree_or_adjective(tag: &str) -> bool {
    tag.starts_with('d') || is_adjective(tag)
}

fn is_verb_or_adjective(tag: &str) -> bool {
    is_verb(tag) || is_adjective(tag)
}

#[cfg(test)]
mod tests {
    use super::{proofread_text as proofread_text_command, ProofreadIssue, ProofreadTermRule};

    fn proofread_text(text: String) -> Result<Vec<ProofreadIssue>, String> {
        proofread_text_command(text, None)
    }

    #[test]
    fn flags_de_before_degree_complement() {
        let issues = proofread_text("他跑的很快".to_string()).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestion, "得");
    }

    #[test]
    fn flags_de_before_action() {
        let issues = proofread_text("她慢慢的走过来".to_string()).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestion, "地");
    }

    #[test]
    fn flags_di_before_noun() {
        let issues = proofread_text("漂亮地衣服".to_string()).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestion, "的");
    }

    #[test]
    fn flags_regex_term_rule() {
        let issues = proofread_text_command(
            "TOP 5 and Top 5".to_string(),
            Some(vec![ProofreadTermRule {
                canonical: "TOP 5".to_string(),
                pattern: "(?i)top 5".to_string(),
            }]),
        )
        .unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].original, "Top 5");
        assert_eq!(issues[0].suggestion, "TOP 5");
    }

    #[test]
    fn applies_capture_placeholder_in_term_rule() {
        let issues = proofread_text_command(
            "Sence".to_string(),
            Some(vec![ProofreadTermRule {
                canonical: "%1ense".to_string(),
                pattern: "([sS])ence".to_string(),
            }]),
        )
        .unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestion, "Sense");
    }

}
