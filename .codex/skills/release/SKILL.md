---
name: release
description: 准备并执行 CSubtitleWorkstation 发布流程。用户要求发布新版本、覆盖重建 Release、触发 GitHub Actions Release workflow、生成发布说明、整理 changelog，或在本仓库使用 `$release` 时使用。
---

# CSubtitleWorkstation 发布流程

用于准备并执行本项目的 GitHub Actions `Release` 发布工作流。

完整流程和脚本在共享目录：

```text
.agents/release/RELEASE.md
.agents/release/scripts/generate_changelog.py
```

使用本 skill 时，先读取 `.agents/release/RELEASE.md` 并严格遵循其中的流程。用户确认最终发布计划和发布说明前，不要触发 workflow、创建 Release、修改 tag 或发布文件。
