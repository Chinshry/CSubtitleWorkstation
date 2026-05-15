# CC字幕压制工作站（CSubtitleWorkstation）需求整理与技术方案设计

## 1. 项目目标

将现有 `@字幕压制V260204.bat` 字幕压制脚本升级为名为 **CSubtitleWorkstation / CC字幕压制工作站** 的 Windows / macOS 通用桌面应用。

```text
项目名：CSubtitleWorkstation
仓库名：CSubtitleWorkstation
中文名：CC字幕压制工作站
```

应用本身负责提供图形界面、配置管理、任务执行、日志展示、应用更新检测等能力；`ffmpeg` 不随应用内置，由用户本机环境提供，应用只负责检测、选择、调用和版本提示。

## 2. 技术选型结论

推荐使用：

```text
桌面框架：Tauri 2
前端：Vue 3 + TypeScript
后端：Rust commands
应用更新：GitHub Pages 静态 JSON + GitHub Releases
配置存储：本地 JSON / TOML
外部依赖：用户本机 ffmpeg
```

选择 Tauri 的原因：

- 安装包体积通常明显小于 Electron，因为 Tauri 使用系统 WebView，不内置 Chromium。
- 前端仍可使用 TypeScript、Vue、React 等 Web 技术开发页面。
- Rust 层适合处理本地文件、子进程、路径、日志流、任务取消等桌面能力。
- 支持 Windows / macOS 打包。
- 通过 GitHub Pages 静态 JSON 检测应用本体更新。

不推荐 Electron 作为首选的原因：

- 对本工具来说 Electron 偏重，安装包和运行内存更大。
- 应用核心只是本地工作流管理和 `ffmpeg` 调用，不需要完整 Chromium + Node 运行时。

## 3. 需求整理

### 3.1 基础压制需求

- 支持选择或拖拽视频文件。
- 支持选择或拖拽字幕文件。
- 支持常见视频格式：
  - `mp4`
  - `mkv`
  - `mov`
  - `ts`
- 支持字幕格式：
  - `ass`
  - `srt`
- 支持输出文件名模板。
- 支持选择输出目录。
- 支持开始压制、取消压制。
- 支持实时显示 `ffmpeg` 日志。
- 支持压制完成、失败、取消三种状态。

### 3.2 压制参数需求

从现有脚本迁移的核心参数：

- `CRF`
- 是否压制 logo：`NeedLogo`
- 是否启用反交错：`NeedYadif`
- 最大码率：`MaxBitrate`
- 编码器选择
- 是否启用字幕压制
- **AVS 压制模式**：仅 Windows 兼容功能（✅ 已实现）

建议第一版支持的编码器：

```text
CPU：libx264
NVIDIA：h264_nvenc
AMD：h264_amf
macOS：h264_videotoolbox
```

#### AVS 兼容模式实现（已完成）

**需求**：
- 检测 Windows 上是否安装了 AviSynth / AviSynth+。
- 增加仅在 Windows 启用的 AVS 模式开关。
- 在 macOS 上隐藏或禁用 AVS 模式。
- 缺少 AVS 依赖时给出明确警告。

**实现方案**：
- 后端 `services/avs_detector.rs`：通过 `ffmpeg -demuxers` 检测 avisynth demuxer，通过注册表 + System32 检测 AviSynth.dll 与版本号。
- 前端 `stores/avsStore.ts`：缓存检测结果，支持调试 mock 层（模拟 AviSynth+ 或 demuxer 缺失）。
- 压制表单 `CompressForm.vue`：AVS 开关仅 Windows 可勾，缺失依赖时禁用并 tooltip 提示。
- 设置页 `SettingsView.vue`：独立 AVS 设置面板，显示双徽章（demuxer / AviSynth+ 状态），缺失时给出安装手册。
- 命令构建 `services/command_builder.rs`：`use_avs=true` 时，ffmpeg 输入改为 AviSynth 脚本，字幕由脚本内 TextSubMod 渲染（跳过 ffmpeg subtitles 滤镜），LOGO overlay 与 yadif 仍然有效。
- 工作目录 `services/avs_workspace.rs`：在 `app_local_data_dir/avs-temp/` 生成 input.avs，包含内置 DLL 的 LoadPlugin 指令与绝对路径引用。

### 3.3 LOGO 叠加需求

采用**可视化 LOGO 叠加**方式：

- 压制表单上保留"压制 LOGO"开关；点击「配置 LOGO」按钮打开模态编辑器 `LogoEditor.vue`。
- 编辑器内对视频抽帧预览，鼠标拖放定位、四角拖拽缩放（保持图片宽高比，禁止形变）。
- LOGO 位置/尺寸按视频百分比存储（`xPct/yPct/wPct/hPct`），自动适配不同分辨率视频。
- 抽帧由 `services/frame_extractor.rs` 用 ffmpeg `-ss <t> -frames:v 1` 写入应用临时目录，前端 `<img>` 通过 `convertFileSrc` 加载。
- 编辑器关闭时清理抽帧缓存。

**分辨率桶记忆**：布局按 `(LOGO 路径, 分辨率桶)` 维度独立持久化。仅 6 种常见桶会写入：

```text
720p-landscape / 720p-portrait
1080p-landscape / 1080p-portrait
4k-landscape / 4k-portrait
```

长边 / 短边 ±8 像素容差，兼容 `1920×1088` 这类 mod16 对齐尺寸；切换 LOGO 或视频时优先按 `(currentBucket, path)` 命中记忆并自动应用。

**旋转视频修正**：手机录制的视频常存为 `1920×1080 + rotation=90`，ffmpeg 解码后画面是 `1080×1920`。`services/video_meta.rs` 同时解析 ffprobe 的 `tags.rotate` / `side_data_list[].rotation` 与 ffmpeg `-i` 文本中的 `displaymatrix: rotation of ...`，±90/±270 时交换 `width`/`height`，返回"显示尺寸"；`CompressJob` 携带 `videoWidth/videoHeight`，让命令构造端直接用显示尺寸算 LOGO 像素，避免压出来的 LOGO 错位或放大。

**压制命令对接**：`services/command_builder.rs::build_logo_overlay` 把百分比换算回像素后，构造 `movie='...',scale=W:H[wm];[in][wm]overlay=X:Y` 滤镜片段；AVS 模式下字幕由 TextSubMod 渲染，但 LOGO overlay 与 yadif 滤镜仍然有效。

**旧 ASS Logo 行解析**：`services/ass_logo.rs` 保留为 `#[allow(dead_code)]`，作为未来"从 ASS 导入 LOGO 行作为初始布局"一类辅助入口的预留点；主流程不再依赖它。

### 3.4 ffmpeg 检测与配置需求

应用不内置 `ffmpeg`。

应用启动时需要检测：

1. 用户是否配置了自定义 `ffmpeg` 路径。
2. 如果配置了自定义路径，优先检测该路径。
3. 如果没有配置，检测系统 `PATH` 中是否存在 `ffmpeg`。
4. 执行 `ffmpeg` 版本检测。
5. 将检测状态展示给用户。

支持用户手动选择：

- 选择 `ffmpeg.exe` / `ffmpeg` 文件。
- 选择包含 `ffmpeg` 的目录。
- 重新检测。
- 恢复使用系统环境变量。

Windows 检测规则：

```text
ffmpeg.exe
```

macOS 检测规则：

```text
ffmpeg
```

应用不要求用户安装或配置 `ffprobe`。视频信息、码率、分辨率、编码格式等信息通过 `ffmpeg -i` 或实际压制日志解析。

设计原则：

- `ffmpeg` 是唯一必需外部依赖。
- 不引入 `ffprobe`，避免增加用户安装成本和依赖体积。
- 复用现有脚本的思路，通过 `ffmpeg` 输出完成媒体信息读取。

### 3.5 ffmpeg 版本检测与升级提示

应用需要执行：

```bash
ffmpeg -version
```

解析第一行版本信息，例如：

```text
ffmpeg version 7.1.1 ...
```

前端展示：

- 当前 `ffmpeg` 路径。
- 当前 `ffmpeg` 版本。
- 来源：系统环境变量 / 用户自定义路径 / 未找到。

升级策略：

- 第一版不自动下载或覆盖用户本机 `ffmpeg`。
- 只做版本检测、状态提示、升级引导。
- Windows 给出下载页面入口。
- macOS 给出 Homebrew 安装或升级提示。

示例提示：

```text
未检测到 ffmpeg，请安装 ffmpeg 或手动选择 ffmpeg 路径。

当前 ffmpeg 版本较旧，建议升级到 7.x 或更高版本。
```

后续可选增强：

- 增加“下载到应用数据目录”的 ffmpeg 管理器。
- 但该功能应作为可选项，不应把 `ffmpeg` 打包进应用安装包。

### 3.6 应用本体更新检测需求

应用需要支持检测自身更新。

注意：应用更新与 `ffmpeg` 更新是两套独立机制。

```text
应用更新：更新本软件本体。
ffmpeg 更新：检测用户本机 ffmpeg 版本并提示升级。
```

第一版应用更新能力：

- 启动时可静默检查更新。
- 设置页提供“检查更新”按钮。
- 检测到新版本后显示版本号、更新说明。
- 引导用户前往 GitHub Releases 下载新版安装包。
- 更新失败时给出明确错误信息。

第一版使用静态 JSON：

```text
GitHub Releases / OSS / CDN
        ↓
latest.json
        ↓
前端更新检测
        ↓
提示用户前往 GitHub Releases 下载
```

当前第一版只做版本检测，不在应用内自动下载和安装。

静态更新 JSON 需要包含：

- `version`
- `notes`
- `pub_date`
- `platforms`
- 各平台安装包 URL

## 4. 页面设计

### 4.1 主页面

主页面用于单个任务快速压制。

区域：

- 视频文件拖拽区。
- 字幕文件拖拽区。
- 输出路径。
- 参数面板。
- 开始压制按钮。
- 当前任务状态。
- 实时日志。

核心控件：

- `CRF` 数字输入。
- `MaxBitrate` 数字输入。
- `NeedLogo` 开关。
- `NeedYadif` 开关。
- 编码器下拉框。
- 输出模板输入框。

### 4.2 设置页面

设置页面负责全局配置。

包括：

- `ffmpeg` 状态。
- `ffmpeg` 路径。
- `ffprobe` 状态。
- 自动检测按钮。
- 手动选择路径按钮。
- 版本检测按钮。
- 默认压制参数。
- 应用更新检测。
- AVS 模式平台提示。

AVS 模式显示规则：

- Windows：显示 AVS 模式开关，并提示需要本机正确安装 AviSynth / AviSynth+ 环境。
- macOS：隐藏或禁用 AVS 模式开关，并显示提示：`AVS 压制仅支持 Windows，macOS 将使用 ffmpeg filter 模式。`

### 4.3 任务日志页面

用于查看压制过程。

包括：

- 当前命令预览。
- `ffmpeg` 原始日志。
- 进度百分比。
- 已用时间。
- 预计剩余时间。
- 输出文件路径。
- 错误详情。

### 4.4 后续可扩展页面

第一版之后可扩展：

- 批量任务队列。
- 预设管理。
- 历史记录。
- 压制结果对比。
- 自动清理临时文件。

## 5. 系统架构设计

```text
┌─────────────────────────────────────┐
│ Vue + TypeScript 前端                │
│                                     │
│ 页面 / 表单 / 状态 / 日志展示         │
└──────────────────┬──────────────────┘
                   │ invoke / event
┌──────────────────▼──────────────────┐
│ Tauri Rust 后端                      │
│                                     │
│ ffmpeg 检测                          │
│ 配置读写                             │
│ ASS 解析                             │
│ 命令构建                             │
│ 子进程管理                           │
│ 日志流转发                           │
│ 应用更新                             │
└──────────────────┬──────────────────┘
                   │ spawn process
┌──────────────────▼──────────────────┐
│ 用户本机 ffmpeg                      │
└─────────────────────────────────────┘
```

## 6. 推荐项目结构

```text
CSubtitleWorkstation/
  package.json
  src/
    main.ts
    App.vue
    views/
      HomeView.vue
      SettingsView.vue
    components/
      VideoMetaCard.vue        # 视频信息卡
      FfmpegStatus.vue
      CompressForm.vue
      CommandPreviewCard.vue
      JobLogPanel.vue
      LogoEditor.vue           # 可视化 LOGO 摆放编辑器
    stores/
      ffmpegStore.ts
      avsStore.ts
      dropStore.ts
      platformStore.ts
    api/
      ffmpeg.ts
      compress.ts
      video.ts
      config.ts
    types.ts

  src-tauri/
    tauri.conf.json
    Cargo.toml
    resources/avs/           # 内置 AVS 插件 DLL
      VSFilterMod.dll
      LSMASHSource.dll
    src/
      lib.rs
      commands/
        ffmpeg.rs
        compress.rs
        config.rs
        updater.rs
        avs.rs               # AVS 检测命令
        video.rs             # inspect_video_meta / extract_video_frame / clear_frame_cache
      services/
        ffmpeg_locator.rs
        command_builder.rs
        avs_detector.rs      # AviSynth+ 与 demuxer 检测
        avs_workspace.rs     # AVS 工作目录与脚本生成
        video_meta.rs        # ffprobe / ffmpeg -i 解析视频元数据 + rotation 修正
        frame_extractor.rs   # LOGO 编辑器抽帧缓存
        config_store.rs      # 本地 JSON 配置读写
        ass_logo.rs          # 旧 ASS LOGO 行解析（dead_code，保留作未来导入入口）
      models/
        app_config.rs        # AppConfig / LogoLayout / LogoLayoutEntry / RecentLogo
        compress_job.rs
        ffmpeg_status.rs
        avs_status.rs        # AVS 检测结果
```

## 7. 核心数据结构

### 7.1 压制任务

```ts
type CompressJob = {
  id: string
  videoPath: string
  subtitlePath: string
  outputPath: string
  crf: number
  maxBitrate?: number              # undefined=不限制 / 0=自动(原码率+1000) / >0=自定义
  needLogo: boolean
  needYadif: boolean
  encoder: 'libx264' | 'h264_nvenc' | 'h264_amf' | 'h264_videotoolbox'
  logoDir?: string
  useAvs?: boolean                 # AVS 兼容模式开关（仅 Windows）
  logoLayout?: LogoLayout | null   # 可视化编辑器输出的 LOGO 布局（百分比）
  videoWidth?: number              # 前端传给后端的"显示尺寸"，已含 rotation 修正
  videoHeight?: number
}

type LogoLayout = {
  path: string
  xPct: number  # 左上角 X 占视频宽度的百分比 [0, 1]
  yPct: number
  wPct: number  # LOGO 宽度占视频宽度的百分比
  hPct: number
}
```

### 7.2 ffmpeg 状态

```ts
type FfmpegStatus = {
  available: boolean
  source: 'system_path' | 'custom_path' | 'not_found'
  ffmpegPath?: string
  ffmpegVersion?: string
  ffprobePath?: string
  ffprobeVersion?: string
  message?: string
}
```

### 7.3 AVS 状态

```ts
type AvsStatus = {
  supportedPlatform: boolean           # 仅 Windows = true
  ffmpegDemuxerAvailable: boolean      # ffmpeg 是否启用 --enable-avisynth
  avisynthInstalled: boolean           # 系统是否装了 AviSynth+
  avisynthVersion?: string             # AviSynth.dll FileVersion
  avisynthInstallPath?: string         # 注册表中的安装目录
  avisynthDllPath?: string             # 实际加载的 DLL 路径
  available: boolean                   # 综合判断：能否启用 AVS 模式
  message?: string                     # 缺失时的提示信息
}
```

### 7.4 应用配置

```ts
type AppConfig = {
  ffmpegMode: 'system' | 'custom'
  ffmpegPath?: string
  defaultCrf: number
  defaultNeedLogo: boolean
  defaultNeedYadif: boolean
  defaultEncoder: string
  outputNameTemplate: string
  checkUpdateOnStartup: boolean
  defaultLogoDir?: string
  defaultUseAvs?: boolean              # AVS 模式默认开关
  recentLogos?: RecentLogo[]           # LOGO 编辑器侧栏「最近使用」列表（最多 10 项）
  lastLogoLayout?: LogoLayout | null   # 未命中分辨率桶时的全局 fallback
  logoLayouts?: LogoLayoutEntry[]      # 按 (分辨率桶, LOGO 路径) 区分的布局记忆
}

type RecentLogo = {
  path: string
  lastUsedAt: number                   # Unix 毫秒
}

type LogoLayoutEntry = {
  bucket: string                       # 720p-landscape / 720p-portrait / 1080p-landscape / 1080p-portrait / 4k-landscape / 4k-portrait
  path: string
  xPct: number
  yPct: number
  wPct: number
  hPct: number
  lastUsedAt: number
}
```

## 8. Rust 后端命令设计

### 8.1 ffmpeg 相关命令

```text
detect_ffmpeg()
select_ffmpeg_path(path)
get_ffmpeg_status()
check_ffmpeg_version()
reset_ffmpeg_to_system_path()
```

### 8.2 AVS 相关命令

```text
detect_avs()                 # 检测 AviSynth+ 与 ffmpeg avisynth demuxer
```

### 8.3 压制相关命令

```text
start_compress(job)
cancel_compress(job_id)
preview_ffmpeg_command(job)
inspect_video_meta(path)           # ffprobe 优先 / ffmpeg -i 回退；返回的 width/height 已应用 rotation
extract_video_frame(path, time)    # LOGO 编辑器抽帧；返回应用临时目录中 PNG 路径
clear_frame_cache()                # 编辑器关闭时清理抽帧缓存
```

### 8.4 配置相关命令

```text
load_config()
save_config(config)
reset_config()
```

### 8.5 应用更新相关命令

```text
get_current_app_version()
```

## 9. ffmpeg 执行策略

后端使用 Rust 子进程启动 `ffmpeg`。

要求：

- 不通过 shell 拼接执行，避免路径和特殊字符问题。
- 使用参数数组传参。
- 同时读取 stdout / stderr。
- 将日志通过 Tauri event 推送给前端。
- 保存当前任务的子进程句柄，用于取消任务。

示例逻辑：

```text
Command::new(ffmpeg_path)
  .args([...])
  .stderr(Stdio::piped())
  .stdout(Stdio::piped())
```

## 10. 跨平台注意事项

### 10.1 Windows

- `ffmpeg` 可执行文件通常是 `ffmpeg.exe`。
- 路径可能包含中文、空格、括号。
- NVIDIA 可使用 `h264_nvenc`。
- AMD 可使用 `h264_amf`。
- 不再依赖 `.bat`、`findstr`、`xcopy`、PowerShell。
- **AVS 兼容模式已实现**：检测 AviSynth+ 与 ffmpeg avisynth demuxer，启用时通过内置 DLL 驱动的 AviSynth 脚本渲染字幕。

### 10.2 macOS

- `ffmpeg` 通常来自 Homebrew。
- 常见路径：

```text
/opt/homebrew/bin/ffmpeg
/usr/local/bin/ffmpeg
```

- 硬件编码可考虑 `h264_videotoolbox`。
- 不支持 AviSynth 工作流，统一走 ffmpeg filter 模式（libass 字幕渲染）。
- AVS 开关在 macOS 上自动禁用，CompressForm 与 SettingsView 均有明确提示。

## 11. 第一版 MVP 范围

第一版必须完成：

- Tauri + Vue + TypeScript 项目。
- 主页面文件拖拽。
- `ffmpeg` 自动检测。
- 手动选择 `ffmpeg` 路径。
- `ffmpeg` 版本检测。
- 基础压制参数。
- ASS / SRT 字幕压制。
- **可视化 LOGO 叠加**（✅ 已完成）：在视频抽帧上拖放 / 四角缩放摆放 LOGO，按百分比存储；布局按 (LOGO, 分辨率桶) 维度独立记忆；旋转视频自动修正显示尺寸。
- **AVS 兼容模式**（✅ 已完成）：Windows 上检测 AviSynth+ 与 ffmpeg avisynth demuxer，启用时通过内置 DLL 驱动的脚本渲染字幕。
- 实时日志。
- 取消任务。
- 本地配置保存。
- 应用本体检查更新。

第一版暂不做：

- 内置 `ffmpeg`。
- 自动下载并安装 `ffmpeg`。
- 复杂批量队列。
- 云端账号。
- 插件系统。
- 完整 ASS 语法解析器。

## 12. 开发阶段计划

### 阶段一：核心验证

- 建立 Tauri 项目。
- 实现 `ffmpeg` 检测。
- 实现手动路径选择。
- 实现版本读取。
- 实现一个最小压制任务。

### 阶段二：迁移脚本逻辑

- 迁移配置项。
- 迁移 ASS logo 解析。
- 迁移 yadif、CRF、码率逻辑。
- 生成跨平台 `ffmpeg` 命令。

### 阶段三：完善界面

- 主页面。
- 设置页面。
- 日志面板。
- 错误提示。
- 任务状态。

### 阶段四：应用更新

- 配置 GitHub Pages 静态更新 JSON。
- 设置页检测新版并展示更新说明。
- 检测到新版后引导用户打开 GitHub Releases。
- 测试 Windows 版本检测。

### 阶段五：打包发布

- Windows 安装包。
- macOS `.app` / `.dmg`。
- 发布说明。
- 更新源维护。

## 13. 风险与处理

### 13.1 ffmpeg 不存在

处理：

- 阻止开始压制。
- 引导用户安装或选择路径。

### 13.2 ffmpeg 信息解析不稳定

处理：

- 继续沿用当前脚本思路，通过 `ffmpeg -i` 和压制日志解析媒体信息。
- 对不同 `ffmpeg` 版本的输出格式做兼容。
- 解析失败时允许用户手动确认或跳过相关辅助功能。

### 13.3 用户路径包含特殊字符

处理：

- 后端用参数数组执行命令。
- 不使用 shell 字符串拼接。

### 13.4 macOS 权限与签名

处理：

- 发布前规划签名与公证。
- 避免运行时写入应用安装目录。

### 13.5 更新源不可用

处理：

- 检测失败时显示明确错误。
- GitHub Pages 地址保持稳定。
- Release 安装包 URL 发布后避免变更。

## 14. 最终定位

本应用定位为：

```text
CC字幕压制工作站
```

应用本身不承担视频编码能力，编码能力来自用户本机 `ffmpeg`。

应用核心价值是：

- 把复杂脚本流程图形化。
- 降低压制参数配置成本。
- 保留当前脚本的 ASS logo 解析能力。
- 提供稳定的跨平台任务执行。
- 提供应用本体更新检测能力。
- 清晰管理外部 `ffmpeg` 依赖。
