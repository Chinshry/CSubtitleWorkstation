# 应用更新发布流程

本项目只做"检测新版并引导用户到 GitHub Releases 下载"，不在应用内自动下载或覆盖安装。

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

## 推荐：GitHub Actions 一键发版

仓库已接入 `.github/workflows/release.yml`，覆盖 Windows NSIS 与 macOS Universal DMG 的构建。两种触发方式：

### 方式 A：推送 tag 自动发版

```bash
git tag v0.1.1
git push origin v0.1.1
```

流水线会自动：

1. 把 `package.json` 与 `src-tauri/Cargo.toml` 的版本号校准到 `0.1.1`。
2. 在 Windows runner 上 `npm run tauri build` 出 NSIS 安装包。
3. 在 macOS runner 上 `npm run tauri build --target universal-apple-darwin` 出兼容 Intel + Apple Silicon 的 DMG。
4. 创建 GitHub Release `v0.1.1` 并上传安装包。

### 方式 B：手动触发（含一键覆盖发布）

在 GitHub Actions 页面选择 `Release` 工作流 → `Run workflow`，填入：

| 输入 | 说明 |
|------|------|
| `version` | 目标版本号，如 `0.1.1`（无需 `v` 前缀） |
| `overwrite` | `true` 时会先删除已存在的同 tag Release 与远端 tag，再重新发布 |
| `notes` | 写入 GitHub Release Body 的更新说明 |

`overwrite=true` 适用于：

- 刚发布的版本立刻发现严重问题，需要重新构建覆盖同一个 tag。
- 调试发版流程，避免每次都换版本号。

> 默认 `overwrite=false`，意味着同 tag 重复触发会失败，保护既有版本不被误覆盖。

### 发布完成后

GitHub Actions 不会自动更新应用内的 manifest，需要手动同步：

1. 编辑 `docs/updates/latest.json`，把 `version` / `notes` / `pub_date` / `platforms[*].url` 改成新版的值。

   ```json
   {
     "version": "0.1.1",
     "notes": "本次更新说明。",
     "pub_date": "2026-05-17T00:00:00Z",
     "platforms": {
       "windows-x86_64": {
         "url": "https://github.com/Chinshry/CSubtitleWorkstation/releases/download/v0.1.1/CSubtitleWorkstation_0.1.1_x64-setup.exe"
       },
       "darwin-universal": {
         "url": "https://github.com/Chinshry/CSubtitleWorkstation/releases/download/v0.1.1/CSubtitleWorkstation_0.1.1_universal.dmg"
       }
     }
   }
   ```

2. 提交并 push `docs/updates/latest.json`，GitHub Pages 会自动更新。

3. 用旧版本点击"检查应用更新"，验证：

   - 能检测到新版本。
   - 能显示更新说明。
   - 点击"前往 GitHub 下载"后能打开对应 Release 页面。
   - manifest 格式错误或网络失败时能显示明确错误。

## 备用：本地手工发版

无法触发 CI 时可在本机出包：

1. 提升版本号：

   ```text
   package.json
   src-tauri/Cargo.toml
   ```

2. 构建安装包：

   ```powershell
   npm run tauri build
   ```

   macOS 需指定 `--target universal-apple-darwin` 才能出双架构 DMG。

3. 在 GitHub Releases 手动创建 `v0.1.1` 并上传安装包。

4. 同步 `docs/updates/latest.json`（同上）并 push。
