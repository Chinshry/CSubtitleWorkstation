<div align="center">

<img src="src-tauri/icons/icon.png" width="120" alt="CC Subtitle Workstation" />

# CC 字幕压制工作站

**一键压制字幕视频，支持 LOGO 叠加、特效字幕、实时进度反馈**

<p>
  <a href="https://github.com/Chinshry/CSubtitleWorkstation/releases">📥 下载</a> ·
  <a href="docs/REQUIREMENTS.md">📖 需求文档</a> ·
  <a href="https://github.com/Chinshry/CSubtitleWorkstation/issues">🐛 反馈问题</a>
</p>

<p>
  <a href="https://github.com/Chinshry/CSubtitleWorkstation/stargazers"><img src="https://img.shields.io/github/stars/Chinshry/CSubtitleWorkstation?style=flat-square&logo=github&color=yellow" alt="GitHub Stars"></a>
  <a href="https://github.com/Chinshry/CSubtitleWorkstation/releases"><img src="https://img.shields.io/github/v/release/Chinshry/CSubtitleWorkstation?include_prereleases&style=flat-square&logo=github&color=blue" alt="Release"></a>
  <a href="https://github.com/Chinshry/CSubtitleWorkstation/releases"><img src="https://img.shields.io/github/downloads/Chinshry/CSubtitleWorkstation/total?style=flat-square&logo=github&color=orange" alt="Downloads"></a>
  <a href="https://github.com/Chinshry/CSubtitleWorkstation/blob/master/LICENSE"><img src="https://img.shields.io/github/license/Chinshry/CSubtitleWorkstation?style=flat-square&color=green" alt="License"></a>
</p>

<p>
  <img src="https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square&logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/Vue-3-4FC08D?style=flat-square&logo=vue.js&logoColor=white" alt="Vue">
  <img src="https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Rust-1-CE422B?style=flat-square&logo=rust&logoColor=white" alt="Rust">
</p>

用可视化界面替代传统字幕压制工作流。拖拽导入 → 自动解析 → 可视化配置 → 实时反馈 → 一键压制。

</div>

---

## 💡 Why?

传统字幕压制工作流依赖**小丸工具箱**等第三方 GUI 工具或 BAT 脚本，存在以下痛点：

- **参数繁琐**：ffmpeg 命令行参数众多，易出错，难以复用
- **反馈缺失**：工具窗口闪现即关，看不到实时进度和日志
- **跨平台困难**：小丸工具箱仅 Windows，macOS/Linux 用户无法使用
- **视频信息不透明**：需要手动用 ffprobe 查询分辨率、帧率、编码器等
- **AVS 压制困难**：需要手动编写 AviSynth 脚本来处理特效字幕（如复杂特效、矢量绘制等），参数复杂，调试困难，且耗时；普通用户也很难判断一份 ASS 到底"普通"还是"带特效"，常常压完才发现标签没渲染
- **LOGO 添加困难**：传统方式需要用 ASS 字幕的 img 标签或矢量绘制命令，这些都需要特效字幕压制才能实现，流程复杂且耗时。本应用采用 ffmpeg overlay filter 直接在视频上叠加 LOGO，不需要特效字幕压制，更高效

**CC 字幕压制工作站** 用可视化界面解决这些问题：拖拽导入 → 自动解析视频 → 可视化配置参数 → 实时进度反馈 → 一键压制。**支持 Windows 和 macOS**，开箱即用。

---

## ✨ Features

- 🎬 **拖拽导入** — 把视频和字幕直接拖到窗口，自动按扩展名分发
- 📊 **视频信息卡** — 自动调用 ffprobe 解析分辨率、帧率、编码器、时长等，支持 CFR/VFR 识别
- 🎨 **可视化参数** — CRF、码率、编码器（x264/x265/NVENC/AMF/VideoToolbox）一目了然
- 🧰 **编码预设管理** — 内置 `x264 平衡` / `x265 体积优先` / `NVENC 快速` / `AMF 快速` / `Apple 快速` 等多套预设，可在「预设」页新增/编辑/导入导出，主页直接切换
- 📝 **输出文件名模板** — 支持 `{video_name}` / `{resolution}` / `{encoder}` / `{crf}` / `{date:YYYYMMDD}` 等变量，默认生成 `{video_name} 中字.mp4`；可保存多套模板并设为默认
- 🖼️ **LOGO 可视化编辑 + 布局保存** — 在视频抽帧上拖放 LOGO，支持四角缩放、画布快捷键缩放、手动输入缩放比例和失焦自动隐藏调整锚点；为不同分辨率（720p/1080p/4K）和屏幕方向（横/竖屏）各自保存一套 LOGO 位置，下次打开自动恢复
- 🎞️ **反交错处理** — yadif 可选开关，处理交错素材
- 🔤 **特效字幕压制**（仅 Windows）— 使用 AviSynth+ 脚本引擎处理复杂特效字幕（如矢量绘制、img 标签等），相比 ffmpeg libass 支持更完善
- 🧠 **特效标签自动识别**（仅 Windows）— 选定字幕后自动扫描 VSFilterMod 扩展标签（`\fsc` / `\xblur` / `\jitter` / `\distort` / `\img` 等），命中即自动勾选「AVS 压制」并在界面提示具体匹配到的标签，省去人工判断特效字幕的步骤
- 🔍 **字幕检查面板** — 集中展示字幕风险：图片路径缺失 / 字体未安装 / 样式缺失作为错误，ASS 色彩矩阵（YCbCr Matrix）问题作为警告，VSFilterMod 特效标签作为建议，并给出处理方式
- 👁️ **命令预览** — 开始压制前预览完整 ffmpeg 参数，支持折叠/复制
- 📈 **实时进度** — 进度条、当前时间/总时长、输出大小、速度、fps、码率、原始 status 行
- 📝 **完整日志** — ffmpeg stdout/stderr 全程透出，按 `\r` 与 `\n` 两种分隔符切行
- ⏹️ **取消任务** — 压制过程中随时取消，进程优雅退出，已编码片段保留可播放
- 🔔 **应用更新检查** — 启动时静默检查（可关闭）或在设置页手动触发，发现新版通过 Toast 通知并引导前往 GitHub Releases 下载
- 🍎 **macOS 兼容自检** — 启动检测 ffmpeg 是否包含 `subtitles` / `ass` filter，缺失时直接提示安装 `ffmpeg-full`（Homebrew），避免压到一半才失败

---

## 📋 支持格式

| 类型 | 支持格式 |
|------|---------|
| **视频** | MP4, MKV, MOV, TS, M4V, FLV, AVI, WebM, WMV, MPG, MPEG, 3GP, 3G2, RM, RMVB, VOB, MTS, M2TS |
| **字幕** | ASS, SSA, SRT, VTT, SUB |
| **LOGO** | PNG, JPG, JPEG, WebP, BMP |

---

## 🖥️ 支持平台

| 功能 | Windows | macOS |
|------|---------|-------|
| **基础压制** | ✅ | ✅ |
| **视频信息解析** | ✅ | ✅ |
| **LOGO 叠加** | ✅ 可选层级（字幕上/下） | ✅ 仅字幕在上 LOGO 在下 |
| **反交错** | ✅ | ✅ |
| **特效字幕压制** | ✅ | ❌ |
| **subtitles/ass filter 自检** | ✅ | ✅（缺失会提示装 `ffmpeg-full`） |
| **硬件加速** | NVIDIA NVENC、AMD AMF | Apple VideoToolbox |
| **安装包格式** | NSIS 安装程序 | Universal DMG（Intel + Apple Silicon） |

> macOS 版本不支持特效字幕压制（AviSynth+ 仅 Windows），统一使用 ffmpeg libass 字幕渲染。
> macOS 版本 LOGO 层级固定为"字幕在上 LOGO 在下"，不支持切换到"LOGO 在上 字幕在下"。
> macOS 上的精简版 ffmpeg（如 `brew install ffmpeg`）可能未启用 `--enable-libass`，本应用启动会自动检测；如缺失 `subtitles` / `ass` filter，请改装 `brew install ffmpeg-full`。

---

## 📸 界面截图

<table>
  <tr>
    <td width="50%" align="center">
      <a href="docs/screenshots/01-home.png"><img src="docs/screenshots/01-home.png" alt="主页 - 拖拽导入与压制参数" /></a>
      <br/><sub><b>主页</b> · 拖拽视频和字幕即可开始，压制参数一目了然</sub>
    </td>
    <td width="50%" align="center">
      <a href="docs/screenshots/02-encoding.png"><img src="docs/screenshots/02-encoding.png" alt="压制中 - 视频信息、字幕检查、实时进度与日志" /></a>
      <br/><sub><b>压制中</b> · 视频信息卡 + 字幕检查 + 实时进度 + ffmpeg 日志</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <a href="docs/screenshots/03-logo-editor.png"><img src="docs/screenshots/03-logo-editor.png" alt="LOGO 可视化编辑器" /></a>
      <br/><sub><b>LOGO 编辑器</b> · 在视频抽帧上拖放缩放，支持画布缩放与比例输入，按分辨率/方向自动保存布局</sub>
    </td>
    <td width="50%" align="center">
      <a href="docs/screenshots/04-subtitle-check.png"><img src="docs/screenshots/04-subtitle-check.png" alt="字幕检查面板" /></a>
      <br/><sub><b>字幕检查</b> · 图片缺失、字体未装、色彩矩阵、特效标签分级提示</sub>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <a href="docs/screenshots/05-presets.png"><img src="docs/screenshots/05-presets.png" alt="预设管理 - 编码预设与文件名模板" /></a>
      <br/><sub><b>预设管理</b> · 编码预设 + 输出文件名模板，可导入导出</sub>
    </td>
    <td width="50%" align="center">
      <a href="docs/screenshots/06-settings.png"><img src="docs/screenshots/06-settings.png" alt="设置面板 - ffmpeg、AVS、应用更新" /></a>
      <br/><sub><b>设置</b> · ffmpeg / AviSynth+ 自检与应用更新检查</sub>
    </td>
  </tr>
</table>

---

## 📥 下载与安装

最新版本请到 [Releases 页面](https://github.com/Chinshry/CSubtitleWorkstation/releases) 下载。

### Windows

下载 `CSubtitleWorkstation_x.y.z_x64-setup.exe`，双击安装即可。

### macOS

下载 `CSubtitleWorkstation_x.y.z_universal.dmg`，**同一个 DMG 同时兼容 Apple Silicon (M1/M2/M3/M4) 与 Intel Mac**，无需区分。

#### macOS首次打开提示「无法验证开发者」怎么办

由于本项目目前没有 Apple 开发者证书签名，macOS Gatekeeper 会在首次打开时拦截：

1. 双击 `.dmg` 挂载磁盘镜像，把应用拖到「应用程序」文件夹
2. 在「应用程序」里**右键点击** CSubtitleWorkstation → 选「打开」
3. 弹出「无法验证开发者」对话框 → 再次点「打开」

如果 macOS 14+ 上述步骤无效（Apple 在新版系统收紧了快捷绕过）：

1. 双击应用让系统弹出拦截提示后关闭
2. 进入「系统设置」→「隐私与安全性」
3. 滑到底部能看到「已阻止使用"CSubtitleWorkstation"」→ 点「仍要打开」→ 输入密码确认

> 此操作每个应用只需做一次，之后双击即可正常打开。

---

## ❓ FAQ

<details>
<summary><strong>ffmpeg 在哪里配置？</strong></summary>

应用启动时会自动检测系统 PATH 上的 `ffmpeg`。如果检测失败或想指定其他版本，进入「设置」页面手动选择 ffmpeg 可执行文件路径。应用会自动在同目录寻找 `ffprobe`。

</details>

<details>
<summary><strong>AVS 模式需要什么环境？</strong></summary>

仅 Windows 支持。需要系统已安装 AviSynth+ 且 ffmpeg 启用了 `--enable-avisynth` 构建。推荐使用 Gyan.dev 的 `ffmpeg-release-full.7z`，其中包含 ffprobe 与 AviSynth+ 支持。

</details>

<details>
<summary><strong>什么样的字幕会被自动识别为「特效字幕」？</strong></summary>

应用只识别 **VSFilterMod 特有的扩展标签**——这些标签不在标准 ASS/SSA 规范中，libass 渲染不出来，必须走 AVS + VSFilterMod 才能正确显示。命中其一即视为特效字幕：

| 类别 | 标签 |
|------|------|
| 缩放 / 模糊 / 偏移 | `\fsc`、`\xblur`、`\yblur`、`\fsvp`、`\fshp` |
| 四角渐变 | `\1vc`–`\4vc`（颜色）、`\1va`–`\4va`（透明度） |
| 抖动 / 变形 | `\jitter`、`\rnd*`、`\distort`、`\frs` |
| 3D / 空间 | `\z`、`\ortho` |
| 特殊移动 | `\mover`、`\moves3/4`、`\movevc` |
| 图片填充 | `\1img`–`\4img` |

> 检测在选定字幕后自动进行，前端实时显示"已检测到 \xxx 等特效，已自动启用 AVS 压制"。如果当前平台不支持 AVS（macOS / 未装 AviSynth+），则只提示但不强制启用。普通 ASS（仅含 `\b` / `\i` / `\c` / `\pos` 等标准标签）继续走 ffmpeg libass，不会被误判。

</details>

<details>
<summary><strong>LOGO 布局怎么保存？</strong></summary>

在压制页 LOGO 编辑器中保存过的布局会按 (LOGO 图, 分辨率桶) 持久化到本地配置。下次打开同样的视频自动恢复。支持 6 个桶：720p/1080p/4K × 横屏/竖屏。

编辑器支持两类缩放：一类是 LOGO 本身的四角拖拽缩放，用于决定最终压制中的 LOGO 尺寸；另一类是预览画布缩放，用于放大细调位置，不影响最终输出尺寸。预览画布可通过快捷键缩放，也可以直接输入缩放比例；当 LOGO 失去焦点时，调整锚点会自动隐藏，避免遮挡预览。

</details>

<details>
<summary><strong>应用会在本地保存哪些配置和缓存？</strong></summary>

应用会在系统用户目录下保存配置、窗口状态、WebView 缓存、压制临时字幕、LOGO 预览帧和 AVS 临时脚本。完整路径和清理建议见 [用户数据与缓存](docs/CACHE_AND_DATA.md)。

</details>

<details>
<summary><strong>压制过程中能取消吗？</strong></summary>

可以。点击「取消」按钮，ffmpeg 进程会收到 SIGINT 信号优雅退出（相当于 Ctrl+C），已编码的部分会被正确写入文件尾，保证输出仍然可播放。

</details>

<details>
<summary><strong>编码预设和输出文件名模板在哪里管理？</strong></summary>

侧边栏「预设」页面集中管理两类资源：

- **编码预设**：内置 `x264 平衡` / `x265 体积优先` / `NVENC 快速` / `AMF 快速` / `Apple 快速` 五套；可新增/编辑/删除自定义预设，支持 `customVideoArgs` 自由扩展 ffmpeg 参数；可导入/导出 JSON 在多机之间同步。
- **输出文件名模板**：支持 `{video_name}` / `{resolution}` / `{encoder}` / `{crf}` / `{date:YYYYMMDD}` / `{date:YYMMDD}` 等变量，可选「与视频同目录」/「固定目录」两种输出目录策略，可保存多套并设为默认。

主页编辑区直接通过下拉框切换当前任务使用的预设和模板。

</details>


---

## 🔧 高级信息

<details>
<summary><strong>开发环境与命令</strong></summary>

### 环境要求

- Node.js + npm
- Rust / Cargo（Tauri 后端）
- Tauri 桌面依赖（Windows: WebView2 / Visual C++ Build Tools；macOS: Xcode Command Line Tools）
- 本机已安装 `ffmpeg`，或在设置页选择 `ffmpeg` 可执行文件

### 开发命令

```bash
npm install
npm run tauri dev      # 开发模式
npm run tauri build    # 打包
npm run build          # 仅前端构建
```

</details>

<details>
<summary><strong>设计原则</strong></summary>

- **不内置 ffmpeg**：由用户自行安装或指定。推荐 Gyan.dev 的 `ffmpeg-release-full.7z`（包含 ffprobe 与 AviSynth+ 支持）
- **自动 ffprobe 定位**：选择 ffmpeg 可执行文件时，应用会自动在同目录寻找 ffprobe，无需单独配置
- **应用更新与 ffmpeg 版本检测独立**：两者互不影响
- **特效字幕压制（仅 Windows）**：需要系统已安装 AviSynth+ 且 ffmpeg 启用了 `--enable-avisynth` 构建
- **macOS / Linux 不支持特效字幕压制**：统一走 ffmpeg filter 模式（libass 字幕渲染）
- **无 shell 调用**：直接通过 Rust `std::process::Command` 调用 ffmpeg / ffprobe，文件名中包含特殊字符无需转义

</details>

---

## 📊 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Chinshry/CSubtitleWorkstation&type=Date)](https://www.star-history.com/#Chinshry/CSubtitleWorkstation&Date)

---

## 🤝 Contributing

Issues 和建议欢迎！提交 PR 前请确保代码通过 linter 和测试。

## 📚 文档

- [需求与技术方案](docs/REQUIREMENTS.md) — 项目设计文档
- [应用更新与发布流程](docs/UPDATE_RELEASE.md) — GitHub Actions 发版、更新清单与验证步骤
- [用户数据与缓存](docs/CACHE_AND_DATA.md) — 本地配置、缓存和临时文件位置
- [待办事项](docs/TODO.md) — 功能路线图与已知问题
