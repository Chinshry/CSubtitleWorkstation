use std::fs;
use std::path::Path;

pub struct SubtitleAnalysis {
    pub has_effects: bool,
    pub detected_tags: Vec<String>,
}

pub fn analyze_subtitle(subtitle_path: &str) -> Result<SubtitleAnalysis, String> {
    let path = Path::new(subtitle_path);
    if !path.exists() {
        return Ok(SubtitleAnalysis {
            has_effects: false,
            detected_tags: vec![],
        });
    }

    let content = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read subtitle file: {err}"))?;

    let mut detected_tags = Vec::new();
    let mut has_effects = false;

    // 只检测 VSFilterMod 特有的扩展标签（不在标准 ASS 中的）
    // 这些标签需要用 AVS 压制才能正确渲染
    let effect_patterns = [
        (r"\fsc", "字体缩放"),           // VSFilterMod 扩展：聚合缩放
        (r"\xblur", "X轴模糊"),         // VSFilterMod 扩展：X轴高斯模糊
        (r"\yblur", "Y轴模糊"),         // VSFilterMod 扩展：Y轴高斯模糊
        (r"\fsvp", "竖直行距"),         // VSFilterMod 扩展：竖直行距
        (r"\fshp", "字体锐化"),         // VSFilterMod 扩展：字体锐化
        (r"\img", "图片填充"),          // VSFilterMod 扩展：用图片替代颜色填充
        (r"\mover", "圆形移动"),        // VSFilterMod 扩展：圆形/椭圆/螺旋轨迹移动
        (r"\moves", "样条移动"),        // VSFilterMod 扩展：样条曲线移动
        (r"\movevc", "向量裁剪移动"),   // VSFilterMod 扩展：可移动的向量裁剪
        (r"\vc", "渐变色"),             // VSFilterMod 扩展：颜色渐变
        (r"\va", "渐变透明度"),         // VSFilterMod 扩展：透明度渐变
        (r"\jitter", "抖动"),           // VSFilterMod 扩展：位置抖动
    ];

    for (pattern, label) in &effect_patterns {
        if content.contains(pattern) {
            has_effects = true;
            let tag_display = format!("{}({})", pattern, label);
            if !detected_tags.contains(&tag_display) {
                detected_tags.push(tag_display);
            }
        }
    }

    Ok(SubtitleAnalysis {
        has_effects,
        detected_tags,
    })
}
