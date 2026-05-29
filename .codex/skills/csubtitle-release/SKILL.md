---
name: csubtitle-release
description: 执行 CSubtitleWorkstation 项目发版流程：同步远程和 tag、确定目标版本、根据上一个版本生成中文更新日志、让用户确认日志、更新 GitHub Release notes 与 docs/updates/latest.json notes，并在确认后触发 GitHub Actions 构建。
---

# CSubtitle 发版流程

## 工作流

当发版需要根据“上一个版本以来的变化”生成更新说明时使用此技能。

1. 先检查远程并同步：
   - 运行 `git fetch --tags --prune`，确保远程分支和 tag 是最新的。
   - 检查当前分支与上游分支的关系，例如 `git status --short --branch`。
   - 如果本地落后远程，先同步远程代码再继续；不要基于过期代码生成更新日志。
   - 如果本地有未提交改动，先确认这些改动是否属于本次发版；不要直接覆盖。
2. 编辑前检查当前发布流程：
   - `.github/workflows/release.yml`
   - `docs/updates/latest.json`
   - `package.json`
   - `src-tauri/Cargo.toml`
3. 确定目标版本：
   - 先找上一个版本 tag，例如 `v0.1.4`。
   - 默认建议版本为上一个版本号 patch +1，例如 `0.1.4` -> `0.1.5`。
   - 让用户确认默认版本，或允许用户输入自定义版本。
   - tag 使用 `vX.Y.Z` 格式。
4. 用户确认版本后，用脚本生成“上一个 tag 到目标 ref”的更新日志：

   ```powershell
   python .codex/skills/csubtitle-release/scripts/generate_changelog.py --repo . --version 0.1.5
   ```

5. 如果用户未指定版本，也可以让脚本按上一个 tag 自动推导 patch +1：

   ```powershell
   python .codex/skills/csubtitle-release/scripts/generate_changelog.py --repo .
   ```

6. 如果目标 tag 已经存在，显式传入目标 tag：

   ```powershell
   python .codex/skills/csubtitle-release/scripts/generate_changelog.py --repo . --version 0.1.5 --to-ref v0.1.5
   ```

7. 生成更新日志后，先展示给用户确认：
   - 展示完整 Markdown 更新日志。
   - 如果要写入 `docs/updates/latest.json`，同时给出拟写入 `notes` 的精简摘要。
   - 用户确认前，不要写入 GitHub Release、`docs/updates/latest.json`、release workflow，也不要触发构建。
8. 用户确认后，把生成的 Markdown 作为 GitHub Release body 的更新日志部分。`docs/updates/latest.json` 的 `notes` 只放用户确认过的精简摘要，因为应用内更新提示需要保持短小。
9. 用户确认更新日志后，继续进入构建流程：
   - 默认构建方式是触发 GitHub Actions 的 `Release` workflow。
   - 触发前展示将使用的版本号、`overwrite` 值、更新日志摘要和 workflow 名称，让用户确认。
   - 用户确认构建后，使用 `gh workflow run Release --field version=<version> --field overwrite=false --field notes=<notes>` 或仓库实际 workflow 要求的等价命令触发。
   - 如果用户明确要求本地构建，则运行本地构建命令，例如 `npm run tauri build`。
   - 构建触发后检查 run 状态，并把 workflow/run 链接或关键状态反馈给用户。
10. 如果要改 GitHub Actions 自动化，读取 `references/github-actions.md`，再把脚本或 GitHub 生成的 release notes 接入现有 release workflow。

## 项目规则

- 以 Git tag 作为版本边界。优先使用 `previous tag..target ref`，不要用文件时间推断。
- 生成前必须先同步远程分支和 tag，避免漏掉别人已经推送的提交或版本 tag。
- 更新日志属于发布内容，必须先让用户确认文本，再写入任何发布文件、创建/更新 GitHub Release 或触发构建。
- 构建属于有副作用操作。即使流程要求“日志后面跟着构建”，也必须先展示构建参数并获得用户确认。
- 更新日志面向用户。提交信息太噪时，要改写成简洁的中文条目。
- 排除纯发布提交，例如 `chore: release vX.Y.Z`。
- 安装包、更新检查、ffmpeg/AVS 行为、打包发布相关变化要写清楚。
- 不要覆盖用户已有改动。工作区不干净时，先读 diff 再编辑发版文件。

## 脚本

`scripts/generate_changelog.py` 是确定性的工具脚本，可以直接运行，不需要额外上下文。它会：

- 省略 `--from-tag` 时自动发现上一个版本 tag；
- 省略 `--version` 时按上一个版本 tag 自动推导 patch +1；
- 通过 `git log` 读取提交；
- 按 conventional commit 类型分组成中文章节；
- 跳过纯发布提交；
- 默认输出 Markdown，传 `--json` 时输出结构化 JSON。

用 `--help` 查看所有参数。

## GitHub Actions

接入 CI 时，保持一份更新日志来源：

- 完整 GitHub Release body：安装/下载说明 + 生成的更新日志。
- 应用更新 manifest：从同一份更新日志提取短摘要。

只有在修改 `.github/workflows/release.yml` 时才读取 `references/github-actions.md`。
