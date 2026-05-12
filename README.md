# CC字幕压制工作站（CSubtitleWorkstation）

Tauri 2 + Vue 3 + TypeScript 桌面应用，用于替代原有字幕压制 BAT 工作流。把拖入的视频和字幕一键交给本机 `ffmpeg` 压制，并提供实时日志、进度、视频信息解析与可视化的参数配置界面。

- 项目名：`CSubtitleWorkstation`
- 仓库名：`CSubtitleWorkstation`
- 中文名：`CC字幕压制工作站`

## 功能概览

### 压制流程

- **拖拽导入**：把视频和 ASS/SRT/VTT/SUB 等字幕直接拖到窗口里，自动按扩展名分发到"视频路径"与"字幕路径"输入框。
- **输出路径自动联动**：视频路径变化时自动生成同目录下 `{原名} output.mp4`；用户手改过的输出路径不会被覆盖。
- **可视化参数**：CRF、最大码率（不限制 / 自动 = 视频原码率 + 1000 Kbps / 自定义 Kbps）、编码器（`libx264` / `h264_nvenc` / `h264_amf` / `h264_videotoolbox`）。
- **ASS LOGO 解析**：勾选后会读取 ASS 文件中的 LOGO 行（约定格式），按视频分辨率自适应缩放并叠加到视频上。
- **yadif 反交错**：可选开关，处理交错素材。
- **AVS 兼容模式**（仅 Windows）：勾选后使用 AviSynth+ 脚本引擎渲染字幕，相比 ffmpeg 的 libass 对复杂 ASS 特效（如 VSFilterMod 扩展标签）支持更完善。需要系统已安装 AviSynth+ 且 ffmpeg 启用了 `--enable-avisynth` 构建。
- **命令预览**：开始压制前可一键预览即将执行的完整 ffmpeg 参数。
- **实时进度**：进度条、当前时间 / 总时长、输出大小、速度、fps、码率、原始 status 行。
- **完整日志**：ffmpeg 的 stdout / stderr 全程透出；按 `\r` 与 `\n` 两种分隔符切行，能看到 ffmpeg 不断刷新的进度行。
- **取消任务**：压制过程中可随时取消，进程会被结束。

### 视频信息卡

拖入视频后自动调用 ffprobe 解析并显示：

- **文件**：大小、时长（起始时间不为 0 时会以括号标出）、容器格式（带友好映射，例如 `mov,mp4,m4a,3gp,3g2,mj2` → `MP4 / MOV`，鼠标悬停可看原始 demuxer 名）。
- **视频**：分辨率（含 DAR / SAR）、编码器与 Profile、像素格式、码率、帧率、**帧率模式（CFR / VFR）**、**总帧数**、色域、色范围。
- **音频**：编码器与 Profile、采样率、声道布局、码率。

每个字段都带中文小标签 + hover 含义说明。ffprobe 不可用时自动回退到 ffmpeg `-i` 文本解析（精度较低，无 CFR/VFR / 总帧数）。

### 设置页

- **ffmpeg 配置**：检测系统 PATH 上的 `ffmpeg`，或手动选择可执行文件路径。状态、来源、路径、版本一目了然。
- **AVS 设置**（仅 Windows）：显示 ffmpeg avisynth demuxer 与 AviSynth+ 的安装状态；缺失时给出安装手册引导。
- **应用更新**：调用 Tauri Updater 检查应用本身是否有新版（与 ffmpeg 版本检测彼此独立）。
- **默认 LOGO 目录**：在压制页填的 `logo 检测目录` 会自动保存为下次启动的默认值。

## 环境要求

- Node.js + npm
- Rust / Cargo（Tauri 后端）
- Tauri 桌面依赖（Windows: WebView2 / Visual C++ Build Tools；macOS: Xcode Command Line Tools）
- 本机已安装 `ffmpeg`，或在设置页选择 `ffmpeg` 可执行文件

Rust 安装（Windows，PowerShell）：

```powershell
winget install Rustlang.Rustup
```

安装后重开终端验证：

```powershell
rustc --version
cargo --version
```

## 开发命令

```bash
npm install
npm run tauri dev
```

打包：

```bash
npm run tauri build
```

## 项目结构

```
CSubtitleWorkstation/
├── src/                    # 前端 Vue 3 + TypeScript
│   ├── App.vue             # 应用外壳 + 全局 Tauri 拖拽监听
│   ├── views/
│   │   ├── HomeView.vue    # 压制主页：表单 + 视频信息卡 + 日志
│   │   └── SettingsView.vue
│   ├── components/
│   │   ├── CompressForm.vue
│   │   ├── VideoMetaCard.vue
│   │   ├── JobLogPanel.vue
│   │   └── FfmpegStatus.vue
│   ├── api/                # 调用 Tauri 后端命令的薄封装
│   ├── stores/             # 全局共享状态（拖拽载荷、ffmpeg/AVS 检测结果等）
│   └── types.ts            # 与后端共享的类型定义
└── src-tauri/              # Rust 后端
    ├── resources/avs/      # 内置 AVS 插件 DLL（VSFilterMod、LSMASHSource）
    └── src/
        ├── commands/       # #[tauri::command]
        │   ├── compress.rs # 启动 / 取消压制
        │   ├── video.rs    # 解析视频信息
        │   ├── ffmpeg.rs   # ffmpeg 检测与路径设置
        │   ├── avs.rs      # AVS 环境检测
        │   ├── config.rs   # 应用配置读写
        │   └── updater.rs  # 应用更新
        └── services/
            ├── command_builder.rs # 构建 ffmpeg 参数（支持 AVS 模式）
            ├── avs_detector.rs    # 检测 AviSynth+ 与 ffmpeg avisynth demuxer
            ├── avs_workspace.rs   # 管理 AVS 工作目录与脚本生成
            ├── video_meta.rs      # 解析 ffmpeg -i 输出
            ├── ass_logo.rs        # 解析 ASS 中的 LOGO 行
            ├── ffmpeg_locator.rs
            └── config_store.rs
```

## 设计原则

- 不内置 ffmpeg，由用户自行安装或指定。
- 推荐使用 Gyan.dev 的 `ffmpeg-release-full.7z`，因为其中带 `ffprobe` 与 AviSynth+ 支持；本工具的视频信息卡（CFR/VFR、总帧数等）依赖 ffprobe，AVS 兼容模式依赖 AviSynth+。
- 选择 ffmpeg 可执行文件时，本工具会**自动在同目录寻找 ffprobe**，无需单独配置。
- 应用更新和 ffmpeg 版本检测彼此独立。
- **AVS 兼容模式**（仅 Windows）：需要系统已安装 AviSynth+ 且 ffmpeg 启用了 `--enable-avisynth` 构建。启用时，ffmpeg 输入改为内置 DLL 驱动的 AviSynth 脚本，字幕由 VSFilterMod 的 TextSubMod 渲染，LOGO overlay 与 yadif 仍然有效。
- macOS / Linux 不支持 AVS 压制，统一走 ffmpeg filter 模式（libass 字幕渲染）。
- 直接通过 Rust `std::process::Command` 调用 ffmpeg / ffprobe，**不经过 shell**——因此文件名中包含 `!`、空格、Unicode 引号等特殊字符也无需像 BAT 那样切换 delayed expansion。

## 路线图

详见 [TODO.md](./TODO.md)：Windows MSI / macOS DMG 打包、Tauri Updater 公私钥、可视化 LOGO 叠加等。
