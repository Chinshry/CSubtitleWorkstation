# TODO

## 应用更新

- 生成 Tauri updater 密钥对：
  - 示例：`npm run tauri signer generate -- -w $env:USERPROFILE\.tauri\csubtitle-workstation.key`
  - 私钥由生成命令产出，公钥会打印在终端。
- 妥善保存 updater 私钥：
  - 私钥放到仓库之外，例如 `$env:USERPROFILE\.tauri\csubtitle-workstation.key`。
  - 私钥不能提交到 git，不能放前端，不能写进发布包。
  - 建议放入密码管理器或离线备份；私钥丢失后，已安装用户可能无法继续接收同一更新链路的更新。
- 替换 `src-tauri/tauri.conf.json` 中的 updater 公钥：
  - 把生成出的公钥写入 `plugins.updater.pubkey`。
  - 在 `plugins.updater.endpoints` 中配置稳定的更新 JSON 地址，例如 GitHub Releases / 对象存储 / 自有服务器上的 `latest.json`。
- 发布构建时用私钥签名：
  - 构建前设置 `TAURI_SIGNING_PRIVATE_KEY` 指向私钥文件。
  - 如果私钥有密码，同时设置 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。
  - 示例：
    - `$env:TAURI_SIGNING_PRIVATE_KEY="$env:USERPROFILE\.tauri\csubtitle-workstation.key"`
    - `npm run tauri build`
- 准备静态更新 JSON：
  - `version` 必须高于当前安装版本。
  - `signature` 填 `.sig` 文件内容本身，不是 `.sig` 文件 URL。
  - `url` 指向对应平台的安装包下载地址。
  - 至少先准备 Windows x64 平台条目，例如 `windows-x86_64`。
- 确定更新文件的最终托管位置：
  - 安装包、签名文件和 `latest.json` 建议统一放在 GitHub Releases、对象存储或 CDN。
  - URL 需要长期稳定，旧版本客户端会持续请求这个 endpoint。
- 前端补齐更新入口：
  - 设置页增加“检查应用更新”按钮。
  - 检测到更新后显示版本号和更新说明。
  - 用户确认后再下载并安装，不做静默强制更新。
- 后端补齐安装命令：
  - 当前已有 `get_current_app_version` 和 `check_app_update`。
  - 继续补 `download_and_install_update`，调用 Tauri updater 下载、安装并提示重启。
- 测试完整升级链路：
  - 先安装 `0.1.0`。
  - 发布 `0.1.1` 更新包和 `latest.json`。
  - 验证应用能检测到更新、完成下载、安装并重启到新版本。
  - 验证签名错误、网络失败、manifest 格式错误时有明确提示。
