# 应用更新与发布流程

本项目只在应用内做“检查新版本并引导用户到 GitHub Releases 下载”，不在应用内自动下载、覆盖或静默安装。

应用更新清单地址固定为：

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
Settings -> Pages -> Deploy from a branch -> master -> /docs
```

发布前确认 `gh auth status` 正常，且当前分支已推送到 GitHub。

## 推荐发布方式

仓库已接入 `.github/workflows/release.yml`。推荐通过 GitHub Actions 手动触发 `Release` 工作流发布正式版本。

在 GitHub Actions 页面选择 `Release` -> `Run workflow`，填写：

| 输入 | 说明 |
| --- | --- |
| `version` | 目标版本号，例如 `0.1.6`，不需要 `v` 前缀 |
| `overwrite` | `true` 时删除同名 Release/tag 后重建，默认 `false` |
| `notes` | 已确认的 Markdown 更新说明；GitHub Release 保留 Markdown，应用内更新清单会自动写入纯文本版本 |

工作流会自动完成：

1. 校验版本号并解析目标 tag，例如 `0.1.6` -> `v0.1.6`。
2. 生成提交范围内的 changelog 原始材料。
3. 将 `package.json`、`src-tauri/Cargo.toml` 和 `docs/updates/latest.json` 更新到目标版本。
4. 自动提交 `chore: release vX.Y.Z` 并推送回当前分支。
5. 创建或更新 draft GitHub Release。
6. 在 Windows runner 构建 NSIS 安装包，在 macOS runner 构建 Universal DMG。
7. 将产物上传到 draft release。
8. 发布 release。
9. 读取 GitHub Release 的 `publishedAt`，回写到 `docs/updates/latest.json` 的 `pub_date`，再提交 `chore: sync release publish date`。

发布完成后，GitHub Pages 会把新的 `docs/updates/latest.json` 暴露给旧版本客户端。

## AI 助手发布入口

仓库内保留了项目级 Codex skill：

```text
.codex/skills/release/SKILL.md
```

在 Codex 中使用 `$release [version] [--overwrite]` 准备发布材料、生成 changelog、整理面向用户的中文更新说明，并在用户确认后触发 `Release` workflow。

仓库也保留了项目级 Claude Code slash command：

```text
.claude/commands/release.md
```

在 Claude Code 中使用 `/release [version] [--overwrite]`。完整流程不写在 command 里，而是由 command 指向共享 `.agents/release/RELEASE.md`。

完整发布流程和共享脚本位于：

```text
.agents/release/RELEASE.md
.agents/release/scripts/generate_changelog.py
```

更新说明必须先由用户确认，再作为 `notes` 传给工作流；工作流会为应用内更新清单生成去除 Markdown 标记的纯文本 notes。

## Tag 推送发布

也可以推送 tag 触发发布：

```bash
git tag v0.1.6
git push origin v0.1.6
```

这种方式适合版本文件和 `docs/updates/latest.json` 已经提前准备好的情况。tag push 不会自动修改并提交 `package.json`、`src-tauri/Cargo.toml` 或更新清单，因此普通发版优先使用手动触发 workflow。

## overwrite=true 的使用场景

`overwrite=true` 会删除远端同名 Release/tag 并重建。只在这些场景使用：

- 刚发布的版本发现严重问题，需要用同一版本号重新构建。
- 调试发布流水线，明确需要覆盖同一个 tag。

默认保持 `overwrite=false`，避免误覆盖已经公开的版本。

## 应用内更新展示

应用前端从 `package.json` 读取当前版本，用 `docs/updates/latest.json` 中的 `version` 比较是否有新版本。

`latest.json` 的核心字段：

```json
{
  "version": "0.1.6",
  "notes": "本次更新说明。",
  "pub_date": "2026-06-01T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "url": "https://github.com/Chinshry/CSubtitleWorkstation/releases/download/v0.1.6/CSubtitleWorkstation_0.1.6_x64-setup.exe"
    },
    "darwin-universal": {
      "url": "https://github.com/Chinshry/CSubtitleWorkstation/releases/download/v0.1.6/CSubtitleWorkstation_0.1.6_universal.dmg"
    }
  }
}
```

设置页会显示当前版本、最新版本、发布时间和完整更新说明；发现新版本后引导用户打开对应的 GitHub Release 页面下载。

## 发布后验证

发布完成后至少验证：

1. GitHub Release 已发布，Windows `*-setup.exe` 和 macOS `*_universal.dmg` 均已上传。
2. `docs/updates/latest.json` 中的 `version`、`notes`、`pub_date` 和下载 URL 与 release 一致。
3. GitHub Pages 上的 `https://chinshry.github.io/CSubtitleWorkstation/updates/latest.json` 可以访问。
4. 旧版本点击“检查应用更新”能检测到新版本，并能打开对应 Release 页面。
5. manifest 格式错误或网络失败时，设置页能显示明确错误提示。

## 本地手工发布备用方案

只有无法使用 GitHub Actions 时才走本地备用方案：

1. 手动更新：

   ```text
   package.json
   src-tauri/Cargo.toml
   docs/updates/latest.json
   ```

2. 本地构建安装包：

   ```powershell
   npm run tauri build
   ```

   macOS 需要指定 `--target universal-apple-darwin` 才能产出 Universal DMG。

3. 在 GitHub Releases 手动创建 `vX.Y.Z` 并上传安装包。
4. 提交并推送 `docs/updates/latest.json`，等待 GitHub Pages 生效。
