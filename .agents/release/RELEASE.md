# CSubtitleWorkstation 发布流程

这是 Codex 与 Claude Code 共用的发布流程单一真源。平台入口只负责把用户带到这里：

- Codex：`.codex/skills/release/SKILL.md`
- Claude Code：`.claude/skills/release/SKILL.md`
- Claude Code 兼容命令：`.claude/commands/release.md`

## 入口参数

接受 `0.1.7` 或 `v0.1.7` 这样的版本号，以及可选的 `--overwrite`。

- 内部统一把版本号规范化为 `X.Y.Z`。
- tag 统一使用 `vX.Y.Z`。
- 如果用户没有传版本号，根据最新 `vX.Y.Z` tag 推断下一个 patch 版本。
- 只有用户传入 `--overwrite` 或明确要求重建已有 release 时，才使用 `overwrite=true`。

## 约束

- 用户明确确认最终发布计划和发布说明前，不要创建或更新 GitHub Release、tag、版本文件，也不要触发 workflow。
- 不要覆盖无关的用户改动。工作区不干净时，先检查并说明哪些文件与本次发布无关。
- 使用 Git tag 作为发布边界。优先使用 `previous tag..target ref`，不要根据文件时间推断发布内容。
- `.agents/release/scripts/generate_changelog.py` 的输出只作为原始材料。最终说明必须改写为面向用户的中文 Markdown。
- 面向用户的 Release Notes 分类标题保留 emoji。
- 除非用户明确要求技术/内部说明，否则最终 Release Notes 不包含 `文档与流程` 或 `内部维护`。
- GitHub Release 使用 Markdown；workflow 会把说明转换后写入 `docs/updates/latest.json`。

## Changelog 边界

普通新版本发布时，按上一个 tag 到 `HEAD` 生成原始材料。

对已有版本执行 `--overwrite` 时，GitHub workflow 会删除现有 release/tag，并用当前 `HEAD` 重建同一个版本。必须向用户说清楚：

- 被替换的已有版本/tag。
- 当前 `HEAD` 会成为重建后的 release。
- 发布说明材料应按“上一个正式 tag 到 `HEAD`”整理，而不是只看旧目标 tag 到 `HEAD`。

例如已有 `v0.1.6` 时覆盖重建 `0.1.6`：

```powershell
python .agents/release/scripts/generate_changelog.py --repo . --version 0.1.6 --from-tag v0.1.5 --to-ref HEAD
```

## 流程

1. 同步并检查发布上下文：

```powershell
git fetch --tags --prune
git status --short --branch
```

按需读取这些文件：

- `.github/workflows/release.yml`
- `docs/updates/latest.json`
- `package.json`
- `src-tauri/Cargo.toml`

2. 生成原始 changelog 材料。

普通发布：

```powershell
python .agents/release/scripts/generate_changelog.py --repo . --version <version>
```

覆盖发布时，必要时显式指定上一个发布 tag：

```powershell
python .agents/release/scripts/generate_changelog.py --repo . --version <version> --from-tag <previous-tag> --to-ref HEAD
```

3. 改写最终中文 Release Notes。只保留有实际用户可见内容的分类，顺序如下：

- `## ⚠️ 重大变更`
- `## ✨ 新增功能`
- `## 🐞 问题修复`
- `## ⚡ 性能优化`
- `## 🎨 体验改进`

只有用户明确要求时，才加入这些技术/内部分类：

- `## 📚 文档与流程`
- `## 🧰 内部维护`

4. 产生任何副作用前，先向用户展示：

- 目标版本
- 目标 tag
- `overwrite` 值
- 工作区未提交改动摘要
- 覆盖发布时的现有 release/tag 状态
- 原始 changelog 材料摘要
- 最终 Markdown 发布说明
- 即将执行的 workflow 命令

5. 只有用户明确确认后，才触发 workflow。

短单行说明可以这样传：

```powershell
gh workflow run Release --field version=<version> --field overwrite=<true|false> --field notes=<confirmed release notes>
```

正常多行 Markdown 说明应优先通过当前 shell 的变量或文件传入，确保换行完整保留。必要时验证已提交的 workflow inputs。

6. 触发后，报告最新 `Release` workflow run 状态和 URL。说明 workflow 会发布 draft release，并在发布成功后用 GitHub Release 的 `publishedAt` 一次性写入 `docs/updates/latest.json` 的版本、说明、发布时间和下载 URL。

## 本地构建替代方案

只有用户明确要求本地构建时才本地构建。使用项目现有本地构建命令，并报告产物路径。
