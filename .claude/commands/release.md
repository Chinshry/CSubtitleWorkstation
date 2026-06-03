---
description: 准备 CSubtitleWorkstation 发布流程，生成发布说明并在确认后触发 GitHub Actions Release workflow。
argument-hint: "[version] [--overwrite]"
---

请使用项目级 release skill 执行发布流程。完整流程和脚本位于共享目录：

```text
.agents/release/RELEASE.md
.agents/release/scripts/generate_changelog.py
```

本次命令参数：

```text
$ARGUMENTS
```

关键要求：

- 在用户确认最终发布计划和发布说明前，不要触发 workflow、创建 Release、修改 tag 或发布文件。
- 发布说明使用中文 Markdown，面向用户的分类标题保留 emoji。
- 默认不包含 `文档与流程` 或 `内部维护`，除非用户明确要求。
- `--overwrite` 会删除现有同名 Release/tag，并用当前 `HEAD` 重建同一版本；必须在计划中明确说明。

需要读取并遵循 `.agents/release/RELEASE.md` 中的完整流程。
