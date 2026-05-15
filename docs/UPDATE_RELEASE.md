# 应用更新发布流程

本项目只做“检测新版并引导用户到 GitHub Releases 下载”，不在应用内自动下载或覆盖安装。

更新检查地址固定为：

```text
https://chinshry.github.io/CSubtitleWorkstation/updates/latest.json
```

该地址由 GitHub Pages 托管，对应仓库内文件：

```text
docs/updates/latest.json
```

## 一次性准备

在 GitHub 仓库启用 Pages：

```text
Settings -> Pages -> Deploy from a branch -> main -> /docs
```

## 发布新版

1. 提升版本号，例如 `0.1.0 -> 0.1.1`：

```text
package.json
src-tauri/Cargo.toml
```

2. 构建安装包：

```powershell
npm run tauri build
```

3. 在 GitHub Releases 创建新 release，例如：

```text
v0.1.1
```

上传 Windows 安装包。

4. 更新 `docs/updates/latest.json`：

```json
{
  "version": "0.1.1",
  "notes": "本次更新说明。",
  "pub_date": "2026-05-15T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "url": "https://github.com/Chinshry/CSubtitleWorkstation/releases/download/v0.1.1/CSubtitleWorkstation_0.1.1_x64-setup.exe"
    }
  }
}
```

5. 提交并 push `docs/updates/latest.json`。

6. 用旧版本点击“检查应用更新”，验证：

- 能检测到新版本。
- 能显示更新说明。
- 点击“前往 GitHub 下载”后能打开对应 Release 页面。
- manifest 格式错误或网络失败时能显示明确错误。
