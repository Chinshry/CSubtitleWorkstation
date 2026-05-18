use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleResourceIssue {
    pub path: String,
    pub resolved_path: String,
    pub line: usize,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleFontIssue {
    pub font: String,
    pub source: String,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleStyleIssue {
    pub style: String,
    pub line: usize,
}

/// ASS [Events] 段 Dialogue/Comment 行命中的 Banner Effect 字段
///
/// 命中规则（必须走 AVS 压制）：
///   - 小写 `banner;...`（任意参数）
///   - 大写 `Banner;delay;lefttoright;fadeawaywidth` 且 fadeawaywidth != 0
/// 不命中（可普通压制）：
///   - 大写 `Banner;8`、`Banner;8;0`、`Banner;8;0;0`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleBannerHit {
    pub line: usize,
    pub raw: String,
}

pub struct SubtitleAnalysis {
    pub has_effects: bool,
    pub detected_tags: Vec<String>,
    /// ASS [Script Info] section YCbCr Matrix value, for example "TV.709" or "PC.601".
    pub ass_matrix: Option<String>,
    pub missing_img_paths: Vec<SubtitleResourceIssue>,
    pub missing_fonts: Vec<SubtitleFontIssue>,
    pub missing_styles: Vec<SubtitleStyleIssue>,
    pub banner_hits: Vec<SubtitleBannerHit>,
}

pub fn analyze_subtitle(subtitle_path: &str) -> Result<SubtitleAnalysis, String> {
    let path = Path::new(subtitle_path);
    if !path.exists() {
        return Ok(empty_analysis());
    }

    let content =
        fs::read_to_string(path).map_err(|err| format!("Failed to read subtitle file: {err}"))?;

    let (mut has_effects, mut detected_tags) = detect_effect_tags(&content);
    let ass_matrix = parse_ass_matrix(&content);

    let is_ass_like = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("ass") || value.eq_ignore_ascii_case("ssa"))
        .unwrap_or(false);

    let (missing_fonts, missing_styles, banner_hits) = if is_ass_like {
        analyze_ass_references(&content)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };
    let missing_img_paths = find_missing_img_paths(&content, path.parent());

    // Banner Effect 字段命中即视为需要 AVS：与 VSFilterMod 标签复用 has_effects / detected_tags
    // 链路，CompressForm 会自动开启 AVS 并展示「检测到特殊标签」提示。
    if !banner_hits.is_empty() {
        has_effects = true;
        let mut seen: HashSet<String> = detected_tags.iter().cloned().collect();
        for hit in &banner_hits {
            let label = format!("{}(滚动横幅)", hit.raw);
            if seen.insert(label.clone()) {
                detected_tags.push(label);
            }
        }
        detected_tags.sort();
    }

    Ok(SubtitleAnalysis {
        has_effects,
        detected_tags,
        ass_matrix,
        missing_img_paths,
        missing_fonts,
        missing_styles,
        banner_hits,
    })
}

fn empty_analysis() -> SubtitleAnalysis {
    SubtitleAnalysis {
        has_effects: false,
        detected_tags: vec![],
        ass_matrix: None,
        missing_img_paths: vec![],
        missing_fonts: vec![],
        missing_styles: vec![],
        banner_hits: vec![],
    }
}

fn detect_effect_tags(content: &str) -> (bool, Vec<String>) {
    let mut tag_set = HashSet::new();

    // Only VSFilterMod-specific extension tags are treated as effects.
    let effect_patterns = [
        (r"\\fsc", "字体缩放"),
        (r"\\xblur", "X轴模糊"),
        (r"\\yblur", "Y轴模糊"),
        (r"\\fsvp", "纵向偏移"),
        (r"\\fshp", "横向偏移"),
        (r"\\[1-4]?vc", "四角颜色渐变"),
        (r"\\[1-4]?va", "四角透明度渐变"),
        (r"\\jitter", "随机抖动"),
        (r"\\rnd", "边界随机扭曲"),
        (r"\\distort", "四角扭曲变形"),
        (r"\\frs", "基线倾斜"),
        (r"\\z", "Z坐标"),
        (r"\\ortho", "正交投影"),
        (r"\\mover", "圆形/椭圆移动"),
        (r"\\moves[34]", "贝塞尔曲线移动"),
        (r"\\movevc", "向量遮罩移动"),
        (r"\\[1-4]?img", "图片填充"),
    ];

    for (pattern, label) in &effect_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.find_iter(content) {
                tag_set.insert(format!("{}({})", cap.as_str(), label));
            }
        }
    }

    let mut detected_tags: Vec<String> = tag_set.into_iter().collect();
    detected_tags.sort();
    (!detected_tags.is_empty(), detected_tags)
}

fn find_missing_img_paths(
    content: &str,
    subtitle_dir: Option<&Path>,
) -> Vec<SubtitleResourceIssue> {
    let Ok(re) = Regex::new(r#"\\(?P<tag>[1-4]?img)\s*\(\s*(?P<path>[^)\r\n]+?)\s*\)"#) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut issues = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        for caps in re.captures_iter(line) {
            let raw_path = caps
                .name("path")
                .map(|m| m.as_str())
                .unwrap_or_default()
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if raw_path.is_empty() {
                continue;
            }
            let resolved = resolve_subtitle_resource_path(&raw_path, subtitle_dir);
            if resolved.exists() {
                continue;
            }
            let tag = caps
                .name("tag")
                .map(|m| format!("\\{}", m.as_str()))
                .unwrap_or_else(|| "\\img".to_string());
            let key = format!("{}:{}:{}", line_idx + 1, tag, resolved.to_string_lossy());
            if seen.insert(key) {
                issues.push(SubtitleResourceIssue {
                    path: raw_path,
                    resolved_path: resolved.to_string_lossy().to_string(),
                    line: line_idx + 1,
                    tag,
                });
            }
        }
    }

    issues
}

fn resolve_subtitle_resource_path(raw_path: &str, subtitle_dir: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(raw_path);
    if path.is_absolute() {
        return path;
    }
    subtitle_dir
        .map(|dir| dir.join(path))
        .unwrap_or_else(|| PathBuf::from(raw_path))
}

fn analyze_ass_references(
    content: &str,
) -> (
    Vec<SubtitleFontIssue>,
    Vec<SubtitleStyleIssue>,
    Vec<SubtitleBannerHit>,
) {
    let parsed = parse_ass_sections(content);
    let installed_fonts = installed_font_names();

    let mut missing_fonts = Vec::new();
    let mut font_seen = HashSet::new();
    for font_ref in parsed.fonts {
        let font_key = normalize_font_name(&font_ref.font);
        if font_key.is_empty() || font_seen.contains(&font_key) {
            continue;
        }
        font_seen.insert(font_key.clone());
        if !installed_fonts.contains(&font_key) {
            missing_fonts.push(font_ref);
        }
    }

    let mut missing_styles = Vec::new();
    let mut style_seen = HashSet::new();
    for style_ref in parsed.used_styles {
        let style_key = normalize_ass_name(&style_ref.style);
        if style_key.is_empty() || parsed.defined_styles.contains(&style_key) {
            continue;
        }
        if style_seen.insert(style_key) {
            missing_styles.push(style_ref);
        }
    }

    (missing_fonts, missing_styles, parsed.banner_hits)
}

struct ParsedAssReferences {
    defined_styles: HashSet<String>,
    used_styles: Vec<SubtitleStyleIssue>,
    fonts: Vec<SubtitleFontIssue>,
    banner_hits: Vec<SubtitleBannerHit>,
}

fn parse_ass_sections(content: &str) -> ParsedAssReferences {
    let mut section = String::new();
    let mut style_format: Vec<String> = Vec::new();
    let mut event_format: Vec<String> = Vec::new();
    let mut defined_styles = HashSet::new();
    let mut used_styles = Vec::new();
    let mut fonts = Vec::new();
    let mut banner_hits = Vec::new();

    for (line_idx, raw) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line.to_ascii_lowercase();
            continue;
        }

        if section == "[v4+ styles]" || section == "[v4 styles]" {
            if let Some(value) = line.strip_prefix("Format:") {
                style_format = parse_ass_format(value);
                continue;
            }
            if let Some(value) = line.strip_prefix("Style:") {
                let fields = split_ass_fields(value, style_format.len().max(1));
                if let Some(style) = ass_field(&style_format, &fields, "name") {
                    let style = style.trim();
                    if !style.is_empty() {
                        defined_styles.insert(normalize_ass_name(style));
                    }
                }
                if let Some(font) = ass_field(&style_format, &fields, "fontname") {
                    let font = font.trim();
                    if !font.is_empty() {
                        fonts.push(SubtitleFontIssue {
                            font: font.to_string(),
                            source: "Style".to_string(),
                            line: Some(line_no),
                        });
                    }
                }
            }
        } else if section == "[events]" {
            if let Some(value) = line.strip_prefix("Format:") {
                event_format = parse_ass_format(value);
                continue;
            }
            let Some((kind, value)) = line.split_once(':') else {
                continue;
            };
            if !kind.eq_ignore_ascii_case("Dialogue") && !kind.eq_ignore_ascii_case("Comment") {
                continue;
            }
            let fields = split_ass_fields(value, event_format.len().max(1));
            if let Some(style) = ass_field(&event_format, &fields, "style") {
                let style = style.trim();
                if !style.is_empty() {
                    used_styles.push(SubtitleStyleIssue {
                        style: style.to_string(),
                        line: line_no,
                    });
                }
            }
            if let Some(effect) = ass_field(&event_format, &fields, "effect") {
                let effect = effect.trim();
                if banner_effect_requires_avs(effect) {
                    banner_hits.push(SubtitleBannerHit {
                        line: line_no,
                        raw: effect.to_string(),
                    });
                }
            }
            if let Some(text) = ass_field(&event_format, &fields, "text") {
                fonts.extend(parse_override_fonts(text, line_no));
            }
        }
    }

    ParsedAssReferences {
        defined_styles,
        used_styles,
        fonts,
        banner_hits,
    }
}

/// 判定 ASS Effect 字段是否为需要 AVS 的 Banner（横幅滚动）特效
///
/// ASS 规范：`Banner;delay[;lefttoright;fadeawaywidth]`
///
/// 命中规则（→ 需要 AVS）：
///   - 严格小写 `banner;...`（无论参数）—— 非标准写法，统一按需 AVS 处理
///   - 首字母大写 `Banner;...` 且 `fadeawaywidth` 显式给出且不为 0
///
/// 不命中（→ 可普通压制）：
///   - 首字母大写 `Banner` 缺省 `fadeawaywidth`（`Banner;8`、`Banner;8;0`）
///   - 首字母大写 `Banner` 且 `fadeawaywidth = 0`（`Banner;8;0;0`）
fn banner_effect_requires_avs(effect: &str) -> bool {
    if effect.is_empty() {
        return false;
    }
    let mut parts = effect.split(';');
    let name = match parts.next() {
        Some(name) => name.trim(),
        None => return false,
    };
    let is_lowercase = name == "banner";
    let is_proper_case = name == "Banner";
    if !is_lowercase && !is_proper_case {
        return false;
    }
    if is_lowercase {
        return true;
    }
    // Banner（首字母大写）：跳过 delay、lefttoright，取第 3 个分号后的 fadeawaywidth
    let _delay = parts.next();
    let _lefttoright = parts.next();
    let fadeaway = match parts.next() {
        Some(value) => value.trim(),
        None => return false,
    };
    if fadeaway.is_empty() {
        return false;
    }
    match fadeaway.parse::<i64>() {
        Ok(0) => false,
        Ok(_) => true,
        // 数字解析失败：保守按缺省处理，不强制 AVS
        Err(_) => false,
    }
}

fn parse_ass_format(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| normalize_ass_name(part.trim()))
        .collect()
}

fn split_ass_fields(value: &str, expected_fields: usize) -> Vec<String> {
    if expected_fields <= 1 {
        return vec![value.trim().to_string()];
    }
    value
        .splitn(expected_fields, ',')
        .map(|part| part.trim().to_string())
        .collect()
}

fn ass_field<'a>(format: &[String], fields: &'a [String], name: &str) -> Option<&'a str> {
    let needle = normalize_ass_name(name);
    format
        .iter()
        .position(|field| field == &needle)
        .and_then(|idx| fields.get(idx))
        .map(String::as_str)
}

fn parse_override_fonts(text: &str, line: usize) -> Vec<SubtitleFontIssue> {
    let Ok(re) = Regex::new(r"\\fn([^\\}]+)") else {
        return Vec::new();
    };
    re.captures_iter(text)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim()))
        .filter(|font| !font.is_empty())
        .map(|font| SubtitleFontIssue {
            font: font.to_string(),
            source: "Override".to_string(),
            line: Some(line),
        })
        .collect()
}

fn normalize_ass_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_font_name(value: &str) -> String {
    value
        .trim()
        .trim_matches('@')
        .to_ascii_lowercase()
        .replace([' ', '-', '_'], "")
}

fn installed_font_names() -> &'static HashSet<String> {
    static INSTALLED_FONTS: OnceLock<HashSet<String>> = OnceLock::new();
    INSTALLED_FONTS.get_or_init(|| {
        let mut names = HashSet::new();
        add_platform_font_names(&mut names);
        names
    })
}

#[cfg(windows)]
fn add_platform_font_names(names: &mut HashSet<String>) {
    for root in [
        "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts",
        "HKCU\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts",
    ] {
        add_windows_registry_fonts(names, root);
    }
    let mut dirs: Vec<PathBuf> = vec![PathBuf::from("C:\\Windows\\Fonts")];
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        // Win10/11 用户级安装位置（无需管理员权限）
        dirs.push(PathBuf::from(local).join("Microsoft\\Windows\\Fonts"));
    }
    let refs: Vec<&Path> = dirs.iter().map(PathBuf::as_path).collect();
    add_font_family_names(names, &refs);
}

#[cfg(windows)]
fn add_windows_registry_fonts(names: &mut HashSet<String>, key: &str) {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = Command::new("reg")
        .args(["query", key])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let Ok(output) = output else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let Some(value_name) = windows_registry_value_name(line) else {
            continue;
        };
        for name in windows_font_registry_names(value_name) {
            names.insert(normalize_font_name(&name));
        }
    }
}

#[cfg(windows)]
fn windows_registry_value_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("HKEY_") || trimmed.starts_with("HK") {
        return None;
    }
    let idx = trimmed
        .find("    REG_")
        .or_else(|| trimmed.find("\tREG_"))?;
    let value_name = trimmed[..idx].trim();
    if value_name.is_empty() {
        return None;
    }
    Some(value_name)
}

#[cfg(not(windows))]
fn add_platform_font_names(names: &mut HashSet<String>) {
    let mut dirs = vec![
        PathBuf::from("/System/Library/Fonts"),
        PathBuf::from("/Library/Fonts"),
        PathBuf::from("/usr/share/fonts"),
        PathBuf::from("/usr/local/share/fonts"),
    ];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(&home).join("Library/Fonts"));
        dirs.push(PathBuf::from(home).join(".fonts"));
    }
    let refs: Vec<&Path> = dirs.iter().map(PathBuf::as_path).collect();
    add_font_family_names(names, &refs);
}

#[cfg(windows)]
fn windows_font_registry_names(name: &str) -> Vec<String> {
    let mut cleaned = name.to_string();
    for marker in ["(TrueType)", "(OpenType)", "(Type 1)"] {
        cleaned = cleaned.replace(marker, " ");
    }

    cleaned
        .split('&')
        .map(clean_windows_font_name_part)
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(windows)]
fn clean_windows_font_name_part(name: &str) -> String {
    let mut parts: Vec<&str> = name.split_whitespace().collect();
    loop {
        let lower_tail = parts
            .last()
            .map(|part| part.to_ascii_lowercase())
            .unwrap_or_default();
        if matches!(
            lower_tail.as_str(),
            "regular"
                | "bold"
                | "italic"
                | "oblique"
                | "light"
                | "medium"
                | "semibold"
                | "demibold"
                | "black"
        ) {
            parts.pop();
            continue;
        }
        break;
    }
    parts.join(" ")
}

fn add_font_family_names(names: &mut HashSet<String>, dirs: &[&Path]) {
    for dir in dirs {
        collect_font_family_names(names, dir, 0);
    }
}

fn collect_font_family_names(names: &mut HashSet<String>, dir: &Path, depth: usize) {
    if depth > 4 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_font_family_names(names, &path, depth + 1);
            continue;
        }
        let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if !matches!(
            ext.to_ascii_lowercase().as_str(),
            "ttf" | "ttc" | "otf" | "otc"
        ) {
            continue;
        }
        let parsed_any = parse_font_file_names(&path, names);
        if !parsed_any {
            // 解析失败时回退到文件 stem，避免完全漏检
            if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
                names.insert(normalize_font_name(stem));
            }
        }
    }
}

/// 解析字体文件 name 表，把所有 Unicode 编码的 family / full / typographic 名（包括中文别名）
/// 归一化后插入集合。返回 true 表示至少成功解析了一个 face。
fn parse_font_file_names(path: &Path, names: &mut HashSet<String>) -> bool {
    const NAME_ID_FAMILY: u16 = 1;
    const NAME_ID_FULL_NAME: u16 = 4;
    const NAME_ID_TYPOGRAPHIC_FAMILY: u16 = 16;

    let Ok(data) = fs::read(path) else {
        return false;
    };
    let face_count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
    let mut any_parsed = false;
    for index in 0..face_count {
        let Ok(face) = ttf_parser::Face::parse(&data, index) else {
            continue;
        };
        any_parsed = true;
        for name in face.names() {
            if !matches!(
                name.name_id,
                NAME_ID_FAMILY | NAME_ID_FULL_NAME | NAME_ID_TYPOGRAPHIC_FAMILY
            ) {
                continue;
            }
            if !name.is_unicode() {
                continue;
            }
            if let Some(text) = name.to_string() {
                let key = normalize_font_name(&text);
                if !key.is_empty() {
                    names.insert(key);
                }
            }
        }
    }
    any_parsed
}

/// Extract `YCbCr Matrix:` from ASS/SSA [Script Info].
fn parse_ass_matrix(content: &str) -> Option<String> {
    let mut in_script_info = false;
    for raw in content.lines() {
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_script_info = line.eq_ignore_ascii_case("[Script Info]");
            continue;
        }
        if !in_script_info {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("YCbCr Matrix") {
            let v = value.trim();
            if v.is_empty() {
                return None;
            }
            return Some(v.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        banner_effect_requires_avs, find_missing_img_paths, parse_ass_matrix, parse_ass_sections,
        resolve_subtitle_resource_path,
    };
    use std::path::Path;

    #[test]
    fn parses_tv_709() {
        let ass = "[Script Info]\nTitle: x\nYCbCr Matrix: TV.709\n[V4+ Styles]\n";
        assert_eq!(parse_ass_matrix(ass).as_deref(), Some("TV.709"));
    }

    #[test]
    fn parses_utf8_bom_script_info() {
        let ass = "\u{feff}[Script Info]\nTitle: x\nYCbCr Matrix: TV.601\n[V4+ Styles]\n";
        assert_eq!(parse_ass_matrix(ass).as_deref(), Some("TV.601"));
    }

    #[test]
    fn case_insensitive_key_keeps_value_case() {
        let ass = "[Script Info]\nycbcr matrix:   pc.601  \n";
        assert_eq!(parse_ass_matrix(ass).as_deref(), Some("pc.601"));
    }

    #[test]
    fn ignores_field_outside_script_info() {
        let ass = "[Script Info]\nTitle: x\n[Events]\nYCbCr Matrix: TV.601\n";
        assert!(parse_ass_matrix(ass).is_none());
    }

    #[test]
    fn missing_field_returns_none() {
        let ass = "[Script Info]\nTitle: x\nPlayResX: 1920\n";
        assert!(parse_ass_matrix(ass).is_none());
    }

    #[test]
    fn empty_value_returns_none() {
        let ass = "[Script Info]\nYCbCr Matrix:    \n";
        assert!(parse_ass_matrix(ass).is_none());
    }

    #[test]
    fn parses_ass_styles_fonts_and_used_styles() {
        let ass = "[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default, Arial, 20\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:01.00,Missing,{\\fnCustom Font}Hi\n";
        let parsed = parse_ass_sections(ass);
        assert!(parsed.defined_styles.contains("default"));
        assert_eq!(parsed.used_styles[0].style, "Missing");
        assert!(parsed.fonts.iter().any(|font| font.font == "Arial"));
        assert!(parsed.fonts.iter().any(|font| font.font == "Custom Font"));
    }

    #[test]
    fn resolves_relative_img_paths_against_subtitle_dir() {
        let resolved = resolve_subtitle_resource_path("res\\100.png", Some(Path::new("E:\\sub")));
        assert_eq!(resolved.to_string_lossy(), "E:\\sub\\res\\100.png");
    }

    #[test]
    fn detects_missing_img_paths() {
        let ass = r"[Events]
Dialogue: 0,0:00:00.00,0:00:01.00,Default,{\1img(E:\missing\100.png)}Hi";
        let issues = find_missing_img_paths(ass, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].tag, "\\1img");
        assert_eq!(issues[0].line, 2);
    }

    // ────────── Banner Effect 字段检测 ──────────

    #[test]
    fn lowercase_banner_always_requires_avs() {
        // 小写 banner 任意参数都必须走 AVS
        assert!(banner_effect_requires_avs("banner;8"));
        assert!(banner_effect_requires_avs("banner;8;0"));
        assert!(banner_effect_requires_avs("banner;8;0;0"));
        assert!(banner_effect_requires_avs("banner;8;0;50"));
    }

    #[test]
    fn proper_case_banner_without_fadeawaywidth_is_normal_compress() {
        // 缺省 fadeawaywidth → 普通压制
        assert!(!banner_effect_requires_avs("Banner;8"));
        assert!(!banner_effect_requires_avs("Banner;8;0"));
        assert!(!banner_effect_requires_avs("Banner;8;1"));
    }

    #[test]
    fn proper_case_banner_with_zero_fadeawaywidth_is_normal_compress() {
        // fadeawaywidth = 0 显式给出 → 普通压制
        assert!(!banner_effect_requires_avs("Banner;8;0;0"));
        assert!(!banner_effect_requires_avs("Banner;8;1;0"));
    }

    #[test]
    fn proper_case_banner_with_nonzero_fadeawaywidth_requires_avs() {
        // fadeawaywidth ≠ 0 → AVS
        assert!(banner_effect_requires_avs("Banner;8;0;50"));
        assert!(banner_effect_requires_avs("Banner;8;1;30"));
        assert!(banner_effect_requires_avs("Banner;8;0;1"));
    }

    #[test]
    fn banner_other_casing_does_not_match() {
        // 严格大小写：全大写 / 混合大小写不被识别为 banner
        assert!(!banner_effect_requires_avs("BANNER;8;0;50"));
        assert!(!banner_effect_requires_avs("BaNnEr;8;0;50"));
        assert!(!banner_effect_requires_avs("bANNER;8;0;50"));
    }

    #[test]
    fn non_banner_effect_does_not_match() {
        assert!(!banner_effect_requires_avs(""));
        assert!(!banner_effect_requires_avs("Karaoke"));
        assert!(!banner_effect_requires_avs("Scroll up;10;100;5;0"));
        assert!(!banner_effect_requires_avs("Scroll down;10;100;5;0"));
    }

    #[test]
    fn banner_with_unparseable_fadeawaywidth_falls_back_to_normal() {
        // 解析失败按缺省处理，不强制 AVS
        assert!(!banner_effect_requires_avs("Banner;8;0;abc"));
        assert!(!banner_effect_requires_avs("Banner;8;0;"));
    }

    #[test]
    fn parses_banner_hits_from_events_section() {
        let ass = "\
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,Banner;8;0;50,Roll text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,Banner;8;0;0,Quiet roll
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,Banner;8,Default roll
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,banner;8,Lowercase
Comment: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,Karaoke,Not banner
";
        let parsed = parse_ass_sections(ass);
        // 命中：第 3 行 Banner;8;0;50、第 6 行 banner;8
        // 不命中：Banner;8;0;0、Banner;8、Karaoke
        assert_eq!(parsed.banner_hits.len(), 2);
        assert_eq!(parsed.banner_hits[0].line, 3);
        assert_eq!(parsed.banner_hits[0].raw, "Banner;8;0;50");
        assert_eq!(parsed.banner_hits[1].line, 6);
        assert_eq!(parsed.banner_hits[1].raw, "banner;8");
    }

    #[test]
    fn parses_banner_hits_when_effect_column_reordered() {
        // Format 列序变动，Effect 移到 Text 前最后一列；命中仍应按列名匹配
        let ass = "\
[Events]
Format: Layer, Start, End, Style, Name, Effect, Text
Dialogue: 0,0:00:00.00,0:00:01.00,Default,,Banner;8;0;50,Hi
";
        let parsed = parse_ass_sections(ass);
        assert_eq!(parsed.banner_hits.len(), 1);
        assert_eq!(parsed.banner_hits[0].raw, "Banner;8;0;50");
    }

    #[test]
    fn no_banner_hits_when_effect_is_empty_or_unrelated() {
        let ass = "\
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,Hi
Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,Karaoke,Hi2
";
        let parsed = parse_ass_sections(ass);
        assert!(parsed.banner_hits.is_empty());
    }
}
