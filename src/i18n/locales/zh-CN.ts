export default {
  app: {
    name: 'CC字幕压制工作站',
    subtitle: 'Subtitle WorkStation',
    drop: {
      unsupportedActiveTool: '当前工具不支持拖入此类文件',
      unsupportedHome: '压制页只支持拖入视频或字幕文件',
      unsupportedFile: '不支持拖入此类文件'
    }
  },
  common: {
    selectPlaceholder: '请选择...',
    choose: '选择',
    unset: '未设置',
    copy: '复制',
    edit: '编辑',
    clear: '清空',
    close: '关闭',
    create: '新建',
    rename: '改名',
    delete: '删除',
    export: '导出',
    save: '保存',
    cancel: '取消',
    confirm: '确定',
    unselected: '未选择',
    noContent: '无内容',
    copied: '已复制',
    copyFailed: '复制失败',
    processing: '处理中...',
    showCommandPreview: '显示命令预览',
    hideCommandPreview: '隐藏命令预览',
    commandPreviewDisabledTip: '选择输入和输出后自动生成命令',
    runDisabledTip: '请先补全任务配置'
  },
  nav: {
    home: '压制',
    presets: '预设',
    tools: '工具',
    settings: '设置',
    expand: '展开',
    collapse: '折叠'
  },
  titleBar: {
    minimize: '最小化',
    maximize: '最大化',
    restore: '向下还原',
    close: '关闭'
  },
  toast: {
    regionLabel: '通知'
  },
  settings: {
    language: {
      title: '界面语言',
      description: '切换应用内置界面文案。用户自定义内容、日志和外部更新说明保持原文。',
      label: '语言',
      system: '跟随系统',
      zhCn: '中文',
      enUs: 'English'
    },
    update: {
      title: '应用更新',
      currentVersion: '当前版本 v{version}',
      check: '检查应用更新',
      checkOnStartup: '启动时自动检查更新',
      download: '前往 GitHub 下载',
      notesTitle: '更新日志',
      publishedAt: '发布于 {date}'
    },
    about: {
      title: '关于',
      author: '作者',
      authorTip: '打开作者 GitHub 主页',
      authorAvatarAlt: 'Chinshry 的 GitHub 头像',
      repository: '项目仓库',
      repositoryTip: '在 GitHub 查看源码 / 提 Issue',
      githubRepository: 'GitHub 仓库',
      license: '许可证',
      licenseTip: '查看完整许可证文本'
    },
    ffmpeg: {
      title: 'ffmpeg 设置',
      status: '状态',
      source: '来源',
      version: '版本',
      sourceSystemPath: '系统环境变量 PATH',
      sourceCustomPath: '手动指定路径',
      sourceNotFound: '未找到',
      ffmpegMissing: 'ffmpeg 未找到',
      ffprobeMissingTip: 'ffprobe 未找到（影响视频信息精度，无法检测 CFR/VFR、总帧数）',
      subtitleFilterAvailable: '可压制 ASS 字幕',
      subtitleFilterMissing: '缺少 subtitles/libass filter，无法压制 ASS 字幕',
      ffprobeNotFoundSameDir: '— 未找到（应与 ffmpeg 同目录）',
      choose: '选择 ffmpeg',
      useSystemPath: '使用系统 PATH',
      refresh: '重新检测',
      hideGuide: '收起安装手册',
      showGuide: '没有 ffmpeg？查看安装手册',
      checkingTitle: '正在检测 ffmpeg 环境',
      checkingDescription: '正在检测 ffmpeg / ffprobe / subtitles/libass，请稍候。',
      dialogChoose: '选择 ffmpeg 可执行文件',
      guides: {
        windows: `<h3>Windows 安装 ffmpeg 四步走</h3><ol><li><strong>装解压软件</strong>（如果没装过）：本工具要用的 ffmpeg 发行包是 <code>.7z</code> 格式，Windows 自带的解压不能处理。下载安装 <a href="https://www.7-zip.org/" target="_blank" rel="noopener">7-Zip</a>（免费、官方）。</li><li><strong>下载 ffmpeg</strong>：打开 <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.7z" target="_blank" rel="noopener">ffmpeg-release-full.7z</a>（Gyan.dev 官方发行版，含 ffmpeg / ffprobe + 全部第三方库，<strong>包含 AviSynth+ 支持</strong>，是本工具完整功能所需）。<div class="muted" style="margin-top:4px;">如果只想压制不需要 AVS，下小一些的 <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip" target="_blank" rel="noopener">essentials.zip</a> 也行，但本工具的 AVS 兼容模式不可用。</div></li><li><strong>解压</strong>：右键 .7z 文件 →「7-Zip」→「解压到当前位置」，再把解压出来的文件夹挪到一个你自己选的固定目录（建议非中文路径、非系统盘根目录）。解压后里面的 <code>bin</code> 目录里就有 <code>ffmpeg.exe</code> 和 <code>ffprobe.exe</code>。</li><li><strong>让本工具找到它（任选其一）</strong>：<ul><li><em>简单办法</em> · 在本工具点 <strong>「选择 ffmpeg」</strong>，浏览到刚才解压目录的 <code>bin\\ffmpeg.exe</code> 即可。本工具会自动在同目录寻找 <code>ffprobe.exe</code>。</li><li><em>进阶办法</em> · 把解压目录下的 <code>bin</code> 加入系统 <strong>环境变量 PATH</strong>：开始菜单搜索“环境变量”→「编辑系统环境变量」→「环境变量」→ 在「Path」中点「新建」→ 粘贴你的 <code>bin</code> 目录完整路径 → 一路确定。然后重启本工具，点 <strong>「使用系统 PATH」</strong> + <strong>「重新检测」</strong>，应显示「可用」。</li></ul></li></ol><p class="muted">提示：解压后不要把 ffmpeg.exe 单独移出 bin 目录，它依赖同目录下的其它文件。</p>`,
        macos: `<h3>macOS 安装 ffmpeg</h3><p class="muted" style="margin-top:0;"><em>AVS 仅 Windows 支持，macOS 必须使用带 subtitles/libass filter 的 ffmpeg-full。</em></p><ul><li><strong>方法一（必需）</strong> · 用 Homebrew 安装 ffmpeg-full：<ol style="margin:6px 0 0; padding-left:20px;"><li>终端执行 <code>brew install ffmpeg-full</code></li><li><code>ffmpeg-full</code> 是 keg-only，不会自动覆盖普通 <code>ffmpeg</code>。安装后在本工具点「选择 ffmpeg」，选择：<div class="cmd-block">/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg</div><div class="muted" style="margin-top:6px;">Intel Mac 通常是 <code>/usr/local/opt/ffmpeg-full/bin/ffmpeg</code>。</div></li><li>配置环境变量到 <code>~/.zprofile</code>（本工具读取此文件获取 PATH，写到 <code>~/.zshrc</code> 不生效）。根据 Mac 芯片选一条执行：<div class="muted" style="margin-top:6px;">Apple Silicon（M 系列芯片）：</div><div class="cmd-block">echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' &gt;&gt; ~/.zprofile\neval "$(/opt/homebrew/bin/brew shellenv)"</div><div class="muted" style="margin-top:6px;">Intel Mac：</div><div class="cmd-block">echo 'eval "$(/usr/local/bin/brew shellenv)"' &gt;&gt; ~/.zprofile\neval "$(/usr/local/bin/brew shellenv)"</div><div class="muted" style="margin-top:6px;">不确定芯片？终端执行 <code>uname -m</code>：<code>arm64</code> 是 Apple Silicon，<code>x86_64</code> 是 Intel。</div><div class="muted" style="margin-top:6px;">Homebrew 安装时若已自动写入 <code>~/.zprofile</code>，此步可跳过。</div></li><li>自检命令：<div class="cmd-block">/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg -hide_banner -filters | grep ' subtitles '</div><div class="muted" style="margin-top:6px;">能看到 <code>subtitles V-&gt;V</code> 才能压制 ASS 字幕。不要用 <code>grep -E 'subtitles|ass'</code>，它会误匹配 allpass/bass/highpass。</div></li></ol></li><li><strong>方法二</strong> · 不想装 Homebrew，从 <a href="https://evermeet.cx/ffmpeg/" target="_blank" rel="noopener">evermeet.cx/ffmpeg</a> 下载静态构建。请按页面说明选择与你 Mac 架构匹配的版本，下载后在本工具点「选择 ffmpeg」指向它。选完必须确认 <strong>subtitles/libass</strong> 状态为 ✓。</li></ul><h4>常见排障</h4><ul><li><code>No such filter: subtitles</code>：当前 ffmpeg 缺少 libass/subtitles，安装并选择 <code>ffmpeg-full</code>。</li><li><code>Unable to open .../subtitle.ass</code>：任务临时字幕文件已清理，回到应用重新开始压制。</li><li><code>Missing key frame...</code>：源 MP4 的 edit list 警告，通常不是失败原因。</li></ul>`,
        linux: `<h3>Linux 安装 ffmpeg</h3><p class="muted" style="margin-top:0;"><em>AVS 仅 Windows 支持，Linux 自动走 ffmpeg filter 模式。h264_videotoolbox 编码器不可用。</em></p><ul><li><strong>Debian / Ubuntu</strong> · 终端执行 <code>sudo apt update &amp;&amp; sudo apt install ffmpeg</code>，装好后本工具点「使用系统 PATH」+「重新检测」。</li><li><strong>Fedora / RHEL</strong> · 启用 RPM Fusion 后执行 <code>sudo dnf install ffmpeg</code>。Arch 系直接 <code>sudo pacman -S ffmpeg</code>。</li><li><strong>不想用包管理器</strong> · 从 <a href="https://johnvansickle.com/ffmpeg/" target="_blank" rel="noopener">johnvansickle.com/ffmpeg</a> 下载静态构建（含 ffmpeg + ffprobe，免装依赖）。解压后把可执行文件放到任意目录，在本工具点「选择 ffmpeg」指向它即可。</li></ul><p class="muted">提示：发行版仓库里的 ffmpeg 版本可能偏旧，缺编码器时优先用静态构建。</p>`
      }
    },
    avs: {
      title: 'AVS 设置',
      description: '启用 AVS 兼容模式需要 ffmpeg 启用 avisynth demuxer 且系统已装 AviSynth+。',
      status: '状态',
      demuxerTip: 'ffmpeg 是否启用 --enable-avisynth 构建',
      avisynthTip: '系统是否安装 AviSynth+ 运行环境',
      avisynthVersion: 'AviSynth+ 版本',
      installPath: '安装目录',
      dllNotFound: '— 未在 System32/SysWOW64 找到',
      unavailable: 'AVS 环境不可用，将无法启用 AVS 压制',
      showGuide: '没有 AviSynth+？查看安装手册',
      checkingTitle: '正在检测 AVS 环境',
      checkingDescription: '正在检测 ffmpeg avisynth demuxer 与 AviSynth+，请稍候。',
      guide: `<h3>为什么需要 AVS 兼容模式？</h3><p class="muted" style="margin-top:0;">ffmpeg 默认走 <strong>libass</strong> 渲染 ASS 字幕，标准 ASS 标签兼容良好，但 <strong>VSFilterMod 扩展标签 libass 完全不支持</strong>，强行压制会直接丢特效。常见踩坑场景：</p><ul style="margin-top:4px;"><li><code>$img(...)</code> —— VSFilterMod 的<strong>图片插入</strong>标签，常用于字幕里嵌入 logo / 装饰图。libass 直接当字符串渲染。</li><li><code>\\vc</code> —— vertical color，<strong>垂直渐变填充</strong>。libass 无此扩展。</li><li><code>\\fsvp</code> —— font scale variable percent，<strong>逐字缩放百分比</strong>，K-Pop 字幕组逐字卡拉模板的核心标签。libass 不识别。</li><li><code>\\fax</code> / <code>\\fay</code>（字体 X/Y 倾斜）—— 综艺字幕组高频使用，libass 渲染结果与 VSFilterMod 差异较大。</li><li>GDI 字体的 hinting / 描边细节（libass 用 freetype，外观存在差异）。</li></ul><p class="muted">AVS 兼容模式用 <strong>VSFilterMod 的 TextSubMod</strong> 渲染字幕，与原始 KMPlayer / PotPlayer 软解播放完全一致——<strong>看到的就是压出来的</strong>。</p><h3 style="margin-top:14px;">启用步骤</h3><ol><li><strong>装 AviSynth+ 运行环境</strong>：去 <a href="https://github.com/AviSynth/AviSynthPlus/releases" target="_blank" rel="noopener">AviSynth+ Releases</a> 下载最新稳定版 <code>AviSynthPlus_x.y.z_*-installer.exe</code>。<div class="muted" style="margin-top:4px;"><strong>⚠ 安装时必须勾选 <code>AviSynth+ (x64)</code></strong>，与 64 位 ffmpeg 匹配。默认安装界面只勾了 x64，注意不要误取消；x86 可不装。安装完成后脚本引擎会写入 <code>C:\\Windows\\System32\\AviSynth.dll</code>，安装目录可自定义。</div></li><li><strong>确认 ffmpeg 是 full 版</strong>：本工具要求 ffmpeg 构建时启用 <code>--enable-avisynth</code>。<a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.7z" target="_blank" rel="noopener">full 版</a> 默认包含；essentials 版不包含。完成上面两步后点 <strong>「重新检测」</strong>，AVS 相关徽章变 ✓ 即可在「压制参数」里勾选「AVS 兼容模式」。</li></ol><p class="muted">内置 <code>VSFilterMod.dll</code> + <code>LSMASHSource.dll</code> 已随本工具打包，无需手动配置；切勿把 essentials 版的 ffmpeg 当 full 版用，否则会报 <code>avisynth: Could not initialize ...</code>。</p>`
    },
    lav: {
      title: 'VP9 DirectShow 解码器',
      description: '仅 VP9 视频在 AVS fallback 下需要。普通 AVS 字幕压制不依赖 LAV Filters。',
      ready: 'LAV Filters 已就绪',
      required: '需要 64 位 LAV Filters',
      x64Components: 'x64 组件',
      detected: '已检测到',
      notDetected: '未检测到',
      directshow: 'DirectShow 注册',
      directshowRegistered: 'Splitter + Video Decoder 已注册',
      directshowMissing: '未检测到完整注册',
      showGuide: '没有 LAV Filters？查看安装手册',
      checkingTitle: '正在检测 VP9 解码环境',
      checkingDescription: '正在检测 64 位 LAV Filters 与 DirectShow 注册状态，请稍候。',
      guide: `<h3>什么时候需要 LAV Filters？</h3><p class="muted" style="margin-top:0;">只有 VP9 视频在 AVS fallback 下使用 <code>DirectShowSource</code> 时需要。它提供 64 位 DirectShow 的 MKV splitter 和 VP9 video decoder。</p><h3 style="margin-top:14px;">安装步骤</h3><ol><li>打开 <a href="https://github.com/Nevcairiel/LAVFilters/releases" target="_blank" rel="noopener">LAV Filters Releases</a>，下载 installer 版本，例如 <code>LAVFilters-x.y.z-Installer.exe</code>。</li><li>安装时确认包含 <strong>64 位 Splitter</strong> 和 <strong>64 位 Video Decoder</strong>。安装完成后重启本应用，再点击 <strong>「重新检测」</strong>。</li></ol>`
    },
    debug: {
      title: '🛠 调试面板',
      description: '仅 dev 构建可见。下方开关可强制改变界面呈现，便于测试不同平台 / 缺失场景。',
      nativePlatform: '真实平台：',
      otherPlatform: '其它平台',
      followSystemPlatform: '跟随系统（{platform}）',
      platformOverrideTitle: '平台调试覆盖中',
      platformOverridePrefix: '当前按',
      platformOverrideMiddle: '渲染界面，真实平台为',
      restoreSystem: '恢复跟随系统',
      restoreRealDetection: '恢复真实检测',
      ffmpegMockTitle: 'ffmpeg 调试 mock 中',
      avsMockTitle: 'AVS 调试 mock 中',
      mocking: '当前正在模拟：',
      platformOverride: '平台覆盖',
      ffmpegMocks: 'ffmpeg / ffprobe / 滤镜缺失模拟',
      ffmpegMocksDescription: '不会影响后端真实状态，仅用于演练 UI 反应。三个独立勾选，可叠加。',
      mockFfmpegMissing: '模拟 ffmpeg 缺失',
      mockFfmpegMissingShort: 'ffmpeg 缺失',
      mockFfmpegMissingHint: '— 首页红色横幅 + 设置页 ffmpeg/ffprobe 双 ✕，开始压制按钮变灰',
      mockFfprobeMissing: '模拟 ffprobe 缺失',
      mockFfprobeMissingShort: 'ffprobe 缺失',
      mockFfprobeMissingHint: '— 设置页 ffprobe ✕；视频信息卡隐藏「帧模式」「总帧数」（仅 ffprobe 才能精准给出）',
      mockSubtitleFilterMissing: '模拟 subtitles/libass 缺失',
      mockSubtitleFilterMissingShort: 'subtitles/libass 缺失',
      mockSubtitleFilterMissingHint: '— 设置页 subtitles/libass ✕；首页提示 ffmpeg 功能不完整，开始压制按钮变灰',
      avsMocks: 'AVS 缺失模拟（仅 Windows 生效）',
      avsMocksDescription: '不影响后端检测；用于演练 AVS 设置面板与压制表单在依赖缺失时的反应。',
      mockAvisynthMissing: '模拟 AviSynth+ 缺失',
      mockAvisynthMissingShort: 'AviSynth+ 缺失',
      mockAvisynthMissingHint: '— AVS 面板 AviSynth+ ✕，压制表单 AVS 开关禁用并 tooltip 提示',
      mockAvsDemuxerMissing: '模拟 ffmpeg avisynth demuxer 缺失',
      mockAvsDemuxerMissingShort: 'ffmpeg avisynth demuxer 缺失',
      mockAvsDemuxerMissingHint: '— AVS 面板 demuxer ✕，说明用户用的是 essentials 版 ffmpeg',
      lavMocks: 'VP9 DirectShow 解码器模拟（仅 Windows 生效）',
      lavMocksDescription: '用于演练 VP9 fallback 依赖缺失时的设置面板状态。',
      mockLavFiltersMissing: '模拟 LAV Filters 缺失',
      mockLavFiltersMissingShort: 'LAV Filters 缺失',
      mockLavFiltersMissingHint: '— VP9 DirectShow 解码器面板 ✕，显示需要 64 位 LAV Filters'
    }
  },
  update: {
    message: {
      connecting: '正在连接更新服务器...',
      available: '发现新版本：{version}',
      latest: '当前已是最新版本'
    },
    error: {
      manifestUnavailable: '更新检查失败：无法访问更新清单，请确认 GitHub Pages 已启用，且 docs/updates/latest.json 已提交并推送到远程仓库。',
      generic: '更新检查失败：{message}',
      serverStatus: '更新服务器返回 {status}',
      missingVersion: '更新清单缺少 version 字段'
    }
  },
  debugMock: {
    ffmpegMissing: '[调试] 模拟 ffmpeg 未找到',
    ffprobeMissing: '[调试] 模拟 ffprobe 缺失（仅影响视频信息精度）',
    subtitleFilterMissing: '[调试] 模拟 subtitles/libass filter 缺失',
    avs: '[调试] 模拟 {parts}'
  },
  diagnostics: {
    dropNoPaths: 'WARN: drop event has no paths. 请确认 tauri.conf.json 的 dragDropEnabled=true 已生效（须重启 tauri dev）',
    logoLayoutSavedDefault: 'LOGO 布局已保存为默认配置',
    logoLayoutSaveFailed: '保存 LOGO 布局失败：{message}',
    logoConfigSaved: 'LOGO 配置已保存：{path}'
  },
  logPanel: {
    defaultTitle: '压制进度',
    defaultIdleTitle: '尚未开始压制',
    defaultIdleTip: '配置好参数后点击上方「开始压制」按钮。',
    copyAllTip: '复制全部日志',
    status: {
      running: '运行中',
      cancelling: '取消中...',
      cancelled: '已取消',
      completed: '已完成',
      failed: '已失败',
      idle: '待开始'
    },
    elapsed: '已用',
    remaining: '剩余',
    estimated: '预计',
    duration: '时长',
    size: '大小',
    speed: '速度',
    bitrate: '码率',
    estimatedSize: '/ 预计 {size}',
    noLogs: '暂无日志',
    elapsedTip: '墙钟耗时：从开始压制到现在的真实流逝时间',
    remainingTip: '剩余 = (视频总时长 - 已压制) / 当前速度',
    estimatedTip: '预计总耗时 = 已用 + 剩余（按当前平滑速度估算）'
  },
  commandPreview: {
    title: '命令预览',
    localWindows: '本机 Windows',
    localPosix: '本机 POSIX',
    nativeFormatTip: '默认使用{platform}对应的命令格式',
    terminalDialect: '选择终端方言',
    options: {
      windows: 'Windows 路 cmd/PowerShell',
      posix: 'POSIX 路 bash/zsh',
      raw: '原始（数组拼接，不转义）'
    },
    restoreNative: '恢复本机',
    copyCommandTip: '复制完整命令',
    noCommand: '无命令',
    emptyTip: '填写视频路径后将自动生成命令。',
    hints: {
      raw: '原始数组拼接（含空格/特殊字符路径不能直接粘到终端）',
      windows: '已按 Windows 规则加引号，可粘到 cmd / PowerShell / Windows Terminal',
      posix: '已按 POSIX 规则加引号，可粘到 bash / zsh / Linux & macOS 终端'
    }
  },
  ffmpegStatus: {
    title: 'ffmpeg 状态',
    description: '应用只检测并调用本机 ffmpeg，不内置 ffmpeg。',
    checking: '检测中',
    available: '可用',
    unavailable: '不可用',
    source: '来源',
    path: '路径',
    version: '版本',
    message: '提示'
  },
  tools: {
    sidebarLabel: '工具目录',
    groupListLabel: '工具分组',
    groupToolsLabel: '{group}工具',
    empty: '待添加',
    groups: {
      text: {
        name: '文字处理',
        description: '字幕文本和词库类的轻量处理。'
      },
      format: {
        name: '格式转换',
        description: '字幕格式和容器格式转换入口。'
      },
      media: {
        name: '媒体处理',
        description: '不重新压制的媒体文件辅助操作。'
      }
    },
    items: {
      ccSubtitle: {
        name: 'CC 字幕整理',
        description: '整理 CC 字幕，把 [] 内文字拆成花字行，其余整理为听轴行；支持读取参考 ASS 样式并导出 ASS。'
      },
      textConversion: {
        name: '繁简转换',
        description: '使用 zhconv 转换繁简文本，自定义词库会优先保护和替换指定词条，适合字幕和普通文本批量处理。'
      },
      proofread: {
        name: '字幕校对',
        description: '使用 jieba-rs 分词与词性标注，检查“的 / 地 / 得”疑似误用；自定义词库会提示专有名词、艺人名和固定译名的统一写法。'
      },
      subtitleFormat: {
        name: '字幕格式转换',
        description: '使用 ffmpeg 在 ASS / SSA / SRT / VTT 之间转换；转到 SRT / VTT 时会丢弃原格式不支持的样式和特效。'
      },
      mediaRemux: {
        name: '视频转 MP4',
        description: '把常见视频容器重新封装为 MP4，默认只复制音视频流，不重新编码；TS / M2TS / MTS 会自动整理 AAC 音频封装头。'
      },
      mediaConcatTs: {
        name: 'TS 分片合并',
        description: '按文件顺序合并 TS / M2TS / MTS 分片，可输出 MP4 或 TS；MP4 会自动整理 AAC 音频封装头，不重新编码。'
      },
      mediaMergeAv: {
        name: '合并音视频',
        description: '保留视频画面，合并单独的音频来源输出 MP4；适合替换或补齐外部音轨。'
      },
      mediaCover: {
        name: '添加封面',
        description: '给 MP4 写入 JPG / PNG 封面，原视频和音频会原样复制，不重新编码。'
      }
    },
    ccRule: {
      label: '整理规则',
      title: 'CC 字幕整理规则',
      command: '读取样式 → 选择听轴/花字 → 导入待整理字幕',
      body: '用于把 Web CC 字幕整理成适合 Aegisub 后续精修的 ASS 结构。',
      items: {
        readStyle: '先读取样式参考 ASS，解析 [V4+ Styles]；必须手动选择听轴样式和花字样式。',
        convertInput: 'SRT 输入会转换为 ASS 输出；ASS / SSA 输入会处理已有 Dialogue 行。',
        bracketText: '遇到 [方括号标签]：括号内文本去掉 []，使用花字样式。',
        dialogAfterBracket: '方括号标签后面的台词会另起一条，使用听轴样式。',
        plainDialog: '没有方括号标签的普通台词整条使用听轴样式。',
        cleanText: '台词只处理 \\N 换行和多余空格；',
        dictionary: '启用自定义词库时，会按词库规则替换名称或固定写法。'
      }
    }
  },
  ccSubtitle: {
    dropOverlay: '松开以读取 ASS / SSA / SRT 字幕',
    input: '输入',
    result: '结果',
    charCount: '{count} 字',
    previewLimit: '仅预览前 {count} 字',
    previewTruncated: '... 已省略预览 {count} 字，复制和导出仍使用完整内容。',
    inputPlaceholder: '拖入 ASS / SSA / SRT 字幕文件后在这里预览内容',
    resizeLabel: '调整输入和结果宽度',
    organizing: '整理中...',
    outputSuffix: '_cc整理',
    confirmDeleteProfile: '删除样式方案「{name}」？',
    status: {
      needStyle: '请先读取样式参考 ASS，并选择听轴样式和花字样式。',
      needStyleBeforeImport: '请先读取样式参考 ASS，并选择听轴样式和花字样式，再导入需要整理的 SRT。',
      noStylesParsed: '没有在样式参考 ASS 的 [V4+ Styles] 中解析到样式。',
      profileCreated: '已新建样式方案「{name}」，请选择听轴样式和花字样式。',
      readingSubtitle: '正在读取字幕文件...',
      exported: '已导出：{path}'
    },
    toast: {
      organizeFailed: 'CC 字幕整理失败',
      noStylesParsed: '未解析到样式',
      readStyleFailed: '读取样式失败',
      profileCreated: '已新建样式方案',
      needStyle: '请先读取样式',
      readSubtitleFailed: '读取字幕失败',
      exported: '已导出',
      exportFailed: '导出失败'
    },
    dialog: {
      exportTitle: '导出 CC 字幕整理结果'
    },
    dictionary: {
      title: '替换词库',
      modalTitle: '自定义词库',
      description: '维护 CC 说话人和台词里的名称规则，整理字幕时会把命中的文本替换为标准写法。',
      targetLabel: '标准写法',
      patternLabel: '匹配规则(支持正则)',
      targetPlaceholder: '例如 示例名称',
      patternPlaceholder: '例如 (?i)EXAMPLE\\s*NAME',
      rawPlaceholder: '"示例名称" = "(?i)EXAMPLE\\s*NAME"',
      ariaLabel: 'CC 字幕自定义词库',
      summary: '{count} 条规则 · {status}',
      enabled: '已启用',
      disabled: '未启用',
      enable: '启用'
    },
    style: {
      title: '样式方案',
      description: '为不同字幕组保存参考 ASS 模板和默认听轴/花字样式。',
      noProfile: '尚未配置样式方案',
      chooseStyles: '请选择听轴样式和花字样式',
      summary: '听轴 {speak} / 花字 {screen}',
      configure: '配置样式方案',
      listLabel: '样式方案列表',
      styleCount: '{count} 个样式',
      speakStyle: '听轴样式',
      screenStyle: '花字样式',
      emptyDescription: '新建时会选择一个参考 ASS / SSA 文件，并用文件名作为默认方案名。',
      create: '新建样式方案',
      rename: '重命名样式方案',
      nameLabel: '方案名称',
      untitled: '未命名方案',
      speakUnset: '未选听轴',
      screenUnset: '未选花字'
    }
  },
  textConversion: {
    mode: {
      s2t: '简体转繁体',
      t2s: '繁体转简体',
      s2tShort: '简体 → 繁体',
      t2sShort: '繁体 → 简体',
      directionLabel: '转换方向'
    },
    suffix: {
      s2t: '_繁体',
      t2s: '_简体'
    },
    status: {
      completed: '已完成{mode}',
      copied: '转换结果已复制',
      reading: '正在读取文本文件...',
      loaded: '已读取文本文件。可先补充自定义词库，确认结果后再导出。',
      exporting: '正在导出转换结果...',
      output: '已输出：{path}',
      exported: '已导出：{path}',
      overwritten: '已覆盖：{path}',
      exportCancelled: '已取消导出，未覆盖现有文件。'
    },
    confirmOverwrite: '输出文件已存在：\n{path}\n\n是否覆盖？',
    saveTitle: '导出转换结果',
    dropOverlay: '松开以转换文本或字幕文件',
    dictionaryButton: '自定义词库',
    input: '输入',
    result: '结果',
    charCount: '{count} 字',
    clear: '清空',
    export: '导出',
    inputPlaceholder: '粘贴要处理的文本\n可拖入 TXT / ASS / SSA / SRT / VTT / SUB 文件',
    converting: '正在转换...',
    resultPlaceholder: '转换结果会显示在这里',
    dictionary: {
      title: '自定义词库',
      description: '维护繁简转换后仍需固定的词条，转换时会先匹配规则再输出标准写法。',
      targetLabel: '标准写法',
      patternLabel: '匹配规则(支持正则)',
      targetPlaceholder: '例如 利落',
      patternPlaceholder: '例如 俐落',
      rawPlaceholder: '"利落" = "俐落"',
      ariaLabel: '繁简转换自定义词库'
    }
  },
  proofread: {
    status: {
      found: '发现 {count} 个疑似问题',
      clean: '没有发现的地得疑似问题',
      applied: '已应用：{original} → {suggestion}',
      ignored: '已忽略当前提示',
      copied: '校对文本已复制',
      reading: '正在读取文件...',
      exporting: '正在导出校对结果...',
      output: '已输出：{path}',
      exported: '已导出：{path}',
      overwritten: '已覆盖：{path}',
      exportCancelled: '已取消导出，未覆盖现有文件。'
    },
    confirmOverwrite: '输出文件已存在：\n{path}\n\n是否覆盖？',
    saveTitle: '导出校对结果',
    outputSuffix: '_校对',
    dropOverlay: '松开以校对文本或字幕文件',
    dictionaryButton: '自定义词库',
    text: '文本',
    issueList: '疑似问题',
    checking: '检查中...',
    itemCount: '{count} 项',
    accept: '采纳',
    ignore: '忽略',
    noIssues: '暂未发现疑似问题',
    resultPlaceholder: '校对结果会显示在这里',
    resizeLabel: '调整文本和问题列表宽度',
    dictionary: {
      title: '自定义词库',
      description: '维护专有名词、艺人名和固定译名，校对时会按匹配规则提示统一写法。',
      targetLabel: '标准写法',
      patternLabel: '匹配规则(支持正则)',
      targetPlaceholder: '例如 ZEROBASEONE',
      patternPlaceholder: '例如 (?i)ZE[EROBASN]{7,12}',
      rawPlaceholder: '"ZEROBASEONE" = "(?i)ZE[EROBASN]{7,12}"',
      ariaLabel: '字幕校对自定义词库'
    }
  },
  encoderOptions: {
    libx264: 'CPU libx264（H.264，兼容性最好，支持 AVS）',
    libx265: 'CPU libx265（H.265/HEVC，体积更小，速度较慢）',
    h264Nvenc: 'NVIDIA h264_nvenc（显卡硬编，速度快，不支持 AVS）',
    h264Amf: 'AMD h264_amf（显卡硬编，速度快，不支持 AVS）',
    h264Videotoolbox: 'macOS h264_videotoolbox（Apple 硬编，不支持 AVS）'
  },
  encodeSettings: {
    quality: '质量值',
    qualityTitle: '质量值',
    qualityEmptyCommand: '留空：不生成 -crf / -cq / -qp 参数',
    qualityCommand: 'x264/x265: -crf {crf}  |  NVENC: -cq {crf}',
    qualityBody: '数值越小画质越好、文件越大；留空则不携带质量参数，适合只按码率控制。',
    qualityItems: {
      x264: 'libx264 / libx265 推荐 18-28：18 视觉无损，23 默认，28 偏低质量。',
      hardware: 'NVENC / AMF 推荐 18-28：通常 19-23 比较均衡。',
      videotoolbox: 'VideoToolbox 不使用该质量值，建议通过最大码率控制。'
    },
    emptyPlaceholder: '留空',
    maxBitrate: '最大码率',
    maxBitrateTitle: '最大码率',
    maxBitrateCommand: '-maxrate {值}k -bufsize {值×2}k',
    maxBitrateBody: '限制视频码率峰值，防止画面剧烈变化时码率失控。',
    maxBitrateItems: {
      none: '不限制：完全跟随质量值。',
      auto: '自动：取原视频码率 + 1000 Kbps。',
      custom: '自定义：按填写的 Kbps 直接生效。'
    },
    bitrateOptions: {
      none: '不限制',
      auto: '自动（视频原码率 + 1000 Kbps）',
      custom: '自定义'
    },
    bitratePlaceholder: '如3000',
    encoder: '编码器',
    encoderTitle: '编码器',
    encoderBody: '选择视频编码后端，会影响速度、体积、兼容性和 AVS 支持。',
    encoderItems: {
      x264: 'libx264：H.264 CPU 软编，兼容性最好、画质稳定，支持 AVS。',
      x265: 'libx265：H.265/HEVC CPU 软编，体积更小，速度较慢。',
      hardware: 'h264_nvenc / h264_amf：显卡硬编，速度快，不支持 AVS。',
      videotoolbox: 'h264_videotoolbox：macOS 硬编，不支持 AVS。'
    }
  },
  videoMeta: {
    dialog: {
      chooseVideo: '选择视频文件',
      chooseSubtitle: '选择字幕文件',
      chooseVideoAndSubtitle: '选择视频和字幕文件',
      videoFilter: '视频',
      subtitleFilter: '字幕',
      videoSubtitleFilter: '视频和字幕'
    },
    dash: '—',
    dimensions: '宽 {width} × 高 {height}',
    dimensionTitle: '画面像素尺寸（{tags}）',
    dimensionTitlePlain: '画面像素尺寸（宽×高）',
    durationWithStart: '{duration}（起始 {start}s）',
    fields: {
      resolution: '分辨率',
      codec: '编码',
      pixel: '像素',
      bitrate: '码率',
      fps: '帧率',
      frameMode: '帧模式',
      totalFrames: '总帧数',
      colorSpace: '色域',
      colorRange: '色范围',
      sampleRate: '采样率',
      channels: '声道'
    },
    approx: '约 ',
    defaultTemplate: '命名模板',
    empty: {
      title: '拖入视频开始处理',
      note: '支持单视频处理；需要字幕压制时可同时拖入字幕',
      video: '视频：mp4 / mkv / mov / ts / m4v / flv / avi / webm / wmv / mpg / 3gp / mts',
      subtitle: '字幕可选：ass / ssa / srt / vtt / sub',
      choose: '选择文件'
    },
    title: '视频信息',
    parsing: '解析中...',
    video: '视频',
    subtitle: '字幕',
    output: '输出',
    file: '文件',
    audio: '音频',
    notImported: '未导入',
    unset: '未设置',
    clearAndReimport: '清除并重新拖入',
    clearVideo: '清除视频',
    clearSubtitle: '清除字幕',
    chooseVideo: '选择视频文件',
    chooseSubtitle: '选择字幕文件',
    outputPlaceholder: '例如：E:\\path\\to\\output.mp4',
    applyTemplate: '套用命名模板',
    applyTemplateLabel: '套用命名模板：{name}',
    editOutputPath: '编辑输出路径',
    editOutput: '编辑输出',
    fileSize: '文件大小',
    size: '大小',
    duration: '时长',
    durationTip: '媒体时长（起始时间若不为 0 会在括号中标出）',
    container: '容器',
    tips: {
      videoCodec: '视频编码器与 Profile：决定压缩算法与档次（如 h264 High、hevc Main10）',
      pixelFormat: '像素格式（色彩采样/位深）：常见 yuv420p 为 8bit 4:2:0，yuv420p10le 为 10bit',
      videoBitrate: '视频码率：每秒数据量，越高画质越好、文件越大',
      fps: '帧率：每秒画面数（fps）',
      cfr: '恒定帧率（CFR）：每帧间隔均匀。ffprobe 中 r_frame_rate ≈ avg_frame_rate',
      vfr: '可变帧率（VFR）：帧间隔不均匀，常见于屏幕录制 / 部分网络视频。压制时如需稳定帧率可考虑重映射。',
      estimatedFrames: '估算总帧数（ffprobe 未提供 nb_frames，按时长 × 帧率计算；未进行耗时逐帧统计）',
      exactFrames: '容器中记录的总帧数（来自 ffprobe nb_frames）',
      colorSpace: '色彩空间/原色：常见 bt709（SDR）、bt2020（HDR）',
      colorRange: '亮度范围：tv/limited 为 16-235，pc/full 为 0-255',
      audioCodec: '音频编码器与 Profile：如 aac LC（低复杂度）、HE-AAC、ac3、eac3、opus',
      sampleRate: '采样率：每秒采样次数，常见 44.1 kHz / 48 kHz',
      channels: '声道布局：mono 单声道 / stereo 立体声 / 5.1 环绕 / 7.1 等',
      audioBitrate: '音频码率：每秒数据量',
      demuxer: 'ffmpeg demuxer：{format}（同一 demuxer 处理的所有兼容扩展名都会列出）'
    }
  },
  subtitleCheck: {
    matrix: {
      assRaw: 'ASS 声明',
      videoStandard: '视频色域',
      videoRange: '视频量化范围',
      undeclared: '未声明',
      unknown: '未知'
    },
    missingImage: {
      title: '字幕引用的图片路径不存在',
      detail: 'ASS/SSA 中的 \\img / \\1img-\\4img 图片填充标签引用了本机不存在的文件，AVS/VSFilterMod 渲染时会缺图或失败。',
      suggestion: '请把图片文件放回原路径，或修改字幕中的 img 路径后重新检测。'
    },
    missingFont: {
      title: '字幕使用的字体未检测到安装',
      detail: '缺失字体会触发系统或渲染器字体替换，可能导致字形、字重、排版宽度和特效位置变化。',
      suggestion: '请安装字幕包附带字体，或把 ASS 样式 Fontname 改为本机已安装字体。'
    },
    missingStyle: {
      title: '字幕行引用了不存在的样式',
      detail: 'Events 段中的 Dialogue/Comment 行引用了 Styles 段未定义的样式名，渲染时会回退默认样式或出现异常效果。',
      suggestion: '请在 [V4+ Styles] 中补齐对应 Style，或把事件行的 Style 字段改为已有样式。'
    },
    lineTag: '第 {line} 行 {tag}',
    line: '第 {line} 行',
    effectSuggestion: '建议启用 AVS 压制模式，或确认非 AVS 输出是否符合预期。',
    titleChecking: '字幕检查中',
    title: '字幕检查',
    analyzingSummary: '正在分析字幕特效、字体和资源引用',
    summary: '发现 {count} 个需要确认的项目',
    level: {
      error: '错误',
      warn: '警告',
      info: '建议',
      ok: '正常'
    },
    effects: {
      title: '检测到 VSFilterMod 标签，建议启用 AVS 压制模式',
      banner: '检测到 ASS Effect 字段的 Banner 滚动横幅（带 fadeawaywidth 或小写 banner），ffmpeg libass 渲染不支持该效果，必须使用 AVS+VSFilterMod 才能正确还原。',
      image: '这些标签通常依赖 AVS/VSFilterMod 渲染；请确认素材资源完整，并开启 AVS 压制以尽量还原字幕效果。',
      modTag: '这些标签通常依赖 AVS/VSFilterMod 渲染；建议开启 AVS 压制以尽量还原字幕效果。',
      fallback: '字幕中包含建议使用 AVS 压制的标签，请在压制前确认 AVS 模式已开启。'
    },
    collapse: '收起',
    expand: '展开',
    collapseDetail: '收起详情',
    viewDetail: '查看详情',
    hitTags: '命中标签'
  },
  colorMatrix: {
    standard: {
      unknown: '未知'
    },
    missingBt2020: {
      title: 'ASS 未声明 YCbCr Matrix，但视频是 BT.2020（HDR/4K）',
      detail: 'libass 默认按分辨率启发式选择矩阵（PlayResY≥720→BT.709），在 BT.2020 视频上烧入字幕颜色会偏。'
    },
    missingFullRange: {
      title: 'ASS 未声明 YCbCr Matrix，但视频是 full range',
      detail: 'libass 默认按 limited range 渲染字幕，烧到 full range 视频上黑色会发灰或白色过曝。'
    },
    none: {
      detail: '字幕作者显式跳过 RGB→YUV 转换。除非你清楚意图，否则一般不需要这样设置。'
    },
    unrecognized: {
      title: 'ASS YCbCr Matrix 值无法识别：{value}',
      detail: '标准取值为 TV.601 / TV.709 / TV.2020 / PC.601 / PC.709 / PC.2020 / None'
    },
    matrixMismatch: {
      title: 'ASS 矩阵({ass}) 与视频({video}) 不匹配',
      detail: '烧入字幕颜色会整体偏色（红/蓝偏移），这是 libass / VSFilterMod 按 ASS 声明的矩阵执行 RGB→YUV 导致的。'
    },
    rangeMismatch: {
      title: 'ASS 量化范围({ass}) 与视频({video}) 不匹配',
      detail: '矩阵一致但量化范围不同；黑/白电平会偏，常见表现为字幕黑色发灰或白色过曝。'
    },
    videoUnknown: {
      title: '视频未声明 color_space，无法严格比对'
    },
    ok: {
      title: '色彩矩阵一致（{matrix}）'
    },
    suggestion: {
      addMatrix: '建议在 ASS [Script Info] 段加入：YCbCr Matrix: {matrix}',
      changeTo: '建议改为 {matrix}',
      changeHeader: '建议把 ASS 头部改为：YCbCr Matrix: {matrix}'
    }
  },
  ffmpegPanel: {
    checkingTitle: '正在检测 ffmpeg 环境',
    checkingSubtitle: '正在检测 ffmpeg / ffprobe，请稍候。',
    checkingSubtitleWithLibass: '正在检测 ffmpeg / ffprobe / subtitles/libass，请稍候。',
    incomplete: 'ffmpeg 功能不完整',
    missing: '未检测到 ffmpeg',
    missingHelp: '请前往左侧「设置」面板配置 ffmpeg 路径，或安装后将其加入系统 PATH。',
    refresh: '重新检测'
  },
  home: {
    start: '开始压制',
    cancel: '取消压制',
    commandPreviewDisabledTip: '等待视频与参数就绪后自动生成命令',
    defaultOutputSuffix: ' 中字',
    logoDisabled: {
      noVideo: '请先选择视频文件',
      metaPending: '视频分辨率未解析完毕'
    },
    logs: {
      emptyVideoPath: '错误：视频路径为空，请先填写或拖入视频文件',
      checkingAvsStaging: '正在检查视频与 AVS 预处理需求...',
      avsStagingCancelled: '已取消压制：VP9 AVS 兼容模式需要临时复制源视频。',
      preparingAvsStaging: '正在准备 AVS 临时文件；如果源视频是 VP9，大文件复制期间 ffmpeg 进度会暂时保持 0%。',
      runJobError: 'runJob 异常：{message}',
      cancelRequested: '已发送取消请求'
    },
    avsStaging: {
      kicker: 'AVS 兼容模式',
      title: '需要临时复制 VP9 源视频',
      summary: '检测到 VP9 视频。本次会先把源视频复制到 ASCII 临时路径，再通过本机 64 位 DirectShow 解码链读取视频；通常需要 64 位 LAV Filters。',
      tempSize: '临时占用',
      tempPath: '临时路径',
      continue: '继续压制'
    }
  },
  presets: {
    import: '批量导入',
    export: '批量导出',
    resetBuiltIn: '恢复内置',
    copyName: '{name} 副本',
    outputDir: {
      sameAsVideo: '跟随视频目录',
      sameAsVideoDescription: '输出到源视频所在文件夹',
      fixed: '固定目录',
      fixedDescription: '始终输出到你选择的文件夹'
    },
    confirm: {
      deleteOutputTemplate: '删除模板「{name}」？此操作不会影响已经生成的输出路径。',
      deleteEncodePreset: '删除压制预设「{name}」？',
      resetEncodePresets: '恢复内置压制预设将重置 5 个内置预设（x264 平衡 / NVENC 快速 / AMF 快速 / Apple 快速 / x265 体积优先）的参数，您自定义新增的预设不会被删除。是否继续？'
    },
    dialog: {
      fixedOutputDir: '选择固定输出目录',
      exportOutputTemplates: '导出输出命名模板',
      importOutputTemplates: '导入输出命名模板',
      exportEncodePresets: '导出压制预设',
      importEncodePresets: '导入压制预设'
    },
    toast: {
      fixedDirRequired: '固定目录不能为空，请先选择目录',
      saveFailed: '保存失败：{message}',
      outputTemplateSaved: '命名模板已保存',
      outputTemplateCreated: '已新建命名模板',
      outputTemplateDuplicated: '已复制命名模板',
      outputTemplateDeleted: '已删除命名模板',
      outputTemplateDefault: '已设为默认命名模板',
      outputTemplateReordered: '已调整命名模板顺序',
      outputTemplateExported: '输出命名模板已导出',
      noOutputTemplatesImported: '未找到可导入的输出命名模板',
      outputTemplateImported: '已导入 {count} 个输出命名模板',
      encodePresetSaved: '压制预设已保存',
      encodePresetExported: '压制预设已导出',
      noEncodePresetsImported: '未找到可导入的压制预设',
      encodePresetImported: '已导入 {count} 个压制预设',
      encodePresetCreated: '已新建压制预设',
      encodePresetDuplicated: '已复制压制预设',
      encodePresetDeleted: '已删除压制预设',
      encodePresetReordered: '已调整压制预设顺序',
      encodePresetReset: '已恢复内置压制预设（自定义预设保留）'
    },
    encodePreset: {
      title: '压制预设',
      description: '管理常用编码器、CRF、最大码率和高级 ffmpeg 视频参数；压制页可直接选择并应用。',
      defaultName: '压制预设 {index}',
      nameLabel: '预设名称',
      advancedArgs: '高级 ffmpeg 视频参数',
      applyPreview: '应用后会覆盖',
      encoderSummary: '编码器 {encoder} / CRF {crf} / 码率 {bitrate}',
      save: '保存预设'
    },
    outputTemplate: {
      title: '输出命名模板',
      description: '建立常用命名规则，在压制页选择模板后可一键套用到输出路径。',
      defaultName: '模板 {index}',
      untitled: '未命名模板',
      nameLabel: '模板名称',
      patternLabel: '文件名模板',
      variablesLabel: '可插入的文件名变量',
      insertVariable: '插入变量',
      insertVariableHint: '点击后插入到光标位置',
      outputDir: '输出目录',
      fixedDirPlaceholder: '请点击右侧按钮选择目录',
      chooseDir: '选择目录',
      fixedDirHint: '固定目录必须通过系统目录选择器设置',
      preview: '示例预览',
      save: '保存模板',
      setDefault: '设为默认'
    }
  },
  outputTemplates: {
    defaultName: '默认',
    defaultPattern: '{video_name} 中字.mp4',
    variables: {
      dateYmd: '日期格式1',
      dateShort: '日期格式2',
      videoName: '视频文件名',
      resolution: '分辨率',
      encoder: '编码器',
      crf: 'CRF'
    }
  },
  encodePresets: {
    balancedX264: 'x264 平衡',
    fastNvenc: 'NVENC 快速',
    fastAmf: 'AMF 快速',
    fastVideotoolbox: 'Apple 快速',
    hevcSmall: 'x265 体积优先'
  },
  logoEditor: {
    title: '配置 LOGO 位置',
    confirmUnsaved: '当前 LOGO 配置尚未保存，确定要关闭吗？',
    dialog: {
      chooseLogo: '选择 LOGO 图片',
      imageFilter: '图片'
    },
    errors: {
      imageLoadFailed: 'LOGO 图片加载失败',
      chooseLogoFirst: '请先选择 LOGO 图片'
    },
    drop: {
      release: '松开以载入图片',
      choose: '选择 LOGO 图片',
      hint: '点击选择，或拖入 PNG / JPG / WEBP / BMP'
    },
    current: {
      title: '当前 LOGO',
      empty: '未选择 LOGO'
    },
    recent: {
      title: '最近使用',
      empty: '暂无记录',
      removeTip: '移除 {name}'
    },
    status: {
      position: '位置：',
      size: '尺寸：',
      percent: '百分比：',
      video: '视频：'
    },
    frame: {
      alt: '预览帧',
      loading: '抽帧中…',
      loadingAria: '抽帧中',
      waiting: '等待视频帧…',
      refresh: '重新抽取当前帧'
    },
    zoom: {
      aria: '画面缩放比例',
      zoom: '缩放：',
      reset: '复位：',
      wheel: '滚轮'
    }
  },
  compressForm: {
    title: '压制参数',
    preset: {
      title: '压制预设',
      apply: '套用预设',
      applyTip: '选择一个压制预设并套用到当前参数',
      lastApplied: '上次套用：{name}',
      tip: '一键应用一组编码器、质量值、最大码率组合。\n\n在左侧侧边栏「预设」页面新增、修改或删除预设。\n应用预设后，下方质量值/最大码率/编码器仍可手动微调，不会回写到预设本身。',
      itemManage: '在左侧「预设」页面新增、修改或删除预设。',
      itemUseCase: '适合把常用平台规格保存成固定方案。',
      applied: '已套用预设',
      appliedNamed: '已套用到当前参数：{name}',
      qualitySummary: '质量 {crf}',
      unlimitedBitrate: '不限码率',
      autoBitrate: '自动码率',
      sourceBitratePlus: '原视频码率+1000k',
      doubleMaxBitrate: '2倍最大码率'
    },
    quick: {
      title: '视频处理',
      off: '关闭',
      custom: '自定义',
      noneSelected: '未选择处理项',
      customScaleSummary: '缩放 {value}',
      body: '把旋转、镜像、分辨率、帧率和视频码率处理编译进当前压制命令，会重新编码视频并输出新文件。',
      itemUseCase: '适合旋转、镜像、缩放、抽帧或码率调整。',
      itemReuse: '字幕、LOGO、编码器和质量值仍复用当前压制页设置。',
      rotate: '旋转',
      rotateBody: '在压制时旋转输出画面，会写入视频滤镜并重新编码画面。',
      rotateItemUseCase: '用于手机竖屏、录屏方向错误等场景。',
      rotateItem180: '180° 使用 hflip,vflip，效果等同画面倒转。',
      mirrorLabel: '镜像',
      mirrorBody: '在压制时对画面做横向或竖向镜像翻转，可与旋转同时使用。',
      mirrorItemH: '横向镜像是左右翻转。',
      mirrorItemV: '竖向镜像是上下翻转。',
      scaleLabel: '分辨率',
      scaleBody: '按预设或自定义表达式缩放输出画面，宽高会尽量保持原比例。',
      scaleItemLandscape: '横屏预设按高度控制，例如 1080 表示输出高 1080。',
      scaleItemPortrait: '竖屏预设按宽度控制，例如 1080 表示输出宽 1080。',
      customScale: '自定义缩放',
      customScaleCommand: 'scale=宽:高',
      customScaleBody: '直接填写 ffmpeg scale 的宽高表达式，用于预设无法覆盖的尺寸。',
      customScaleItemHeight: '例如 -1:1080 表示高度 1080，宽度按比例自动计算。',
      customScaleItemWidth: '例如 1080:-1 表示宽度 1080，高度按比例自动计算。',
      customScalePlaceholder: '如 -1:1080 或 1080:-1',
      frameRate: '帧率',
      frameRateBody: '限制输出视频的帧率，常用于压低体积或统一发布规格。',
      frameRateItemEmpty: '留空表示不调整帧率。',
      frameRateItemValues: '填写 30 会输出 30 fps；填写 60 会输出 60 fps。',
      videoBitrate: '视频码率',
      videoBitrateBody: '为视频流指定目标码率，主要用于控制输出体积和平台规格。',
      videoBitrateItemEmpty: '留空表示不额外指定视频码率，仍使用当前编码器和质量值。',
      videoBitrateItemValue: '填写 5000 表示目标视频码率约 5000 Kbps。',
      noChange: '不调整',
      rotation: {
        none: '不旋转',
        cw: '顺时针 90°',
        ccw: '逆时针 90°',
        rotate180: '旋转 180°'
      },
      mirror: {
        none: '不镜像',
        hflip: '横向镜像',
        vflip: '竖向镜像'
      },
      scale: {
        none: '不调整分辨率',
        landscape4k: '横屏 4K（高 2160）',
        landscape1080: '横屏 1080（高 1080）',
        landscape720: '横屏 720（高 720）',
        portrait1080: '竖屏 1080（宽 1080）',
        portrait720: '竖屏 720（宽 720）'
      }
    },
    advanced: {
      show: '显示附加参数',
      hide: '隐藏附加参数',
      label: '附加 ffmpeg 视频参数',
      note: '这些参数会追加到视频编码参数后；输入、滤镜、编码器、音频和输出路径仍由工作站管理。'
    },
    options: {
      yadif: '使用反交错压制',
      yadifTitle: '反交错压制',
      yadifBody: '把交错信号合成连续画面，消除横向锯齿或梳状伪影。',
      yadifItemInterlaced: 'TV 录制、转录、DV、磁带数字化素材常见隔行，建议开启。',
      yadifItemProgressive: '网络发布视频通常已经是逐行扫描，一般不需要开启。'
    },
    logo: {
      title: '压制 LOGO',
      body: '在视频画面上叠加一张 LOGO 图片，可视化设置图片、位置与大小。',
      itemConfigure: '点击「配置 LOGO」进入编辑器。',
      itemKeepLayout: '关闭开关时，已保存的 LOGO 布局会保留，但不会参与压制。',
      openTip: '打开 LOGO 编辑器，可视化设置图片、位置与大小',
      configure: '配置 LOGO',
      reconfigure: '重新配置 LOGO',
      notConfigured: '尚未配置 LOGO',
      summary: '已配置：{name} · {position} · {size}',
      sizePixels: '{width} × {height} 像素',
      layer: 'LOGO 层级',
      layerBody: '控制字幕和 LOGO 的覆盖顺序。',
      layerItemBottom: '字幕在上 LOGO 在下：LOGO 会被字幕遮挡。',
      layerItemTop: 'LOGO 在上 字幕在下：LOGO 完整覆盖字幕。',
      layerItemAvs: 'AVS 模式下字幕由 AVS 渲染，LOGO 层级会锁定为 LOGO 在上。',
      layerBottom: '字幕在上 LOGO 在下',
      layerBottomTitle: 'LOGO 会被字幕遮挡',
      layerTop: 'LOGO 在上 字幕在下',
      layerTopTitle: 'LOGO 完整覆盖字幕',
      position: {
        center: '画面中央',
        topCenter: '顶部居中',
        bottomCenter: '底部居中',
        leftCenter: '左侧居中',
        rightCenter: '右侧居中',
        topLeft: '左上角',
        topRight: '右上角',
        bottomLeft: '左下角',
        bottomRight: '右下角'
      }
    },
    avs: {
      title: 'AVS 压制模式',
      body: '启用 AviSynth+ 脚本作为 ffmpeg 输入，字幕由 VSFilterMod 渲染；LOGO overlay 与 yadif 仍然有效。',
      itemWindows: '仅 Windows 支持，需要本机安装 AviSynth+，且 ffmpeg 启用 --enable-avisynth。',
      itemUseCase: '适合复杂 ASS 特效字幕；不勾选则走 ffmpeg filter 模式。',
      toggleTip: '启用 AviSynth+ 脚本作为 ffmpeg 输入，字幕由 VSFilterMod 的 TextSubMod 渲染。\n仅 Windows 支持；需要本机安装 AviSynth+ 且 ffmpeg 启用了 --enable-avisynth（如 Gyan.dev full 版）。\n启用后 LOGO overlay 与 yadif 仍然有效，但 ffmpeg subtitles 滤镜会被跳过。\n\n可启用 AVS 压制模式；不勾选则走 ffmpeg filter 模式。',
      windowsOnly: 'AVS 压制仅 Windows 支持',
      checking: '正在检测 AVS 环境…',
      unavailable: 'AVS 环境不可用',
      lavResolving: '正在检测 64 位 LAV Filters；VP9 视频需确认 DirectShow 解码支持后才能启用 AVS…',
      lavCheckingTip: '正在检测 64 位 LAV Filters。\nVP9 视频经 DirectShow 解码链读取，确认解码支持后才能启用 AVS。',
      lavMissingTip: '未检测到 64 位 LAV Filters。\n压制 VP9 视频时会经 DirectShow 解码链读取，缺少 LAV 可能导致解码失败。\n建议安装 64 位 LAV Filters 后重试。',
      autoEnabled: '检测到字幕特效（{tags}），已自动启用 AVS 压制',
      detectedSpecialTags: '检测到特殊标签',
      lavChecking: 'LAV 检测中…',
      lavMissing: 'LAV 缺失'
    }
  },
  subtitleFormat: {
    ready: '可以开始转换',
    disabled: {
      checking: '正在检测 ffmpeg',
      ffmpegUnavailable: '请先在设置页配置可用的 ffmpeg',
      input: '请选择输入字幕',
      output: '请选择输出字幕路径',
      conflict: '输出路径不能和输入字幕相同'
    },
    outputFile: '{stem} 转换.{format}',
    dialog: {
      inputTitle: '选择要转换格式的字幕',
      subtitleFilter: '字幕',
      outputTitle: '选择输出字幕路径'
    },
    outputLog: '已输出：{path}',
    dropzone: {
      title: '拖入字幕开始转换',
      note: '支持 ASS / SSA / SRT / VTT / SUB',
      description: '选择目标格式后会自动生成输出路径；转换到 SRT / VTT 时会简化不支持的样式。',
      choose: '选择字幕'
    },
    inputLabel: '输入字幕',
    inputPlaceholder: '选择 ass / ssa / srt / vtt / sub 字幕文件',
    targetFormat: '目标格式',
    targetFormatTitle: '选择输出字幕格式',
    outputLabel: '输出字幕',
    outputPlaceholder: '选择输出字幕路径',
    conflictWarning: '输出路径不能和输入字幕相同，请选择一个新文件。',
    noteTitle: '处理说明',
    noteAss: 'ASS / SSA 保留样式能力更强；SRT / VTT 更通用，但只能表达基础文本和时间轴。',
    noteSimplify: '如果源字幕包含复杂定位、特效、字体样式，转换成 SRT / VTT 后这些信息会按目标格式能力被简化。',
    start: '开始转换',
    cancel: '取消转换',
    running: '转换中...',
    progressTitle: '转换进度',
    idleTitle: '尚未开始转换',
    idleTip: '选择输入、目标格式和输出路径后点击上方「开始转换」。'
  },
  mediaTool: {
    ready: '可以开始转换',
    disabled: {
      checking: '正在检测 ffmpeg',
      ffmpegUnavailable: '请先在设置页配置可用的 ffmpeg',
      segmentFolder: '请选择分片目录',
      inputVideo: '请选择输入视频',
      cover: '请选择封面图片',
      audio: '请选择音频来源',
      output: '请选择输出 {format} 路径',
      conflict: '输出路径不能和输入文件相同',
      loadingSegments: '正在读取分片列表',
      noSegments: '所选目录中没有可合并的 TS / M2TS / MTS 分片'
    },
    description: {
      concatTsToMp4: '按当前排序合并 TS / M2TS / MTS 分片，输出 {format}，不重新编码。',
      addCoverToMp4: '给 MP4 写入 JPG / PNG 封面，原视频和音频会原样复制。',
      mergeAudioVideo: '保留视频画面，合并单独的音频来源，输出 MP4。',
      remuxToMp4: '把常见视频容器重新封装为 MP4，默认只复制音视频流。'
    },
    outputName: {
      concatTsToMp4: '{stem} 合并.{extension}',
      addCoverToMp4: '{stem} 添加封面.mp4',
      mergeAudioVideo: '{stem} 合并音频.mp4',
      remuxToMp4: '{stem} MP4封装.mp4'
    },
    dialog: {
      inputCoverVideo: '选择要添加封面的 MP4 视频',
      inputVideo: '选择要转为 MP4 封装的视频文件',
      videoFilter: '视频',
      cover: '选择封面图片',
      coverFilter: '封面图片',
      audio: '选择要合并的音频来源文件',
      audioVideoFilter: '音频或视频',
      segmentFolder: '选择 TS 分片所在文件夹',
      output: '选择输出 {format} 文件',
      outputFilter: '{format} 视频'
    },
    dropzone: {
      concatTitle: '拖入 TS 分片目录开始合并',
      videoTitle: '拖入视频开始处理',
      concatNote: '支持 TS / M2TS / MTS 分片目录',
      coverNote: '支持 MP4 / M4V / MOV',
      videoNote: '支持常见视频容器',
      concatDescription: '读取分片后会按文件名顺序合并，可选择输出 MP4 或 TS。',
      mergeDescription: '选择视频后再补充音频来源，默认原样复制不重新编码。',
      coverDescription: '选择视频后再补充 JPG / PNG 封面图，原音视频不重新编码。',
      remuxDescription: '选择视频后会自动生成输出 MP4 路径，默认只复制音视频流。'
    },
    chooseSegmentFolder: '选择分片目录',
    chooseVideo: '选择视频',
    input: {
      segmentFolder: '分片目录',
      video: '输入视频',
      segmentPlaceholder: '选择包含 .ts / .m2ts / .mts 的文件夹',
      coverVideoPlaceholder: '选择 mp4 / m4v / mov 视频文件',
      mergeVideoPlaceholder: '选择要保留画面的视频文件',
      videoPlaceholder: '选择 mkv / mov / ts / flv 等视频文件',
      audio: '输入音频',
      audioPlaceholder: '选择音频或视频文件',
      cover: '封面图片',
      coverPlaceholder: '选择 jpg / png 封面图片'
    },
    outputFormat: '输出格式',
    outputFormatTitle: '选择 TS 分片合并输出格式',
    outputLabel: '输出 {format}',
    outputPlaceholder: '选择输出位置',
    conflictWarning: '输出路径不能和输入文件相同，请选择一个新的 {format} 文件。',
    noteTitle: '处理说明',
    notes: {
      remux1: '将视频换成 MP4 容器，视频流和音频流默认原样复制，不重新编码。',
      remux2: 'TS / M2TS / MTS 输入会自动整理 AAC 音频封装头，让它符合 MP4 规范；这不会改变音质。',
      cover1: '在 MP4 中写入一张封面图，适合让播放器和文件管理器显示自定义封面。',
      cover2: '原视频和音频会原样复制，不重新编码；封面图会作为封面流写入文件。',
      merge1: '保留输入视频的画面，并把音频来源文件的第一条音轨作为输出文件的主音轨。',
      merge2: '视频和音频默认原样复制，不重新编码；输出会按较短的一路结束，避免尾部空跑。'
    },
    segments: {
      title: '分片顺序预览',
      loading: '读取中...',
      count: '{count} 个文件 · {size}',
      empty: '尚未读取到分片',
      noSegments: '所选文件夹中没有 TS / M2TS / MTS 分片。',
      truncated: '仅显示前 {visible} 个；实际会按当前排序合并全部 {total} 个分片。',
      note: '选择 MP4 时会自动整理 TS 分片中 AAC 音频的封装头；选择 TS 时会保留 TS 容器输出，均不重新编码。'
    },
    errors: {
      incompatibleContainer: '当前音视频流可能不兼容 {format} 容器；请到压制页重新编码后再输出。'
    },
    cancelRequested: '已发送取消请求',
    start: '开始转换',
    cancel: '取消转换',
    previewDisabledTip: '选择输入和输出后自动生成命令',
    progressTitle: '转换进度',
    idleTitle: '尚未开始转换',
    idleTip: '选择输入和输出后点击上方「开始转换」按钮'
  },
  ruleDictionary: {
    close: '关闭',
    tabsLabel: '词库编辑方式',
    entryTab: '词条编辑',
    rawTab: '原始文本',
    add: '新增',
    delete: '删除',
    invalidLine: '这一行格式无法识别，可切到原始文本检查。',
    invalidPattern: '匹配规则看起来不是有效正则。',
    addFirst: '新增第一条规则',
    testMatch: '试匹配',
    testPlaceholder: '输入一小段字幕文本，检查上面的规则会不会命中',
    noMatches: '当前没有匹配到任何词条。',
    summary: '{valid} 条可用规则',
    invalidSummary: '，{invalid} 条需要检查',
    autoSaveHint: '。修改后会自动记忆并重新处理',
    captureHint: '；目标文本支持 %1 捕获组',
    endPunctuation: '。',
    done: '完成'
  }
} as const
