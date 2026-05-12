<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { setFfmpegPath, resetFfmpegToSystem } from '../api/ffmpeg'
import { checkAppUpdate, getCurrentAppVersion } from '../api/updater'
import {
  ffmpegStatus,
  initFfmpegStatus,
  isFfmpegMissingMocked,
  isFfmpegMocked,
  isFfprobeMissingMocked,
  clearAllFfmpegMocks,
  refreshFfmpegStatus,
  setFfmpegMissingMock,
  setFfprobeMissingMock,
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

const status = ffmpegStatus
const appVersion = ref('')
const updateMessage = ref('')
const guideOpen = ref(false)
const avsGuideOpen = ref(false)
const debugPanelOpen = ref(false)

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

const mockSummary = computed(() => {
  const parts: string[] = []
  if (isFfmpegMissingMocked.value) parts.push('ffmpeg 缺失')
  if (isFfprobeMissingMocked.value) parts.push('ffprobe 缺失')
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
  const version = await checkAppUpdate()
  updateMessage.value = version ? `发现新版本：${version}` : '当前已是最新版本'
}

onMounted(async () => {
  appVersion.value = await getCurrentAppVersion()
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
            <span class="status-pill" :class="status?.available ? 'ok' : 'bad'" :title="status?.ffmpegPath ?? 'ffmpeg 未找到'">
              <span class="status-icon">{{ status?.available ? '✓' : '✕' }}</span>
              <span>ffmpeg</span>
            </span>
            <span class="status-pill" :class="status?.ffprobePath ? 'ok' : 'bad'" style="margin-left:8px;" :title="status?.ffprobePath ?? 'ffprobe 未找到（影响视频信息精度，无法检测 CFR/VFR、总帧数）'">
              <span class="status-icon">{{ status?.ffprobePath ? '✓' : '✕' }}</span>
              <span>ffprobe</span>
            </span>
          </dd>
        </div>
        <div><dt>来源</dt><dd>{{ sourceText }}</dd></div>
        <div><dt>ffmpeg</dt><dd>{{ status?.ffmpegPath ?? '—' }}</dd></div>
        <div><dt>ffprobe</dt><dd>{{ status?.ffprobePath ?? '— 未找到（应与 ffmpeg 同目录）' }}</dd></div>
        <div><dt>版本</dt><dd>{{ status?.ffmpegVersion ?? '—' }}</dd></div>
      </dl>
      <div class="actions left">
        <button @click="chooseFfmpeg">选择 ffmpeg</button>
        <button class="secondary" @click="useSystemPath">使用系统 PATH</button>
        <button class="secondary" @click="refresh">重新检测</button>
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
        <p class="muted" style="margin-top:0;"><em>AVS 仅 Windows 支持，macOS 自动走 ffmpeg filter 模式。</em></p>
        <ul>
          <li>
            <strong>方法一（推荐）</strong> · 用 Homebrew：终端执行
            <code>brew install ffmpeg</code>，装好后本工具点「使用系统 PATH」+「重新检测」。
          </li>
          <li>
            <strong>方法二</strong> · 不想装 Homebrew，从
            <a href="https://evermeet.cx/ffmpeg/" target="_blank" rel="noopener">evermeet.cx/ffmpeg</a>
            下 universal binary（含 Intel + Apple Silicon），是个独立的 <code>ffmpeg</code> 文件；放到任意目录后在本工具点「选择 ffmpeg」指向它。
          </li>
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
              title="ffmpeg 是否启用 --enable-avisynth 构建"
            >
              <span class="status-icon">{{ avsStatus?.ffmpegDemuxerAvailable ? '✓' : '✕' }}</span>
              <span>ffmpeg avisynth demuxer</span>
            </span>
            <span
              class="status-pill"
              :class="avsStatus?.avisynthInstalled ? 'ok' : 'bad'"
              style="margin-left:8px;"
              title="系统是否安装 AviSynth+ 运行环境"
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
        <button class="secondary" @click="refreshAvsStatus">重新检测</button>
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
          <h2>应用更新</h2>
          <p>应用本体更新和 ffmpeg 版本检测是两套独立机制。</p>
        </div>
        <span class="badge">v{{ appVersion }}</span>
      </div>
      <div class="actions left">
        <button @click="checkUpdate">检查应用更新</button>
      </div>
      <p class="notice" v-if="updateMessage">{{ updateMessage }}</p>
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
          <h4>ffmpeg / ffprobe 缺失模拟</h4>
          <p class="muted">不会影响后端真实状态，仅用于演练 UI 反应。两个独立勾选，可叠加。</p>
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
