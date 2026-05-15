// 旧 BAT 兼容入口：解析 ASS 文件中带 .png 的 LOGO 行。主流程已切到可视化编辑器
// （commands/video.rs::extract_video_frame + LogoEditor.vue），这里保留供"从 ASS 导入"
// 一类的辅助功能后续接入。
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AssLogo {
    pub image_path: String,
    pub position_x: i32,
    pub position_y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn parse_ass_logo(
    subtitle_path: &str,
    video_width: Option<i32>,
    video_height: Option<i32>,
    logo_dir: Option<&str>,
) -> Result<Option<AssLogo>, String> {
    let raw = fs::read_to_string(subtitle_path)
        .map_err(|err| format!("Failed to read subtitle file: {err}"))?;

    let logo_line = match raw
        .lines()
        .find(|line| line.to_ascii_lowercase().contains(".png"))
    {
        Some(line) => line.trim(),
        None => return Ok(None),
    };

    if logo_line.to_ascii_lowercase().starts_with("dialogue:") {
        return Err("ASS logo line is not commented. Please comment the logo dialogue line before compressing.".to_string());
    }

    let play_res_x = find_ass_number(&raw, "PlayResX");
    let play_res_y = find_ass_number(&raw, "PlayResY");
    let scale = match (play_res_x, play_res_y, video_width, video_height) {
        (Some(px), Some(py), Some(vw), Some(vh)) if px > 0 && py > 0 => {
            let sx = vw as f64 / px as f64;
            let sy = vh as f64 / py as f64;
            if (sx - sy).abs() < 0.01 {
                sx
            } else {
                1.0
            }
        }
        _ => 1.0,
    };

    let image_name = extract_png_name(logo_line)
        .ok_or_else(|| "Could not parse logo png name from ASS line.".to_string())?;
    let (x, y) = extract_position(logo_line)
        .ok_or_else(|| "Could not parse logo position from ASS line.".to_string())?;
    let (width, height) = extract_size(logo_line)
        .ok_or_else(|| "Could not parse logo size from ASS line.".to_string())?;

    let subtitle_dir = Path::new(subtitle_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let image_path = resolve_logo_path(subtitle_dir, &image_name, logo_dir)?;

    Ok(Some(AssLogo {
        image_path: image_path.to_string_lossy().to_string(),
        position_x: (x as f64 * scale).round() as i32,
        position_y: (y as f64 * scale).round() as i32,
        width: (width as f64 * scale).round() as i32,
        height: (height as f64 * scale).round() as i32,
    }))
}

fn find_ass_number(raw: &str, key: &str) -> Option<i32> {
    raw.lines().find_map(|line| {
        let (left, right) = line.split_once(':')?;
        if left.trim().eq_ignore_ascii_case(key) {
            right.trim().parse::<i32>().ok()
        } else {
            None
        }
    })
}

fn extract_png_name(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let end = lower.find(".png")? + 4;
    let start = line[..end]
        .rfind(|c: char| c == '}' || c == '/' || c == '\\' || c == ',' || c.is_whitespace())
        .map(|index| index + 1)
        .unwrap_or(0);
    Some(line[start..end].trim().to_string())
}

fn extract_position(line: &str) -> Option<(i32, i32)> {
    let start = line.find('(')? + 1;
    let end = line[start..].find(')')? + start;
    let inside = &line[start..end];
    let mut nums = inside
        .split(|c: char| c == ':' || c == ',' || c.is_whitespace())
        .filter_map(|part| part.trim().parse::<i32>().ok());
    Some((nums.next()?, nums.next()?))
}

fn extract_size(line: &str) -> Option<(i32, i32)> {
    let normalized = line.replace('\\', " ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    for window in tokens.windows(6) {
        if window[0].eq_ignore_ascii_case("l") {
            let width = window[1].parse::<i32>().ok()?;
            let height = window[3].parse::<i32>().ok()?;
            if width > 0 && height > 0 {
                return Some((width, height));
            }
        }
    }
    None
}

fn resolve_logo_path(
    subtitle_dir: &Path,
    image_name: &str,
    logo_dir: Option<&str>,
) -> Result<PathBuf, String> {
    let mut tried: Vec<PathBuf> = Vec::new();

    // 1) 用户在表单上指定的 logo 目录（最高优先）
    if let Some(dir) = logo_dir.map(str::trim).filter(|s| !s.is_empty()) {
        let candidate = Path::new(dir).join(image_name);
        if candidate.exists() {
            return Ok(candidate);
        }
        tried.push(candidate);
    }

    // 2) 字幕同级目录
    let candidate = subtitle_dir.join(image_name);
    if candidate.exists() {
        return Ok(candidate);
    }
    tried.push(candidate);

    // 3) 字幕同级 res/logo
    let candidate = subtitle_dir.join("res").join("logo").join(image_name);
    if candidate.exists() {
        return Ok(candidate);
    }
    tried.push(candidate);

    // 4) 字幕上级 res/logo
    let candidate = subtitle_dir
        .parent()
        .unwrap_or(subtitle_dir)
        .join("res")
        .join("logo")
        .join(image_name);
    if candidate.exists() {
        return Ok(candidate);
    }
    tried.push(candidate);

    let tried_list = tried
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n  - ");
    Err(format!(
        "找不到 logo 图片 \"{image_name}\"。请在表单「logo 检测目录」中填入图片所在目录。已尝试：\n  - {tried_list}"
    ))
}
