<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { loadConfig, saveConfig } from '../api/config'
import { setFfmpegPath, resetFfmpegToSystem } from '../api/ffmpeg'
import { getCurrentAppVersion } from '../api/updater'
import {
  ffmpegStatus,
  initFfmpegStatus,
  isFfmpegMissingMocked,
  isFfmpegMocked,
  isFfprobeMissingMocked,
  isSubtitleFilterMissingMocked,
  clearAllFfmpegMocks,
  refreshFfmpegStatus,
  setFfmpegMissingMock,
  setFfprobeMissingMock,
  setSubtitleFilterMissingMock,
  setFfmpegStatus
} from '../stores/ffmpegStore'
import {
  isWindows,
  isMacOS,
  isLinux,
  isPlatformOverridden,
  nativePlatform,
  nativePlatformLabel,
  platform,
  platformLabel,
  setPlatformOverride,
  type Platform
} from '../stores/platformStore'
import { avsStatus, initAvsStatus, refreshAvsStatus, isAvisynthMissingMocked, isAvsDemuxerMissingMocked, setAvisynthMissingMock, setAvsDemuxerMissingMock, clearAllAvsMocks, isAvsMocked } from '../stores/avsStore'
import {
  availableUpdateVersion,
  refreshAppUpdate,
  updateInfo,
  updateMessage,
  updateReleaseUrl,
  updateState
} from '../stores/updateStore'
import type { AppConfig } from '../types'
import authorAvatarUrl from '../assets/avatar-chinshry.png'

const status = ffmpegStatus
const appVersion = ref('')
const appConfig = ref<AppConfig | null>(null)
const guideOpen = ref(false)
const avsGuideOpen = ref(false)
const debugPanelOpen = ref(false)

// 作者头像：用 vite 打包的本地静态资源，离线 / 网络受限场景始终可用
// 仍保留 onError fallback，对极端情况（资源构建丢失）兜底
const avatarFailed = ref(false)
function onAvatarError() {
  avatarFailed.value = true
}

// 仅在开发构建里显示调试面板
const isDev = import.meta.env.DEV

const sourceText = computed(() => {
  switch (status.value?.source) {
    case 'system_path': return '系统环境变量 PATH'
    case 'custom_path': return '手动指定路径'
    case 'not_found': return '未找到'
    default: return '—'
  }
})

// 调试覆盖单选项
const overrideOptions: Array<{ value: Platform | null; label: string }> = [
  { value: null, label: `跟随系统（${nativePlatformLabel}）` },
  { value: 'windows', label: 'Windows' },
  { value: 'macos', label: 'macOS' },
  { value: 'linux', label: 'Linux' }
]

const overrideModel = computed<Platform | 'native'>({
  get: () => (isPlatformOverridden.value ? platform.value : 'native'),
  set: (value) => {
    setPlatformOverride(value === 'native' ? null : value)
  }
})

// ffmpeg 调试 mock：两个独立开关，可同时勾选
const ffmpegMissingModel = computed<boolean>({
  get: () => isFfmpegMissingMocked.value,
  set: (value) => setFfmpegMissingMock(value)
})

const ffprobeMissingModel = computed<boolean>({
  get: () => isFfprobeMissingMocked.value,
  set: (value) => setFfprobeMissingMock(value)
})

const subtitleFilterMissingModel = computed<boolean>({
  get: () => isSubtitleFilterMissingMocked.value,
  set: (value) => setSubtitleFilterMissingMock(value)
})

const mockSummary = computed(() => {
  const parts: string[] = []
  if (isFfmpegMissingMocked.value) parts.push('ffmpeg 缺失')
  if (isFfprobeMissingMocked.value) parts.push('ffprobe 缺失')
  if (isSubtitleFilterMissingMocked.value) parts.push('subtitles/libass 缺失')
  return parts.join(' + ')
})

// AVS 调试 mock
const avisynthMissingModel = computed<boolean>({
  get: () => isAvisynthMissingMocked.value,
  set: (value) => setAvisynthMissingMock(value)
})
const avsDemuxerMissingModel = computed<boolean>({
  get: () => isAvsDemuxerMissingMocked.value,
  set: (value) => setAvsDemuxerMissingMock(value)
})
const avsMockSummary = computed(() => {
  const parts: string[] = []
  if (isAvisynthMissingMocked.value) parts.push('AviSynth+ 缺失')
  if (isAvsDemuxerMissingMocked.value) parts.push('ffmpeg avisynth demuxer 缺失')
  return parts.join(' + ')
})

async function refresh() {
  await refreshFfmpegStatus()
}

async function chooseFfmpeg() {
  const selected = await open({
    multiple: false,
    directory: false,
    title: '选择 ffmpeg 可执行文件'
  })
  if (typeof selected === 'string') {
    setFfmpegStatus(await setFfmpegPath(selected))
  }
}

async function useSystemPath() {
  setFfmpegStatus(await resetFfmpegToSystem())
}

async function checkUpdate() {
  await refreshAppUpdate()
}

const updateResultTitle = computed(() => {
  if (updateState.value === 'error') return '无法检查更新'
  if (updateState.value === 'progress') return '正在处理'
  if (updateInfo.value?.available) return '发现新版本'
  return '当前版本'
})

const updateNotesTitle = computed(() => (
  updateInfo.value?.available ? '新版本更新日志' : '当前版本更新日志'
))

async function setStartupUpdateCheck(value: boolean) {
  if (!appConfig.value) return
  const next = {
    ...appConfig.value,
    checkUpdateOnStartup: value
  }
  appConfig.value = next
  await saveConfig(next)
}

// 异步按钮 loading 状态管理：
// - 延迟 180ms 才显示 spinner：耗时极短的操作（如 mock 检测）完全跳过 spinner，避免一闪
// - 一旦显示，至少停留 400ms：消除"显示瞬间又消失"造成的布局闪烁
const busy = reactive<Record<string, boolean>>({})
const isBusy = (key: string) => !!busy[key]

const SPINNER_SHOW_DELAY = 180
const SPINNER_MIN_DURATION = 400

async function withBusy<T>(key: string, fn: () => Promise<T>): Promise<T | undefined> {
  if (busy[key]) return
  let shownAt: number | null = null
  const timer = window.setTimeout(() => {
    busy[key] = true
    shownAt = Date.now()
  }, SPINNER_SHOW_DELAY)
  try {
    return await fn()
  } finally {
    if (shownAt !== null) {
      const elapsed = Date.now() - shownAt
      if (elapsed < SPINNER_MIN_DURATION) {
        await new Promise<void>((r) => window.setTimeout(r, SPINNER_MIN_DURATION - elapsed))
      }
      busy[key] = false
    } else {
      window.clearTimeout(timer)
    }
  }
}

onMounted(async () => {
  appVersion.value = await getCurrentAppVersion()
  appConfig.value = await loadConfig()
  // 切到设置页不再重复检测，仅在首次进入时跑一次
  await initFfmpegStatus()
  if (isWindows.value) {
    await initAvsStatus()
  }
})
</script>

<template>
  <main class="workspace">
    <!-- 调试覆盖横幅：覆盖生效时即可见（生产版也保留提示，避免误以为 bug） -->
    <div v-if="isPlatformOverridden" class="debug-banner">
      <strong>⚠ 平台调试覆盖中</strong>
      <span>
        当前按
        <code>{{ platformLabel(platform) }}</code>
        渲染界面，真实平台为
        <code>{{ nativePlatformLabel }}</code>。
      </span>
      <button class="secondary" @click="setPlatformOverride(null)">恢复跟随系统</button>
    </div>

    <div v-if="isFfmpegMocked" class="debug-banner">
      <strong>⚠ ffmpeg 调试 mock 中</strong>
      <span>
        当前正在模拟：<code>{{ mockSummary }}</code>。
      </span>
      <button class="secondary" @click="clearAllFfmpegMocks">恢复真实检测</button>
    </div>

    <section class="panel">
      <div class="panel-heading">
        <div>
          <h2>ffmpeg 设置</h2>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>状态</dt>
          <dd>
            <span class="status-pill" :class="status?.available ? 'ok' : 'bad'" v-tooltip="status?.ffmpegPath ?? 'ffmpeg 未找到'">
              <span class="status-icon">{{ status?.available ? '✓' : '✕' }}</span>
              <span>ffmpeg</span>
            </span>
            <span class="status-pill" :class="status?.ffprobePath ? 'ok' : 'bad'" style="margin-left:8px;" v-tooltip="status?.ffprobePath ?? 'ffprobe 未找到（影响视频信息精度，无法检测 CFR/VFR、总帧数）'">
              <span class="status-icon">{{ status?.ffprobePath ? '✓' : '✕' }}</span>
              <span>ffprobe</span>
            </span>
            <span class="status-pill" :class="status?.subtitleFilterAvailable ? 'ok' : 'bad'" style="margin-left:8px;" v-tooltip="status?.subtitleFilterAvailable ? '可压制 ASS 字幕' : '缺少 subtitles/libass filter，无法压制 ASS 字幕'">
              <span class="status-icon">{{ status?.subtitleFilterAvailable ? '✓' : '✕' }}</span>
              <span>subtitles/libass</span>
            </span>
          </dd>
        </div>
        <div><dt>来源</dt><dd>{{ sourceText }}</dd></div>
        <div><dt>ffmpeg</dt><dd>{{ status?.ffmpegPath ?? '—' }}</dd></div>
        <div><dt>ffprobe</dt><dd>{{ status?.ffprobePath ?? '— 未找到（应与 ffmpeg 同目录）' }}</dd></div>
        <div><dt>版本</dt><dd>{{ status?.ffmpegVersion ?? '—' }}</dd></div>
      </dl>
      <p v-if="status?.message" class="notice" style="color:#a35000;">
        ⚠ {{ status.message }}
      </p>
      <div class="actions left">
        <button
          :class="{ 'is-busy': isBusy('chooseFfmpeg') }"
          @click="withBusy('chooseFfmpeg', chooseFfmpeg)"
        >选择 ffmpeg</button>
        <button
          class="secondary"
          :class="{ 'is-busy': isBusy('useSystemPath') }"
          @click="withBusy('useSystemPath', useSystemPath)"
        >使用系统 PATH</button>
        <button
          class="secondary"
          :class="{ 'is-busy': isBusy('ffmpegRefresh') }"
          @click="withBusy('ffmpegRefresh', refresh)"
        >重新检测</button>
        <button class="secondary" @click="guideOpen = !guideOpen">
          {{ guideOpen ? '收起安装手册' : '没有 ffmpeg？查看安装手册' }}
        </button>
      </div>

      <!-- Windows 安装指南 -->
      <div v-if="guideOpen && isWindows" class="install-guide">
        <h3>Windows 安装 ffmpeg 四步走</h3>
        <ol>
          <li>
            <strong>装解压软件</strong>（如果没装过）：本工具要用的 ffmpeg 发行包是 <code>.7z</code> 格式，Windows 自带的解压不能处理。下载安装
            <a href="https://www.7-zip.org/" target="_blank" rel="noopener">7-Zip</a>
            （免费、官方）。
          </li>
          <li>
            <strong>下载 ffmpeg</strong>：打开
            <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.7z" target="_blank" rel="noopener">
              ffmpeg-release-full.7z
            </a>
            （Gyan.dev 官方发行版，含 ffmpeg / ffprobe + 全部第三方库，<strong>包含 AviSynth+ 支持</strong>，是本工具完整功能所需）。
            <div class="muted" style="margin-top:4px;">如果只想压制不需要 AVS，下小一些的
              <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip" target="_blank" rel="noopener">essentials.zip</a>
              也行，但本工具的 AVS 兼容模式不可用。
            </div>
          </li>
          <li>
            <strong>解压</strong>：右键 .7z 文件 →「7-Zip」→「解压到当前位置」，再把解压出来的文件夹挪到一个你自己选的固定目录（建议非中文路径、非系统盘根目录）。解压后里面的 <code>bin</code> 目录里就有 <code>ffmpeg.exe</code> 和 <code>ffprobe.exe</code>。
          </li>
          <li>
            <strong>让本工具找到它（任选其一）</strong>：
            <ul>
              <li>
                <em>简单办法</em> · 在本工具点
                <strong>「选择 ffmpeg」</strong>，浏览到刚才解压目录的 <code>bin\ffmpeg.exe</code> 即可。本工具会自动在同目录寻找 <code>ffprobe.exe</code>。
              </li>
              <li>
                <em>进阶办法</em> · 把解压目录下的 <code>bin</code> 加入系统
                <strong>环境变量 PATH</strong>：开始菜单搜索"环境变量"→「编辑系统环境变量」→「环境变量」→ 在「Path」中点「新建」→ 粘贴你的 <code>bin</code> 目录完整路径 → 一路确定。然后重启本工具，点
                <strong>「使用系统 PATH」</strong> + <strong>「重新检测」</strong>，应显示「可用」。
              </li>
            </ul>
          </li>
        </ol>
        <p class="muted">提示：解压后不要把 ffmpeg.exe 单独移出 bin 目录，它依赖同目录下的其它文件。</p>
      </div>

      <!-- macOS 安装指南 -->
      <div v-if="guideOpen && isMacOS" class="install-guide">
        <h3>macOS 安装 ffmpeg</h3>
        <p class="muted" style="margin-top:0;"><em>AVS 仅 Windows 支持，macOS 必须使用带 subtitles/libass filter 的 ffmpeg-full。</em></p>
        <ul>
          <li>
            <strong>方法一（必需）</strong> · 用 Homebrew 安装 ffmpeg-full：
            <ol style="margin:6px 0 0; padding-left:20px;">
              <li>终端执行 <code>brew install ffmpeg-full</code></li>
              <li>
                <code>ffmpeg-full</code> 是 keg-only，不会自动覆盖普通 <code>ffmpeg</code>。安装后在本工具点「选择 ffmpeg」，选择：
                <div class="cmd-block">/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg</div>
                <div class="muted" style="margin-top:6px;">Intel Mac 通常是 <code>/usr/local/opt/ffmpeg-full/bin/ffmpeg</code>。</div>
              </li>
              <li>
                配置环境变量到 <code>~/.zprofile</code>（本工具读取此文件获取 PATH，写到 <code>~/.zshrc</code> 不生效）。根据 Mac 芯片选一条执行：
                <div class="muted" style="margin-top:6px;">Apple Silicon（M 系列芯片）：</div>
                <div class="cmd-block">echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' &gt;&gt; ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"</div>
                <div class="muted" style="margin-top:6px;">Intel Mac：</div>
                <div class="cmd-block">echo 'eval "$(/usr/local/bin/brew shellenv)"' &gt;&gt; ~/.zprofile
eval "$(/usr/local/bin/brew shellenv)"</div>
                <div class="muted" style="margin-top:6px;">不确定芯片？终端执行 <code>uname -m</code>：<code>arm64</code> 是 Apple Silicon，<code>x86_64</code> 是 Intel。</div>
                <div class="muted" style="margin-top:6px;">Homebrew 安装时若已自动写入 <code>~/.zprofile</code>，此步可跳过。</div>
              </li>
              <li>
                自检命令：
                <div class="cmd-block">/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg -hide_banner -filters | grep ' subtitles '</div>
                <div class="muted" style="margin-top:6px;">能看到 <code>subtitles V-&gt;V</code> 才能压制 ASS 字幕。不要用 <code>grep -E 'subtitles|ass'</code>，它会误匹配 allpass/bass/highpass。</div>
              </li>
            </ol>
          </li>
          <li>
            <strong>方法二</strong> · 不想装 Homebrew，从
            <a href="https://evermeet.cx/ffmpeg/" target="_blank" rel="noopener">evermeet.cx/ffmpeg</a>
            下载静态构建。请按页面说明选择与你 Mac 架构匹配的版本，下载后在本工具点「选择 ffmpeg」指向它。选完必须确认 <strong>subtitles/libass</strong> 状态为 ✓。
          </li>
        </ul>
        <h4>常见排障</h4>
        <ul>
          <li><code>No such filter: subtitles</code>：当前 ffmpeg 缺少 libass/subtitles，安装并选择 <code>ffmpeg-full</code>。</li>
          <li><code>Unable to open .../subtitle.ass</code>：任务临时字幕文件已清理，回到应用重新开始压制。</li>
          <li><code>Missing key frame...</code>：源 MP4 的 edit list 警告，通常不是失败原因。</li>
        </ul>
      </div>

      <!-- Linux 安装指南 -->
      <div v-if="guideOpen && isLinux" class="install-guide">
        <h3>Linux 安装 ffmpeg</h3>
        <p class="muted" style="margin-top:0;"><em>AVS 仅 Windows 支持，Linux 自动走 ffmpeg filter 模式。h264_videotoolbox 编码器不可用。</em></p>
        <ul>
          <li>
            <strong>Debian / Ubuntu</strong> · 终端执行
            <code>sudo apt update &amp;&amp; sudo apt install ffmpeg</code>，装好后本工具点「使用系统 PATH」+「重新检测」。
          </li>
          <li>
            <strong>Fedora / RHEL</strong> · 启用 RPM Fusion 后执行
            <code>sudo dnf install ffmpeg</code>。Arch 系直接
            <code>sudo pacman -S ffmpeg</code>。
          </li>
          <li>
            <strong>不想用包管理器</strong> · 从
            <a href="https://johnvansickle.com/ffmpeg/" target="_blank" rel="noopener">johnvansickle.com/ffmpeg</a>
            下载静态构建（含 ffmpeg + ffprobe，免装依赖）。解压后把可执行文件放到任意目录，在本工具点「选择 ffmpeg」指向它即可。
          </li>
        </ul>
        <p class="muted">提示：发行版仓库里的 ffmpeg 版本可能偏旧，缺编码器时优先用静态构建。</p>
      </div>
    </section>

    <!-- AVS 设置：仅 Windows 展示，仿 ffmpeg 设置面板风格 -->
    <section v-if="isWindows" class="panel">
      <div v-if="isAvsMocked" class="debug-banner">
        <strong>⚠ AVS 调试 mock 中</strong>
        <span>当前正在模拟：<code>{{ avsMockSummary }}</code>。</span>
        <button class="secondary" @click="clearAllAvsMocks">恢复真实检测</button>
      </div>

      <div class="panel-heading">
        <div>
          <h2>AVS 设置</h2>
          <p>启用 AVS 兼容模式需要 ffmpeg 启用 avisynth demuxer 且系统已装 AviSynth+。</p>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>状态</dt>
          <dd>
            <span
              class="status-pill"
              :class="avsStatus?.ffmpegDemuxerAvailable ? 'ok' : 'bad'"
              v-tooltip="'ffmpeg 是否启用 --enable-avisynth 构建'"
            >
              <span class="status-icon">{{ avsStatus?.ffmpegDemuxerAvailable ? '✓' : '✕' }}</span>
              <span>ffmpeg avisynth demuxer</span>
            </span>
            <span
              class="status-pill"
              :class="avsStatus?.avisynthInstalled ? 'ok' : 'bad'"
              style="margin-left:8px;"
              v-tooltip="'系统是否安装 AviSynth+ 运行环境'"
            >
              <span class="status-icon">{{ avsStatus?.avisynthInstalled ? '✓' : '✕' }}</span>
              <span>AviSynth+</span>
            </span>
          </dd>
        </div>
        <div><dt>AviSynth+ 版本</dt><dd>{{ avsStatus?.avisynthVersion ?? '—' }}</dd></div>
        <div><dt>安装目录</dt><dd>{{ avsStatus?.avisynthInstallPath ?? '—' }}</dd></div>
        <div><dt>AviSynth.dll</dt><dd>{{ avsStatus?.avisynthDllPath ?? '— 未在 System32/SysWOW64 找到' }}</dd></div>
      </dl>
      <p v-if="avsStatus && !avsStatus.available" class="notice" style="color:#a35000;">
        ⚠ {{ avsStatus.message ?? 'AVS 环境不可用，将无法启用 AVS 压制' }}
      </p>
      <div class="actions left">
        <button
          class="secondary"
          :class="{ 'is-busy': isBusy('avsRefresh') }"
          @click="withBusy('avsRefresh', refreshAvsStatus)"
        >重新检测</button>
        <button class="secondary" @click="avsGuideOpen = !avsGuideOpen">
          {{ avsGuideOpen ? '收起安装手册' : '没有 AviSynth+？查看安装手册' }}
        </button>
      </div>

      <div v-if="avsGuideOpen" class="install-guide">
        <h3>为什么需要 AVS 兼容模式？</h3>
        <p class="muted" style="margin-top:0;">
          ffmpeg 默认走 <strong>libass</strong> 渲染 ASS 字幕，标准 ASS 标签兼容良好，但
          <strong>VSFilterMod 扩展标签 libass 完全不支持</strong>，强行压制会直接丢特效。
          常见踩坑场景：
        </p>
        <ul style="margin-top:4px;">
          <li><code>$img(...)</code> —— VSFilterMod 的<strong>图片插入</strong>标签，常用于字幕里嵌入 logo / 装饰图。libass 直接当字符串渲染。</li>
          <li><code>\vc</code> —— vertical color，<strong>垂直渐变填充</strong>。libass 无此扩展。</li>
          <li><code>\fsvp</code> —— font scale variable percent，<strong>逐字缩放百分比</strong>，K-Pop 字幕组逐字卡拉模板的核心标签。libass 不识别。</li>
          <li><code>\fax</code> / <code>\fay</code>（字体 X/Y 倾斜）—— 综艺字幕组高频使用，libass 渲染结果与 VSFilterMod 差异较大。</li>
          <li>GDI 字体的 hinting / 描边细节（libass 用 freetype，外观存在差异）。</li>
        </ul>
        <p class="muted">
          AVS 兼容模式用 <strong>VSFilterMod 的 TextSubMod</strong> 渲染字幕，与原始 KMPlayer / PotPlayer 软解播放完全一致——
          <strong>看到的就是压出来的</strong>。
        </p>

        <h3 style="margin-top:14px;">启用步骤</h3>
        <ol>
          <li>
            <strong>装 AviSynth+ 运行环境</strong>：去
            <a href="https://github.com/AviSynth/AviSynthPlus/releases" target="_blank" rel="noopener">
              AviSynth+ Releases
            </a>
            下载最新稳定版 <code>AviSynthPlus_x.y.z_*-installer.exe</code>。
            <div class="muted" style="margin-top:4px;">
              <strong>⚠ 安装时必须勾选 <code>AviSynth+ (x64)</code></strong>，与 64 位 ffmpeg 匹配。默认安装界面只勾了 x64，注意不要误取消；
              x86 可不装。安装完成后脚本引擎会写入 <code>C:\Windows\System32\AviSynth.dll</code>，安装目录可自定义。
            </div>
          </li>
          <li>
            <strong>确认 ffmpeg 是 full 版</strong>：本工具要求 ffmpeg 构建时启用
            <code>--enable-avisynth</code>。
            <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.7z" target="_blank" rel="noopener">
              full 版
            </a>
            默认包含；essentials 版不包含。完成上面两步后点
            <strong>「重新检测」</strong>，两枚徽章都变 ✓ 即可在「压制参数」里勾选「AVS 兼容模式」。
          </li>
        </ol>
        <p class="muted">
          内置 <code>VSFilterMod.dll</code> + <code>LSMASHSource.dll</code> 已随本工具打包，无需手动配置；切勿把 essentials 版的 ffmpeg 当 full 版用，否则会报
          <code>avisynth: Could not initialize ...</code>。
        </p>
      </div>
    </section>

    <section class="panel">
      <div class="panel-heading">
        <div>
          <div class="update-title-row">
            <h2>应用更新</h2>
            <span class="current-version">当前版本 v{{ appVersion }}</span>
          </div>
        </div>
      </div>
      <div class="actions left">
        <button
          :class="{ 'is-busy': isBusy('checkUpdate') }"
          @click="withBusy('checkUpdate', checkUpdate)"
        >检查应用更新</button>
        <label v-if="appConfig" class="switch-row update-startup-toggle">
          <input
            type="checkbox"
            :checked="appConfig.checkUpdateOnStartup"
            @change="setStartupUpdateCheck(($event.target as HTMLInputElement).checked)"
          />
          <span class="switch" aria-hidden="true"></span>
          <span>启动时自动检查更新</span>
        </label>
      </div>
      <div v-if="updateMessage" class="update-result" :class="`update-result-${updateState}`">
        <span class="update-result-icon">
          {{ updateState === 'error' ? '!' : updateState === 'success' ? '✓' : 'i' }}
        </span>
        <div>
          <strong>
            {{ updateResultTitle }}
          </strong>
          <p>{{ updateMessage }}</p>
          <p v-if="updateInfo?.notes" class="update-notes">
            <span class="update-notes-title">{{ updateNotesTitle }}</span>
            {{ updateInfo.notes }}
          </p>
          <p v-if="updateState === 'success' && updateInfo?.available" class="update-notes">
            请在 GitHub Releases 下载新版安装包，关闭当前应用后安装。
            <a
              class="button-link update-download-link"
              :href="updateReleaseUrl"
              target="_blank"
              rel="noopener noreferrer"
            >前往 GitHub 下载 v{{ availableUpdateVersion }}</a>
          </p>
        </div>
      </div>
    </section>

    <section class="panel">
      <div class="panel-heading">
        <div>
          <h2>关于</h2>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>作者</dt>
          <dd>
            <a
              class="author-link"
              href="https://github.com/Chinshry"
              target="_blank"
              rel="noopener noreferrer"
              v-tooltip="'打开作者 GitHub 主页'"
            >
              <img
                v-if="!avatarFailed"
                :src="authorAvatarUrl"
                class="author-avatar"
                alt="Chinshry 的 GitHub 头像"
                width="24"
                height="24"
                @error="onAvatarError"
              />
              <span v-else class="author-avatar author-avatar-fallback" aria-hidden="true">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                  <circle cx="12" cy="7" r="4" />
                </svg>
              </span>
              <span class="author-name">Chinshry</span>
            </a>
          </dd>
        </div>
        <div>
          <dt>项目仓库</dt>
          <dd>
            <a
              class="repo-link"
              href="https://github.com/Chinshry/CSubtitleWorkstation"
              target="_blank"
              rel="noopener noreferrer"
              v-tooltip="'在 GitHub 查看源码 / 提 Issue'"
            >
              <svg class="repo-link-icon" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                <path fill="currentColor" fill-rule="evenodd" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
              </svg>
              GitHub 仓库
            </a>
          </dd>
        </div>
        <div>
          <dt>许可证</dt>
          <dd>
            <a
              class="license-chip"
              href="https://github.com/Chinshry/CSubtitleWorkstation/blob/master/LICENSE"
              target="_blank"
              rel="noopener noreferrer"
              v-tooltip="'查看完整许可证文本'"
            >GNU GPL v3.0</a>
          </dd>
        </div>
      </dl>
    </section>

    <!-- 调试面板：仅开发构建可见 -->
    <section v-if="isDev" class="panel debug-panel">
      <div class="panel-heading">
        <div>
          <h2>🛠 调试面板</h2>
          <p>
            仅 dev 构建可见。下方开关可强制改变界面呈现，便于测试不同平台 / 缺失场景。
            真实平台：<code>{{ nativePlatformLabel }}</code>。
          </p>
        </div>
        <button class="secondary" @click="debugPanelOpen = !debugPanelOpen">
          {{ debugPanelOpen ? '收起' : '展开' }}
        </button>
      </div>
      <div v-if="debugPanelOpen" class="debug-body">
        <div class="debug-group">
          <h4>平台覆盖</h4>
          <div class="debug-options">
            <label v-for="opt in overrideOptions" :key="String(opt.value)" class="debug-radio">
              <input
                type="radio"
                name="platform-override"
                :value="opt.value === null ? 'native' : opt.value"
                v-model="overrideModel"
              />
              <span>{{ opt.label }}</span>
            </label>
          </div>
        </div>
        <div class="debug-group">
          <h4>ffmpeg / ffprobe / 滤镜缺失模拟</h4>
          <p class="muted">不会影响后端真实状态，仅用于演练 UI 反应。三个独立勾选，可叠加。</p>
          <div class="debug-options">
            <label class="debug-check">
              <input type="checkbox" v-model="ffmpegMissingModel" />
              <span>
                模拟 ffmpeg 缺失
                <em class="muted">— 首页红色横幅 + 设置页 ffmpeg/ffprobe 双 ✕，开始压制按钮变灰</em>
              </span>
            </label>
            <label class="debug-check">
              <input type="checkbox" v-model="ffprobeMissingModel" />
              <span>
                模拟 ffprobe 缺失
                <em class="muted">— 设置页 ffprobe ✕；视频信息卡隐藏「帧模式」「总帧数」（仅 ffprobe 才能精准给出）</em>
              </span>
            </label>
            <label class="debug-check">
              <input type="checkbox" v-model="subtitleFilterMissingModel" />
              <span>
                模拟 subtitles/libass 缺失
                <em class="muted">— 设置页 subtitles/libass ✕；首页提示 ffmpeg 功能不完整，开始压制按钮变灰</em>
              </span>
            </label>
          </div>
        </div>
        <div class="debug-group">
          <h4>AVS 缺失模拟（仅 Windows 生效）</h4>
          <p class="muted">不影响后端检测；用于演练 AVS 设置面板与压制表单在依赖缺失时的反应。</p>
          <div class="debug-options">
            <label class="debug-check">
              <input type="checkbox" v-model="avisynthMissingModel" />
              <span>
                模拟 AviSynth+ 缺失
                <em class="muted">— AVS 面板 AviSynth+ ✕，压制表单 AVS 开关禁用并 tooltip 提示</em>
              </span>
            </label>
            <label class="debug-check">
              <input type="checkbox" v-model="avsDemuxerMissingModel" />
              <span>
                模拟 ffmpeg avisynth demuxer 缺失
                <em class="muted">— AVS 面板 demuxer ✕，说明用户用的是 essentials 版 ffmpeg</em>
              </span>
            </label>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
.debug-banner {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  padding: 8px 14px;
  margin-bottom: 12px;
  background: #fff8d6;
  border: 1px solid #e0c870;
  border-radius: 6px;
  color: #6a5300;
  font-size: 13px;
}
.debug-banner code {
  background: rgba(0, 0, 0, 0.06);
  padding: 1px 6px;
  border-radius: 3px;
}
.debug-banner button {
  margin-left: auto;
}

.debug-panel {
  border: 1px dashed #b89aff;
  background: #f7f2ff;
}
.debug-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin-top: 8px;
}
.debug-group h4 {
  margin: 0 0 6px 0;
  font-size: 13px;
  color: #5a3da6;
}
.debug-group .muted {
  margin: 0 0 6px 0;
  font-size: 12px;
  color: #7a7a7a;
}
.debug-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.debug-radio {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 14px;
}
.debug-check {
  display: inline-flex;
  align-items: flex-start;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
}
.debug-check input[type="checkbox"] {
  margin-top: 3px;
}
.debug-radio em,
.debug-check em {
  font-style: normal;
  font-size: 12px;
  color: #7a7a7a;
}
</style>
