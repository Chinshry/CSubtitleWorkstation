# 用户数据与缓存

本文整理 CC 字幕压制工作站在用户使用过程中会写入的本地数据、缓存和临时文件。路径中的应用目录由 Tauri 根据应用标识 `com.cbash.csubtitleworkstation` 生成。

应用使用单实例运行。重复打开应用时不会启动第二个进程，而是唤起已有窗口；因此异常残留的运行期临时缓存会在下一次真正启动应用时统一清理。

## Windows

### 用户配置

目录：

```text
C:\Users\<用户名>\AppData\Roaming\com.cbash.csubtitleworkstation\
```

常见文件：

- `config.json`：应用配置。包含 ffmpeg 路径、默认压制参数、编码预设、输出命名模板、最近 LOGO、LOGO 布局、更新检查设置等。
- `.window-state.json`：窗口位置、大小、最大化状态。

删除影响：

- 删除 `config.json` 会恢复应用默认配置。
- 删除 `.window-state.json` 只会重置窗口状态。

### WebView 缓存与调试开关

目录：

```text
C:\Users\<用户名>\AppData\Local\com.cbash.csubtitleworkstation\EBWebView\
```

内容：

- Edge WebView2 页面缓存、Cookie、Local Storage、Session Storage 和内部数据库。
- 开发/调试用 localStorage 开关，例如模拟 ffmpeg 缺失、ffprobe 缺失、subtitles/libass 缺失、AviSynth 缺失、平台覆盖等。

删除影响：

- 清空后 WebView 会重新生成这些文件。
- 调试开关会恢复默认关闭。
- 如果应用正在运行，部分文件会被 WebView 占用，需要先退出应用再删除。

### 压制临时字幕

目录：

```text
C:\Users\<用户名>\AppData\Local\com.cbash.csubtitleworkstation\filter-temp\
```

用途：

- 每次压制任务会在 `filter-temp\<job-id>\` 下临时复制字幕，例如 `subtitle.ass`。
- 这样做是为了把字幕转到 ASCII 临时路径，避免 ffmpeg subtitles filter、AVS/VSFilterMod 在中文路径或特殊字符路径下失败。

清理时机：

- 应用启动时会尝试清理整个 `filter-temp`。
- 正常压制结束、失败或取消后会尝试删除对应任务目录。
- 如果应用或 ffmpeg 被强制结束，可能残留，可以在应用退出后手动删除整个 `filter-temp`。

### LOGO 编辑器抽帧缓存

目录：

```text
C:\Users\<用户名>\AppData\Local\com.cbash.csubtitleworkstation\logo-editor-frames\
```

用途：

- LOGO 编辑器从当前视频抽取预览帧，文件名类似 `frame-0.png`、`frame-1.png`。

清理时机：

- 应用启动时会尝试清理该目录。
- 打开或关闭 LOGO 编辑器时会清理旧帧。
- 异常退出后可能残留，可以手动删除该目录。

### AVS 临时脚本

目录：

```text
C:\Users\<用户名>\AppData\Local\com.cbash.csubtitleworkstation\avs-temp\
```

用途：

- AVS 模式会写入 `input.avs`，供 ffmpeg 通过 AviSynth 输入读取。
- 文件会被后续 AVS 任务覆盖。

清理建议：

- 应用启动时会尝试清理整个 `avs-temp`。
- 该文件不包含应用配置，应用退出后可以手动删除。

## macOS

Tauri 会把同类数据放到 macOS 的用户 Library 目录下。常见位置如下：

```text
~/Library/Application Support/com.cbash.csubtitleworkstation/
~/Library/Caches/com.cbash.csubtitleworkstation/
~/Library/WebKit/com.cbash.csubtitleworkstation/
```

实际目录会随 Tauri/WebView 运行时略有差异。含义与 Windows 对应：

- Application Support：用户配置、窗口状态。
- Caches/WebKit：WebView 缓存、Local Storage、运行时数据。
- 应用本地数据目录：压制临时字幕、LOGO 抽帧、AVS 相关临时文件。

## 开发模式额外文件

如果用仓库脚本启动开发环境，可能会出现这些项目内文件：

```text
E:\Project\CSubtitleWorkstation\.tauri-dev.out.log
E:\Project\CSubtitleWorkstation\.tauri-dev.err.log
```

这是开发启动日志，不是用户配置。可以删除。

以下目录是开发依赖或构建产物，不属于用户缓存：

```text
node_modules\
dist\
src-tauri\target\
```

## 重置建议

只想恢复应用配置：

1. 退出应用。
2. 删除 Roaming/Application Support 下的 `config.json`。
3. 重新打开应用。

想完全清空用户侧运行数据：

1. 退出应用，确认没有 `csubtitle-workstation` 或相关 WebView2 进程占用。
2. 删除应用的 Roaming/Application Support 配置目录。
3. 删除应用的 Local/Caches/WebView 数据目录。
4. 重新打开应用，目录会自动重建。
