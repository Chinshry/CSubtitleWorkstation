# GitHub Actions 接入

只有在编辑 `.github/workflows/release.yml` 时使用这份参考。

## 推荐方式

在解析出发布版本之后、创建 draft release 之前，增加更新日志生成步骤。

```yaml
- name: Generate changelog
  id: changelog
  shell: bash
  env:
    VERSION: ${{ steps.meta.outputs.version }}
    TAG: ${{ steps.meta.outputs.tag }}
  run: |
    set -euo pipefail
    python .codex/skills/csubtitle-release/scripts/generate_changelog.py \
      --repo . \
      --version "$VERSION" \
      --to-ref HEAD > RELEASE_CHANGELOG.md
    {
      echo 'body<<CHANGELOG_EOF'
      cat RELEASE_CHANGELOG.md
      echo 'CHANGELOG_EOF'
    } >> "$GITHUB_OUTPUT"
```

然后把 `${{ steps.changelog.outputs.body }}` 传给 release 创建脚本，并用于生成 `docs/updates/latest.json` 的 `notes`。

## 本项目注意事项

- 当前 workflow 会创建或复用 draft release，构建 Windows 和 macOS 产物，然后发布 draft。
- Release body 要保留安装说明，把生成的更新日志追加到安装说明下面。
- `docs/updates/latest.json` 要比完整 GitHub Release body 更短。通常取前 3-6 条，或手工压缩成短摘要。
- `workflow_dispatch` 场景下当前 tag 通常还不存在，所以从上一个 tag 生成到 `HEAD`。
- tag push 发版时，tag 已经存在，可以从上一个 tag 生成到 `${{ github.ref_name }}`。

## 日志确认后的构建

用户确认更新日志后，继续触发 GitHub Actions 构建。触发前再次展示这些参数：

- workflow：`Release`
- version：目标版本号，例如 `0.1.5`
- overwrite：默认 `false`
- notes：写入应用更新提示的精简摘要

触发命令示例：

```powershell
gh workflow run Release --field version=0.1.5 --field overwrite=false --field notes="本次更新摘要"
```

触发后检查运行状态：

```powershell
gh run list --workflow Release --limit 3
```

如果仓库 workflow 名称或输入字段变化，以 `.github/workflows/release.yml` 为准。
