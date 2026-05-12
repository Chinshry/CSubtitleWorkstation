# TODO

## 打包

- 修复并验证 Windows MSI 打包。
- 准备正式版的应用图标资源。
- 配置 release 构建产物输出目录。
- 在 macOS 上验证 `.app` / `.dmg` 打包流程。
- 制定 Windows 与 macOS 的代码签名方案。

## 应用更新

- 生成 Tauri updater 密钥对。
- 把私钥放到仓库之外保存。
- 替换 `src-tauri/tauri.conf.json` 中占位的 updater 公钥。
- 准备静态更新 JSON。
- 确定更新文件的最终托管位置。
- 测试 `0.1.0` 到更高版本的更新流程。

## 可视化 LOGO 叠加（替代 ASS LOGO 行解析）

目标：放弃当前从 ASS 文件解析 LOGO 行的方式，改为提供一个独立的 LOGO 开关。开启后弹出可视化窗口，预览视频画面、自由摆放 LOGO，最终把位置和大小写入压制命令。

### 用户流程

1. 压制表单上保留 `压制 LOGO` 开关；移除"LOGO 检测目录"输入。
2. 开启开关后点击 `配置 LOGO` 按钮 → 打开 LOGO 编辑器窗口（独立 Tauri 窗口或模态层）。
3. 编辑器内：
   - 上方：视频播放/抽帧预览区域；播放控制条可拖动到任意时间点。
   - 左侧：`选择 LOGO 图片` 按钮 + 最近使用记录列表（缩略图 + 路径）。
   - 画面内 LOGO 可拖动定位、四角拖拽缩放；缩放必须**保持图片宽高比不变**（按住 Shift 也保持，单纯禁止变形）。
   - 顶部状态条显示当前位置（x, y）与尺寸（w, h），以及相对原视频分辨率的百分比。
4. 确认后回到主页，开关旁标签显示 `已配置：xxx.png · 位置 …% · 大小 …%`。

### 持久化

- 新增配置字段（`AppConfig` 与 `config_store`）：
  - `recentLogos: { path: string; lastUsedAt: number }[]`（最多保留 8–10 项，按时间倒序）。
  - `lastLogoLayout: { path: string; xPct: number; yPct: number; wPct: number; hPct: number }`（按百分比存，跨分辨率视频时按比例自适应；用户也可在编辑器里勾选"绝对像素"切换存储方式）。
- 打开编辑器时默认载入 `lastLogoLayout`；如当前 LOGO 路径与上次不同，仍套用上次的位置/大小百分比作为起点。

### 视频预览

- 用 ffmpeg 抽帧到本地临时目录（如 `-ss <t> -frames:v 1` 输出 PNG），前端 `<img>` 显示。
- 编辑器内的"播放"按钮逐帧或按秒抽帧刷新；不强求实时播放。
- 抽帧 fps 上限做节流，避免拖动播放头时频繁 fork ffmpeg。

### 压制命令对接

- 压制开始时，若 LOGO 开关开启且有有效布局：
  - 读取真实视频分辨率，把百分比换算回像素 → 生成 `overlay=x:y` 与 `scale=w:h` 参数。
  - 复用现有 filter graph 构造逻辑（参考 `services/command_builder.rs:build_preview` 中 `movie='...',scale=W:H[wm];[in][wm]overlay=X:Y`）。
- 移除 `services/ass_logo.rs` 对 ASS 文件的解析；如需保留旧 BAT 兼容入口，做成可选导入"从 ASS LOGO 行导入位置和大小"，仅作为初始值填充编辑器。

### 边界与细节

- 图片格式至少支持 PNG、JPG、WebP；带 alpha 通道的 PNG 应正确叠加。
- LOGO 在画面外拖拽时做边界吸附或允许越界（按需，倾向允许但给出提示）。
- 视频文件未填写或解析失败时，禁用 `配置 LOGO` 按钮。
- 编辑器关闭未保存时弹出确认。
