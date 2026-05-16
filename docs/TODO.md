# TODO

## 应用更新

- [x] GitHub Pages 固定更新检查地址：`https://chinshry.github.io/CSubtitleWorkstation/updates/latest.json`。
- [x] 更新清单模板：`docs/updates/latest.json`。
- [x] 设置页提供"检查应用更新"按钮。
- [x] 检测到更新后显示版本号和更新说明。
- [x] 检测到新版后引导用户前往 GitHub Releases 下载，不做应用内自动安装。
- [x] 支持"启动时自动检查更新"开关；启动检查只静默运行，有更新时通过右上角 Toast 通知。
- [x] 引入全局 `AppToast` 组件 + `useToast` composable，承载更新提示与其他通知。
- [ ] 在仓库 Settings -> Pages 中启用 `main` 分支的 `/docs` 目录。
- [ ] 发布 `0.1.1` 后测试旧版本能检测到更新，并能打开 GitHub Releases 下载页面。

## 编码预设 / 输出模板

- [x] 内置 5 套编码预设（x264 平衡 / x265 体积优先 / NVENC 快速 / AMF 快速 / Apple 快速）。
- [x] 支持新增/编辑/删除自定义预设，`customVideoArgs` 自由扩展 ffmpeg 参数。
- [x] 编码预设导入导出 JSON。
- [x] 多套输出文件名模板 + `{video_name}` / `{resolution}` / `{encoder}` / `{crf}` / `{date:YYYYMMDD}` 变量。
- [x] 三种输出目录策略：与视频同目录 / 固定目录 / 每次手动选择。
- [ ] 输出模板支持导入导出 JSON（目前仅编码预设支持）。

## 字幕检查

- [x] `SubtitleCheckPanel` 集中展示风险：VSFilterMod 特效标签 + ASS 色彩矩阵不匹配警告。
- [x] 移除独立的 `ColorMatrixWarningBanner`，整合到 `SubtitleCheckPanel`。

## 跨平台

- [x] macOS 启动检测 ffmpeg `subtitles` / `ass` filter 可用性，缺失时直接提示装 `ffmpeg-full`。
- [x] macOS `ffmpeg_locator` 优先匹配 Homebrew 路径下的 `ffmpeg-full`。
- [x] `-vf` 参数按平台分流构建：macOS 用 `filename=<path>` 显式形式 + 空格/冒号/逗号/分号转义；Windows 沿用单引号 + 反斜杠双倍。

## 体验

- [x] 引入 `tauri-plugin-window-state`，记忆窗口位置/尺寸/最大化状态。
- [x] 自定义沉浸式标题栏 `TitleBar.vue`。
- [x] 命令预览面板支持折叠/复制。

## 临时文件清理

- [ ] 压制完成后自动清理 AVS 临时脚本：每次 AVS 模式压制生成 `app_local_data_dir/avs-temp/input.avs` 后，压制完成（成功/失败/取消）时删除该脚本。
- [ ] 设置页添加"清理临时文件"按钮：手动清理 `app_local_data_dir/filter-temp` 和 `app_local_data_dir/avs-temp` 下的所有残留临时文件。

## ASS 语法解析器扩展

当前 `subtitle_analyzer.rs` 已实现：特效标签检测、YCbCr Matrix 解析、字体/样式/图片引用检查。后续可扩展方向：

- [ ] **ASS 格式规范化/修复**：自动修复常见格式问题（行尾空格、缺失字段）、统一脚本编码（GBK/UTF-8）、修复不规范的时间码格式。
- [ ] **ASS 兼容性检查**：检测与不同播放器/AVS 的兼容性、样式冲突或重复定义、识别可能不被渲染的特殊标签。
- [ ] **性能预测**：扫描复杂特效密度，预测渲染压力；识别可能导致压制缓慢的模式。
- [ ] **字幕统计分析**：统计行数、总时长、平均每行时长；分析字体/颜色/特效使用频率；识别冗余或未使用的样式定义。
- [ ] **ASS 转换**：高质量 ASS → SRT 转换（保留关键信息）；样式简化为纯文本备选方案。
- [ ] **智能建议**：基于字幕复杂度建议是否需要 AVS 模式；基于特效类型建议合适的编码预设；检测可能导致压制失败的模式。

## CI / 发布

- [x] GitHub Actions 接入自动发版流水线（Windows NSIS + macOS Universal DMG）。
- [x] release.yml 支持一键覆盖发布（同一 tag 重新发包覆盖旧 asset）。
- [ ] 验证 macOS 公证 / 签名流程后再启用（当前仍依赖用户手动绕过 Gatekeeper）。
