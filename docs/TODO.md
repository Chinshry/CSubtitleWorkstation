# TODO

## 应用更新 / 发布

- [x] GitHub Pages 固定更新检查地址：`https://chinshry.github.io/CSubtitleWorkstation/updates/latest.json`。
- [x] 更新清单文件：`docs/updates/latest.json`。
- [x] 设置页提供“检查应用更新”按钮。
- [x] 启动时自动检查更新开关；静默检查，发现更新后通过右上角 Toast 通知。
- [x] 检测到更新后显示版本号、发布时间和完整更新说明。
- [x] 检测到新版本后引导用户前往 GitHub Releases 下载，不做应用内自动安装。
- [x] 全局 `AppToast` 组件 + `useToast` composable。
- [x] GitHub Pages 已按 `master` 分支 `/docs` 目录托管。
- [x] `release.yml` 支持 workflow_dispatch 一键发布。
- [x] workflow_dispatch 自动更新 `package.json`、`src-tauri/Cargo.toml` 和 `docs/updates/latest.json`。
- [x] workflow_dispatch 自动创建 draft release、构建 Windows NSIS 和 macOS Universal DMG、上传产物并发布。
- [x] 发布后自动用 GitHub Release `publishedAt` 回写 `docs/updates/latest.json` 的 `pub_date`。
- [x] 应用当前版本显示改为读取前端 `package.json`，避免安装包内 Tauri 命令版本显示不稳定。
- [ ] 每次正式发布后，用旧版本验证能检测到新版本并打开对应 Release 页面。

## 编码预设 / 输出模板

- [x] 内置 5 套编码预设：x264 平衡 / x265 体积优先 / NVENC 快速 / AMF 快速 / Apple 快速。
- [x] 支持新增、编辑、删除自定义预设，`customVideoArgs` 可自由扩展 ffmpeg 视频参数。
- [x] 编码预设导入导出 JSON。
- [x] 多套输出文件名模板，支持 `{video_name}` / `{resolution}` / `{encoder}` / `{crf}` / `{date:YYYYMMDD}` 等变量。
- [x] 两种输出目录策略：与视频同目录 / 固定目录。
- [ ] 输出模板支持导入导出 JSON。

## 字幕检查

- [x] `SubtitleCheckPanel` 集中展示图片路径、字体、样式错误、ASS 色彩矩阵警告、VSFilterMod 特效标签建议。
- [x] 移除独立的 `ColorMatrixWarningBanner`，整合到 `SubtitleCheckPanel`。

## 跨平台

- [x] macOS 启动检测 ffmpeg `subtitles` / `ass` filter 可用性，缺失时提示安装 `ffmpeg-full`。
- [x] macOS `ffmpeg_locator` 优先匹配 Homebrew 路径下的 `ffmpeg-full`。
- [x] `-vf` 参数按平台分流构建：macOS 使用 `filename=<path>` 显式形式并转义特殊字符，Windows 保留单引号 + 反斜杠双倍方案。
- [ ] 验证 macOS 公证 / 签名流程后再启用；当前仍依赖用户手动绕过 Gatekeeper。

## 体验

- [x] 引入 `tauri-plugin-window-state`，记忆窗口位置、尺寸和最大化状态。
- [x] 自定义沉浸式标题栏 `TitleBar.vue`。
- [x] 命令预览面板支持折叠和复制。

## 临时文件清理

- [ ] 压制完成后自动清理 AVS 临时脚本：每次 AVS 模式压制生成 `app_local_data_dir/avs-temp/input.avs` 后，在成功、失败或取消时删除该脚本。
- [ ] 设置页添加“清理临时文件”按钮：手动清理 `app_local_data_dir/filter-temp` 和 `app_local_data_dir/avs-temp` 下的残留临时文件。

## ASS 语法解析器扩展

当前 `subtitle_analyzer.rs` 已实现：特效标签检测、YCbCr Matrix 解析、字体、样式和图片引用检查。后续可扩展：

- [ ] ASS 格式规范化 / 修复：自动修复常见格式问题、统一脚本编码、修复不规范时间码。
- [ ] ASS 兼容性检查：检测播放器或 AVS 兼容性、样式冲突、重复定义、可能无法渲染的特殊标签。
- [ ] 性能预测：扫描复杂特效密度，预测渲染压力，识别可能导致压制变慢的模式。
- [ ] 字幕统计分析：统计行数、总时长、平均每行时长、字体/颜色/特效使用频率。
- [ ] ASS 转换：高质量 ASS -> SRT 转换，保留关键文本信息。
- [ ] 智能建议：根据字幕复杂度建议是否启用 AVS，根据特效类型建议编码预设。
