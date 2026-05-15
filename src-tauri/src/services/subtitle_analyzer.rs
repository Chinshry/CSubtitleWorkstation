use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct SubtitleAnalysis {
    pub has_effects: bool,
    pub detected_tags: Vec<String>,
    /// ASS [Script Info] 段中的 YCbCr Matrix 字段原始值，例如 "TV.709"、"PC.601"、"None"
    /// 仅 ASS/SSA 格式可能存在；其它字幕格式或字段缺失时为 None
    pub ass_matrix: Option<String>,
}

pub fn analyze_subtitle(subtitle_path: &str) -> Result<SubtitleAnalysis, String> {
    let path = Path::new(subtitle_path);
    if !path.exists() {
        return Ok(SubtitleAnalysis {
            has_effects: false,
            detected_tags: vec![],
            ass_matrix: None,
        });
    }

    let content = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read subtitle file: {err}"))?;

    let mut detected_tags: Vec<String>;
    let mut has_effects = false;
    let mut tag_set = HashSet::new();

    // 只检测 VSFilterMod 特有的扩展标签（不在标准 ASS 中的）
    // 这些标签需要用 AVS 压制才能正确渲染
    let effect_patterns = [
        // 一、基础缩放/模糊/偏移
        (r"\\fsc", "字体缩放"),           // 字体整体缩放（同 \fscx+\fscy）
        (r"\\xblur", "X轴模糊"),         // X 方向边缘模糊（高斯）
        (r"\\yblur", "Y轴模糊"),         // Y 方向边缘模糊（高斯）
        (r"\\fsvp", "纵向偏移"),         // 纵向偏移（上下移动，基点不变）
        (r"\\fshp", "横向偏移"),         // 横向偏移（左右移动，基点不变）

        // 二、渐变（颜色+透明度）
        (r"\\[1-4]?vc", "四角颜色渐变"),       // 四角颜色渐变（\1vc主色、\2vc次色、\3vc边框、\4vc阴影）
        (r"\\[1-4]?va", "四角透明度渐变"),     // 四角透明度渐变（同上 1/2/3/4）

        // 三、抖动/变形
        (r"\\jitter", "随机抖动"),       // 字幕随机抖动
        (r"\\rnd", "边界随机扭曲"),      // 边界随机扭曲（\rndx、\rndy、\rndz）
        (r"\\distort", "四角扭曲变形"),  // 四角扭曲变形
        (r"\\frs", "基线倾斜"),          // 基线倾斜

        // 四、3D/空间
        (r"\\z", "Z坐标"),              // Z 坐标（远近感，配合 \frx/\fry）
        (r"\\ortho", "正交投影"),        // 正交投影（做 3D 效果）

        // 五、特殊移动（曲线/圆/遮罩）
        (r"\\mover", "圆形/椭圆移动"),   // 圆形/椭圆/螺旋移动（极坐标）
        (r"\\moves[34]", "贝塞尔曲线移动"), // 三阶/四阶贝塞尔曲线移动
        (r"\\movevc", "向量遮罩移动"),   // 独立移动矢量遮罩（\clip/\iclip）

        // 六、图片填充
        (r"\\[1-4]?img", "图片填充"),          // 用 PNG 图片替代颜色填充（\1img主色、\2img次色、\3img边框、\4img阴影）
    ];

    for (pattern, label) in &effect_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&content) {
                has_effects = true;

                // 提取所有匹配到的具体标签
                for cap in re.find_iter(&content) {
                    let matched_tag = cap.as_str();
                    let tag_display = format!("{}({})", matched_tag, label);
                    tag_set.insert(tag_display);
                }
            }
        }
    }

    detected_tags = tag_set.into_iter().collect();
    detected_tags.sort();

    let ass_matrix = parse_ass_matrix(&content);

    Ok(SubtitleAnalysis {
        has_effects,
        detected_tags,
        ass_matrix,
    })
}

/// 从 ASS/SSA 文件 [Script Info] 段中提取 `YCbCr Matrix:` 字段值。
/// 标准取值：TV.601 / TV.709 / TV.2020 / PC.601 / PC.709 / PC.2020 / TV.FCC / TV.240M / None
/// 注意：字段名大小写不敏感；值的前后空白会被裁掉，但保留原始大小写。
fn parse_ass_matrix(content: &str) -> Option<String> {
    let mut in_script_info = false;
    for raw in content.lines() {
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        // 段头：[Script Info]、[V4+ Styles]、[Events] ……
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
    use super::parse_ass_matrix;

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
        // 出现在 [Events] 段里的同名键不应被采纳
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
}
