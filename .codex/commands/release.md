---
description: 准备并执行 CSubtitleWorkstation 发版流程：生成更新日志、等待确认并触发 GitHub Actions。
argument-hint: [version] [--overwrite]
---

# 发布 CSubtitleWorkstation

此命令用于准备并执行 CSubtitleWorkstation 的发版流程。

本次命令参数：

```text
$ARGUMENTS
```

## 规则

- 在用户审阅并确认生成的更新日志与发版参数前，不要创建或更新 GitHub Release、不要更新发布文件、不要推送 tag、不要触发构建。
- 不要覆盖无关的用户改动。如果工作区不干净，先检查改动，并说明哪些文件与本次发版无关。
- 使用 Git tag 作为发版边界。优先使用 `previous tag..target ref`，不要根据文件时间推断发布内容。
- 更新日志脚本只用于生成原始素材，不要把机械生成稿直接当最终文案。
- 最终更新文案必须由 Codex 根据原始素材重新组织成用户可读的中文说明，并在用户确认后使用。
- 最终更新文案按中文分类标题组织；只保留本次实际包含内容的分类。常用标题为：
  - `## ⚠️ 重大变更`
  - `## ✨ 新增功能`
  - `## 🐞 问题修复`
  - `## ⚡ 性能优化`
  - `## 🎨 体验改进`
  - `## 📚 文档与流程`
  - `## 🧰 内部维护`
- GitHub Release 使用确认后的 Markdown 文案；应用内 `latest.json.notes` 由 workflow 自动转换成纯文本，避免旧客户端原样显示 `##`、`-` 等 Markdown 标记。
- 仓库的发布工作流是 `.github/workflows/release.yml`；以它的 workflow inputs 为准。

## 流程

1. 同步发布元数据：
   - 运行 `git fetch --tags --prune`。
   - 检查 `git status --short --branch`。
   - 读取 `.github/workflows/release.yml`、`docs/updates/latest.json`、`package.json` 和 `src-tauri/Cargo.toml`。

2. 确定目标版本：
   - 如果 `$ARGUMENTS` 包含语义化版本号，使用该版本。
   - 用户传入的版本号可以带 `v`，也可以不带 `v`；内部统一把版本号规范化为 `X.Y.Z`。
   - 否则根据最新的 `vX.Y.Z` tag 推断下一个 patch 版本。
   - tag 格式始终使用 `vX.Y.Z`；如果用户只传 `X.Y.Z`，自动补成 `vX.Y.Z`。
   - 如果 `$ARGUMENTS` 包含 `--overwrite`，计划使用 `overwrite=true`；否则使用 `overwrite=false`。

3. 生成原始更新素材：
   - 优先使用仓库内的确定性脚本：
     ```powershell
     python .codex/scripts/generate_changelog.py --repo . --version <version>
     ```
   - 如果目标 tag 已存在，并且要把它作为比较终点，追加：
     ```powershell
     --to-ref v<version>
     ```
   - 这份输出只是原始素材：用于确认提交范围和避免漏项。
   - 根据原始素材重写一份面向用户的中文 Markdown 更新说明。可以合并相近条目、改写技术性表达，但不要编造脚本素材之外的功能或修复。
   - 如果本次只有修复内容，使用 `## 🐞 问题修复` 作为顶层标题；如果包含多个分类，按 `重大变更`、`新增功能`、`问题修复`、`性能优化`、`体验改进`、`文档与流程`、`内部维护` 的顺序组织，且只展示有内容的分类。

4. 在产生副作用前展示发版计划：
   - 目标版本。
   - 目标 tag。
   - overwrite 模式。
   - 原始脚本素材。
   - 准备写入 GitHub Release 的最终 Markdown 更新说明；workflow 会派生纯文本版本写入 `docs/updates/latest.json` `notes`。
   - 即将执行的 GitHub Actions workflow 命令。

5. 用户明确确认后，触发发布工作流：
   ```powershell
   gh workflow run Release --field version=<version> --field overwrite=<true|false> --field notes=<confirmed release notes>
   ```

6. 触发后：
   - 获取 `Release` workflow 的最新运行记录。
   - 报告 run 状态和 URL。
   - 说明 workflow 会发布 draft release，并用 GitHub Release 的 `publishedAt` 回写 `docs/updates/latest.json` 的 `pub_date`。

## 本地构建替代方案

只有在用户明确要求本地构建时才本地构建。此时运行项目现有的本地构建命令，并报告产物路径。
