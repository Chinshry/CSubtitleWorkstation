use regex::Regex;
use std::collections::HashSet;
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

    Ok(SubtitleAnalysis {
        has_effects,
        detected_tags,
    })
}
