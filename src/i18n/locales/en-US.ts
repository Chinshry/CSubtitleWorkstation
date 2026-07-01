export default {
  app: {
    name: 'CC Subtitle Workstation',
    subtitle: 'Subtitle WorkStation',
    drop: {
      unsupportedActiveTool: 'The current tool does not support this file type.',
      unsupportedHome: 'The compression page only accepts video or subtitle files.',
      unsupportedFile: 'This file type is not supported.'
    }
  },
  common: {
    selectPlaceholder: 'Select...',
    choose: 'Choose',
    unset: 'Not set',
    copy: 'Copy',
    edit: 'Edit',
    clear: 'Clear',
    close: 'Close',
    create: 'New',
    rename: 'Rename',
    delete: 'Delete',
    export: 'Export',
    save: 'Save',
    cancel: 'Cancel',
    confirm: 'OK',
    unselected: 'Unselected',
    noContent: 'No content',
    copied: 'Copied',
    copyFailed: 'Copy failed',
    processing: 'Processing...',
    showCommandPreview: 'Show Command Preview',
    hideCommandPreview: 'Hide Command Preview',
    commandPreviewDisabledTip: 'The command is generated after input and output are selected.',
    runDisabledTip: 'Complete the task configuration first.'
  },
  nav: {
    home: 'Encode',
    presets: 'Presets',
    tools: 'Tools',
    settings: 'Settings',
    expand: 'Expand',
    collapse: 'Collapse'
  },
  titleBar: {
    minimize: 'Minimize',
    maximize: 'Maximize',
    restore: 'Restore down',
    close: 'Close'
  },
  toast: {
    regionLabel: 'Notifications'
  },
  settings: {
    language: {
      title: 'Interface Language',
      description: 'Switch built-in interface copy. User content, logs, and external release notes stay in their original language.',
      label: 'Language',
      system: 'System Default',
      zhCn: '中文',
      enUs: 'English'
    },
    update: {
      title: 'App Updates',
      currentVersion: 'Current version v{version}',
      check: 'Check for Updates',
      checkOnStartup: 'Check for updates on startup',
      download: 'Download on GitHub',
      notesTitle: 'Release Notes',
      publishedAt: 'Published {date}'
    },
    about: {
      title: 'About',
      author: 'Author',
      authorTip: 'Open author GitHub profile',
      authorAvatarAlt: "Chinshry's GitHub avatar",
      repository: 'Repository',
      repositoryTip: 'View source or open an issue on GitHub',
      githubRepository: 'GitHub Repository',
      license: 'License',
      licenseTip: 'View full license text'
    },
    ffmpeg: {
      title: 'ffmpeg Settings',
      status: 'Status',
      source: 'Source',
      version: 'Version',
      sourceSystemPath: 'System PATH',
      sourceCustomPath: 'Custom path',
      sourceNotFound: 'Not found',
      ffmpegMissing: 'ffmpeg not found',
      ffprobeMissingTip: 'ffprobe not found. Video metadata precision is reduced, and CFR/VFR or total frame count cannot be detected.',
      subtitleFilterAvailable: 'ASS subtitle burn-in is available',
      subtitleFilterMissing: 'Missing subtitles/libass filter. ASS subtitle burn-in is unavailable.',
      ffprobeNotFoundSameDir: '- Not found (should be in the same folder as ffmpeg)',
      choose: 'Choose ffmpeg',
      useSystemPath: 'Use System PATH',
      refresh: 'Recheck',
      hideGuide: 'Hide Install Guide',
      showGuide: 'No ffmpeg? View install guide',
      checkingTitle: 'Checking ffmpeg Environment',
      checkingDescription: 'Checking ffmpeg / ffprobe / subtitles/libass. Please wait.',
      dialogChoose: 'Choose ffmpeg executable',
      guides: {
        windows: `<h3>Install ffmpeg on Windows</h3><ol><li><strong>Install an extractor</strong> if needed: the recommended ffmpeg package is a <code>.7z</code> archive, which Windows may not extract by default. Install <a href="https://www.7-zip.org/" target="_blank" rel="noopener">7-Zip</a>.</li><li><strong>Download ffmpeg</strong>: open <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.7z" target="_blank" rel="noopener">ffmpeg-release-full.7z</a>. This Gyan.dev full build includes ffmpeg / ffprobe, third-party libraries, and <strong>AviSynth+ support</strong>, which is required for the full feature set.<div class="muted" style="margin-top:4px;">If you only need normal encoding without AVS, the smaller <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip" target="_blank" rel="noopener">essentials.zip</a> also works, but AVS compatibility mode will be unavailable.</div></li><li><strong>Extract it</strong>: right-click the archive, extract it with 7-Zip, then move the extracted folder to a stable location. The <code>bin</code> folder contains <code>ffmpeg.exe</code> and <code>ffprobe.exe</code>.</li><li><strong>Point the app to it</strong>:<ul><li><em>Simple path</em> · Click <strong>Choose ffmpeg</strong> in this app and select <code>bin\\ffmpeg.exe</code>. The app will look for <code>ffprobe.exe</code> in the same folder.</li><li><em>Advanced path</em> · Add the extracted <code>bin</code> folder to the system <strong>PATH</strong>, restart this app, then click <strong>Use System PATH</strong> and <strong>Recheck</strong>.</li></ul></li></ol><p class="muted">Tip: do not move ffmpeg.exe out of the bin folder by itself; it depends on files in the same folder.</p>`,
        macos: `<h3>Install ffmpeg on macOS</h3><p class="muted" style="margin-top:0;"><em>AVS is Windows-only. On macOS, use an ffmpeg-full build with the subtitles/libass filter.</em></p><ul><li><strong>Method 1 (recommended)</strong> · Install ffmpeg-full with Homebrew:<ol style="margin:6px 0 0; padding-left:20px;"><li>Run <code>brew install ffmpeg-full</code> in Terminal.</li><li><code>ffmpeg-full</code> is keg-only and does not replace the regular <code>ffmpeg</code>. After installation, click Choose ffmpeg in this app and select:<div class="cmd-block">/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg</div><div class="muted" style="margin-top:6px;">Intel Macs usually use <code>/usr/local/opt/ffmpeg-full/bin/ffmpeg</code>.</div></li><li>Configure PATH in <code>~/.zprofile</code>. This app reads that file for PATH; <code>~/.zshrc</code> is not enough. Use the command for your Mac:<div class="muted" style="margin-top:6px;">Apple Silicon:</div><div class="cmd-block">echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' &gt;&gt; ~/.zprofile\neval "$(/opt/homebrew/bin/brew shellenv)"</div><div class="muted" style="margin-top:6px;">Intel Mac:</div><div class="cmd-block">echo 'eval "$(/usr/local/bin/brew shellenv)"' &gt;&gt; ~/.zprofile\neval "$(/usr/local/bin/brew shellenv)"</div><div class="muted" style="margin-top:6px;">Not sure which chip you have? Run <code>uname -m</code>: <code>arm64</code> is Apple Silicon, <code>x86_64</code> is Intel.</div></li><li>Self-check command:<div class="cmd-block">/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg -hide_banner -filters | grep ' subtitles '</div><div class="muted" style="margin-top:6px;">You must see <code>subtitles V-&gt;V</code> to burn ASS subtitles. Avoid <code>grep -E 'subtitles|ass'</code>; it can match unrelated filters such as allpass or bass.</div></li></ol></li><li><strong>Method 2</strong> · If you do not want Homebrew, download a static build from <a href="https://evermeet.cx/ffmpeg/" target="_blank" rel="noopener">evermeet.cx/ffmpeg</a>. Choose a build matching your Mac architecture, then select it with Choose ffmpeg. Confirm subtitles/libass is available afterward.</li></ul><h4>Troubleshooting</h4><ul><li><code>No such filter: subtitles</code>: the selected ffmpeg lacks libass/subtitles. Install and select <code>ffmpeg-full</code>.</li><li><code>Unable to open .../subtitle.ass</code>: the temporary subtitle file was cleaned up. Start the encode again from the app.</li><li><code>Missing key frame...</code>: usually an MP4 edit-list warning, not the failure cause.</li></ul>`,
        linux: `<h3>Install ffmpeg on Linux</h3><p class="muted" style="margin-top:0;"><em>AVS is Windows-only. Linux uses ffmpeg filter mode automatically, and h264_videotoolbox is unavailable.</em></p><ul><li><strong>Debian / Ubuntu</strong> · Run <code>sudo apt update &amp;&amp; sudo apt install ffmpeg</code>, then click Use System PATH and Recheck.</li><li><strong>Fedora / RHEL</strong> · Enable RPM Fusion, then run <code>sudo dnf install ffmpeg</code>. On Arch, run <code>sudo pacman -S ffmpeg</code>.</li><li><strong>Without a package manager</strong> · Download a static build from <a href="https://johnvansickle.com/ffmpeg/" target="_blank" rel="noopener">johnvansickle.com/ffmpeg</a>. It includes ffmpeg and ffprobe. Extract it anywhere and select the executable with Choose ffmpeg.</li></ul><p class="muted">Tip: distro repository ffmpeg builds can be old. Use a static build first when encoders are missing.</p>`
      }
    },
    avs: {
      title: 'AVS Settings',
      description: 'AVS compatibility mode requires ffmpeg with the avisynth demuxer and AviSynth+ installed on the system.',
      status: 'Status',
      demuxerTip: 'Whether ffmpeg was built with --enable-avisynth',
      avisynthTip: 'Whether the AviSynth+ runtime is installed',
      avisynthVersion: 'AviSynth+ Version',
      installPath: 'Install Path',
      dllNotFound: '- Not found in System32/SysWOW64',
      unavailable: 'The AVS environment is unavailable, so AVS encoding cannot be enabled',
      showGuide: 'No AviSynth+? View install guide',
      checkingTitle: 'Checking AVS Environment',
      checkingDescription: 'Checking ffmpeg avisynth demuxer and AviSynth+. Please wait.',
      guide: `<h3>Why AVS compatibility mode is needed</h3><p class="muted" style="margin-top:0;">ffmpeg normally renders ASS subtitles through <strong>libass</strong>. Standard ASS tags work well, but <strong>VSFilterMod extension tags are not supported by libass</strong>, so forcing normal burn-in can drop effects. Common cases include:</p><ul style="margin-top:4px;"><li><code>$img(...)</code>: VSFilterMod image insertion, often used for subtitle logos or decorative images. libass renders it as plain text.</li><li><code>\\vc</code>: vertical color gradient fill. libass does not support this extension.</li><li><code>\\fsvp</code>: font scale variable percent, a core tag in some per-character karaoke templates. libass does not recognize it.</li><li><code>\\fax</code> / <code>\\fay</code>: font X/Y shear, frequently used in variety-show subtitles, with visible differences between libass and VSFilterMod.</li><li>GDI font hinting and outline details can also differ because libass uses freetype.</li></ul><p class="muted">AVS compatibility mode renders subtitles with <strong>VSFilterMod TextSubMod</strong>, matching the original KMPlayer / PotPlayer software-rendered result more closely.</p><h3 style="margin-top:14px;">Setup steps</h3><ol><li><strong>Install the AviSynth+ runtime</strong>: download the latest stable <code>AviSynthPlus_x.y.z_*-installer.exe</code> from <a href="https://github.com/AviSynth/AviSynthPlus/releases" target="_blank" rel="noopener">AviSynth+ Releases</a>.<div class="muted" style="margin-top:4px;"><strong>Make sure <code>AviSynth+ (x64)</code> is selected</strong> so it matches 64-bit ffmpeg. x86 is optional. After installation, the script engine writes <code>C:\\Windows\\System32\\AviSynth.dll</code>.</div></li><li><strong>Confirm ffmpeg is a full build</strong>: this app requires ffmpeg built with <code>--enable-avisynth</code>. The <a href="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.7z" target="_blank" rel="noopener">Gyan.dev full build</a> includes it by default; essentials does not. After both steps, click <strong>Recheck</strong> and the AVS badges should turn ✓.</li></ol><p class="muted">Bundled <code>VSFilterMod.dll</code> and <code>LSMASHSource.dll</code> are included with this app. No manual setup is needed. Do not use the essentials ffmpeg build as a full build, or AVS initialization can fail.</p>`
    },
    lav: {
      title: 'VP9 DirectShow Decoder',
      description: 'Only VP9 video in AVS fallback needs this. Normal AVS subtitle encoding does not depend on LAV Filters.',
      ready: 'LAV Filters ready',
      required: '64-bit LAV Filters required',
      x64Components: 'x64 Components',
      detected: 'Detected',
      notDetected: 'Not detected',
      directshow: 'DirectShow Registration',
      directshowRegistered: 'Splitter + Video Decoder registered',
      directshowMissing: 'Full registration not detected',
      showGuide: 'No LAV Filters? View install guide',
      checkingTitle: 'Checking VP9 Decode Environment',
      checkingDescription: 'Checking 64-bit LAV Filters and DirectShow registration. Please wait.',
      guide: `<h3>When LAV Filters are needed</h3><p class="muted" style="margin-top:0;">Only VP9 video in AVS fallback mode needs LAV Filters. It provides a 64-bit DirectShow MKV splitter and VP9 video decoder for <code>DirectShowSource</code>.</p><h3 style="margin-top:14px;">Setup steps</h3><ol><li>Open <a href="https://github.com/Nevcairiel/LAVFilters/releases" target="_blank" rel="noopener">LAV Filters Releases</a> and download the installer, such as <code>LAVFilters-x.y.z-Installer.exe</code>.</li><li>During installation, confirm the <strong>64-bit Splitter</strong> and <strong>64-bit Video Decoder</strong> components are included. Restart this app afterward and click <strong>Recheck</strong>.</li></ol>`
    },
    debug: {
      title: 'Debug Panel',
      description: 'Visible only in dev builds. These switches force UI states for platform and missing-dependency testing.',
      nativePlatform: 'Native platform: ',
      otherPlatform: 'Other Platform',
      followSystemPlatform: 'Follow System ({platform})',
      platformOverrideTitle: 'Platform debug override active',
      platformOverridePrefix: 'Rendering as',
      platformOverrideMiddle: 'while native platform is',
      restoreSystem: 'Restore System Default',
      restoreRealDetection: 'Restore Real Detection',
      ffmpegMockTitle: 'ffmpeg debug mock active',
      avsMockTitle: 'AVS debug mock active',
      mocking: 'Currently mocking: ',
      platformOverride: 'Platform Override',
      ffmpegMocks: 'ffmpeg / ffprobe / filter missing mocks',
      ffmpegMocksDescription: 'These do not affect backend state. They only exercise UI responses and can be combined.',
      mockFfmpegMissing: 'Mock ffmpeg missing',
      mockFfmpegMissingShort: 'ffmpeg missing',
      mockFfmpegMissingHint: '- Home banner turns red, Settings shows ffmpeg/ffprobe as missing, and Start Encoding is disabled',
      mockFfprobeMissing: 'Mock ffprobe missing',
      mockFfprobeMissingShort: 'ffprobe missing',
      mockFfprobeMissingHint: '- Settings marks ffprobe missing; Video Info hides frame mode and total frames',
      mockSubtitleFilterMissing: 'Mock subtitles/libass missing',
      mockSubtitleFilterMissingShort: 'subtitles/libass missing',
      mockSubtitleFilterMissingHint: '- Settings marks subtitles/libass missing and the Encode page disables start for incomplete ffmpeg features',
      avsMocks: 'AVS missing mocks (Windows only)',
      avsMocksDescription: 'These do not affect backend detection. They exercise AVS Settings and Encode parameter behavior when dependencies are missing.',
      mockAvisynthMissing: 'Mock AviSynth+ missing',
      mockAvisynthMissingShort: 'AviSynth+ missing',
      mockAvisynthMissingHint: '- AVS panel marks AviSynth+ missing and the Encode form disables the AVS switch',
      mockAvsDemuxerMissing: 'Mock ffmpeg avisynth demuxer missing',
      mockAvsDemuxerMissingShort: 'ffmpeg avisynth demuxer missing',
      mockAvsDemuxerMissingHint: '- AVS panel marks the demuxer missing, usually indicating an essentials ffmpeg build',
      lavMocks: 'VP9 DirectShow decoder mock (Windows only)',
      lavMocksDescription: 'Exercises the Settings panel state when VP9 fallback dependencies are missing.',
      mockLavFiltersMissing: 'Mock LAV Filters missing',
      mockLavFiltersMissingShort: 'LAV Filters missing',
      mockLavFiltersMissingHint: '- VP9 DirectShow Decoder panel shows that 64-bit LAV Filters are required'
    }
  },
  update: {
    message: {
      connecting: 'Connecting to the update server...',
      available: 'New version available: {version}',
      latest: 'You are already on the latest version'
    },
    error: {
      manifestUnavailable: 'Update check failed: the update manifest is unreachable. Confirm GitHub Pages is enabled and docs/updates/latest.json has been committed and pushed.',
      generic: 'Update check failed: {message}',
      serverStatus: 'Update server returned {status}',
      missingVersion: 'Update manifest is missing the version field'
    }
  },
  debugMock: {
    ffmpegMissing: '[Debug] Mock ffmpeg not found',
    ffprobeMissing: '[Debug] Mock ffprobe missing (only affects video metadata precision)',
    subtitleFilterMissing: '[Debug] Mock subtitles/libass filter missing',
    avs: '[Debug] Mock {parts}'
  },
  diagnostics: {
    dropNoPaths: 'WARN: drop event has no paths. Confirm tauri.conf.json has dragDropEnabled=true and restart tauri dev.',
    logoLayoutSavedDefault: 'LOGO layout saved as default configuration',
    logoLayoutSaveFailed: 'Failed to save LOGO layout: {message}',
    logoConfigSaved: 'LOGO configuration saved: {path}'
  },
  logPanel: {
    defaultTitle: 'Encoding Progress',
    defaultIdleTitle: 'Encoding Not Started',
    defaultIdleTip: 'Configure the parameters, then click Start Encoding above.',
    copyAllTip: 'Copy all logs',
    status: {
      running: 'Running',
      cancelling: 'Cancelling...',
      cancelled: 'Cancelled',
      completed: 'Completed',
      failed: 'Failed',
      idle: 'Waiting'
    },
    elapsed: 'Elapsed',
    remaining: 'Remaining',
    estimated: 'Estimate',
    duration: 'Duration',
    size: 'Size',
    speed: 'Speed',
    bitrate: 'Bitrate',
    estimatedSize: '/ est. {size}',
    noLogs: 'No logs yet',
    elapsedTip: 'Wall-clock time elapsed since the task started.',
    remainingTip: 'Remaining = (total duration - processed duration) / current speed.',
    estimatedTip: 'Estimated total time = elapsed + remaining, based on the current smoothed speed.'
  },
  commandPreview: {
    title: 'Command Preview',
    localWindows: 'Local Windows',
    localPosix: 'Local POSIX',
    nativeFormatTip: 'Uses the command format for {platform} by default.',
    terminalDialect: 'Select terminal dialect',
    options: {
      windows: 'Windows cmd/PowerShell',
      posix: 'POSIX bash/zsh',
      raw: 'Raw array join, no escaping'
    },
    restoreNative: 'Use Local',
    copyCommandTip: 'Copy full command',
    noCommand: 'No command',
    emptyTip: 'The command is generated after the video path is filled.',
    hints: {
      raw: 'Raw array join. Paths with spaces or special characters cannot be pasted directly into a terminal.',
      windows: 'Quoted for Windows. Paste into cmd, PowerShell, or Windows Terminal.',
      posix: 'Quoted for POSIX. Paste into bash, zsh, Linux, or macOS Terminal.'
    }
  },
  ffmpegStatus: {
    title: 'ffmpeg Status',
    description: 'The app only detects and calls the local ffmpeg. It does not bundle ffmpeg.',
    checking: 'Checking',
    available: 'Available',
    unavailable: 'Unavailable',
    source: 'Source',
    path: 'Path',
    version: 'Version',
    message: 'Message'
  },
  tools: {
    sidebarLabel: 'Tool Directory',
    groupListLabel: 'Tool Groups',
    groupToolsLabel: '{group} tools',
    empty: 'Coming Soon',
    groups: {
      text: {
        name: 'Text Processing',
        description: 'Lightweight subtitle text and dictionary tools.'
      },
      format: {
        name: 'Format Conversion',
        description: 'Subtitle and container format conversion entries.'
      },
      media: {
        name: 'Media Processing',
        description: 'Auxiliary media operations without re-encoding.'
      }
    },
    items: {
      ccSubtitle: {
        name: 'CC Subtitle Organizer',
        description: 'Organize CC subtitles by splitting [] text into styled screen-text lines and the rest into dialogue lines. Supports importing ASS / SSA / SRT / VTT, reading reference ASS styles, and exporting ASS.'
      },
      textConversion: {
        name: 'Simplified/Traditional Conversion',
        description: 'Convert Simplified and Traditional Chinese with zhconv. Custom dictionaries protect and replace specified terms first, suitable for subtitles and batch text processing.'
      },
      proofread: {
        name: 'Subtitle Proofreading',
        description: 'Use jieba-rs segmentation and POS tagging to check likely 的 / 地 / 得 misuse. Custom dictionaries help enforce names, artist names, and fixed translations.'
      },
      subtitleFormat: {
        name: 'Subtitle Format Conversion',
        description: 'Convert between ASS / SSA / SRT / VTT. TTML can be used as input and converted to SRT.'
      },
      mediaRemux: {
        name: 'Video to MP4',
        description: 'Remux common video containers to MP4. Audio and video streams are copied by default without re-encoding. TS / M2TS / MTS automatically normalize AAC bitstream headers.'
      },
      mediaConcatTs: {
        name: 'Merge TS Segments',
        description: 'Merge TS / M2TS / MTS segments in file order and output MP4 or TS. MP4 output normalizes AAC bitstream headers without re-encoding.'
      },
      mediaMergeAv: {
        name: 'Merge Audio and Video',
        description: 'Keep the video track and merge a separate audio source into MP4. Useful for replacing or adding an external audio track.'
      },
      mediaCover: {
        name: 'Add Cover Art',
        description: 'Write JPG / PNG cover art into MP4 while copying original video and audio streams without re-encoding.'
      }
    },
    ccRule: {
      label: 'Rules',
      title: 'CC Subtitle Organizer Rules',
      command: 'Read styles → Select dialogue/screen styles → Import subtitle',
      body: 'Organizes Web CC subtitles into an ASS structure suitable for further editing in Aegisub.',
      items: {
        readStyle: 'Read a reference ASS first and parse [V4+ Styles]; dialogue and screen-text styles must be selected manually.',
        convertInput: 'SRT / VTT input is converted to ASS output; ASS / SSA input processes existing Dialogue lines.',
        bracketText: 'For [bracket tags], text inside [] is stripped of brackets and uses the screen-text style.',
        dialogAfterBracket: 'Dialogue after a bracket tag is moved into a separate line using the dialogue style.',
        plainDialog: 'Plain dialogue without bracket tags uses the dialogue style for the whole line.',
        cleanText: 'Dialogue text only normalizes \\N line breaks and extra spaces.',
        dictionary: 'When the custom dictionary is enabled, names or fixed spellings are replaced according to dictionary rules.'
      }
    }
  },
  ccSubtitle: {
    dropOverlay: 'Release to read ASS / SSA / SRT / VTT subtitles',
    input: 'Input',
    result: 'Result',
    charCount: '{count} chars',
    previewLimit: 'Previewing first {count} chars',
    previewTruncated: '... Preview omitted {count} chars. Copy and export still use the full content.',
    inputPlaceholder: 'Drop an ASS / SSA / SRT / VTT subtitle file here to preview its content',
    resizeLabel: 'Resize input and result panes',
    organizing: 'Organizing...',
    outputSuffix: '_cc_organized',
    confirmDeleteProfile: 'Delete style profile "{name}"?',
    status: {
      needStyle: 'Read a reference ASS first, then select dialogue and screen-text styles.',
      needStyleBeforeImport: 'Read a reference ASS and select dialogue and screen-text styles before importing the subtitle to organize.',
      noStylesParsed: 'No styles were parsed from [V4+ Styles] in the reference ASS.',
      profileCreated: 'Created style profile "{name}". Select dialogue and screen-text styles.',
      readingSubtitle: 'Reading subtitle file...',
      exported: 'Exported: {path}'
    },
    toast: {
      organizeFailed: 'CC subtitle organize failed',
      noStylesParsed: 'No styles parsed',
      readStyleFailed: 'Failed to read styles',
      profileCreated: 'Style profile created',
      needStyle: 'Read styles first',
      readSubtitleFailed: 'Failed to read subtitle',
      exported: 'Exported',
      exportFailed: 'Export failed'
    },
    dialog: {
      exportTitle: 'Export CC Subtitle Organizer Result'
    },
    dictionary: {
      title: 'Replacement Dictionary',
      modalTitle: 'Custom Dictionary',
      description: 'Maintain name rules for CC speakers and dialogue. Matching text is replaced with the standard spelling during organization.',
      targetLabel: 'Standard Spelling',
      patternLabel: 'Match Rule (regex supported)',
      targetPlaceholder: 'Example Name',
      patternPlaceholder: 'e.g. (?i)EXAMPLE\\s*NAME',
      rawPlaceholder: '"Example Name" = "(?i)EXAMPLE\\s*NAME"',
      ariaLabel: 'CC subtitle custom dictionary',
      summary: '{count} rule(s) · {status}',
      enabled: 'Enabled',
      disabled: 'Disabled',
      enable: 'Enable'
    },
    style: {
      title: 'Style Profiles',
      description: 'Save reference ASS templates and default dialogue/screen-text styles for different subtitle groups.',
      noProfile: 'No style profile configured',
      chooseStyles: 'Select dialogue and screen-text styles',
      summary: 'Dialogue {speak} / Screen {screen}',
      configure: 'Configure Styles',
      listLabel: 'Style Profile List',
      styleCount: '{count} style(s)',
      speakStyle: 'Dialogue Style',
      screenStyle: 'Screen Text Style',
      emptyDescription: 'Creating a profile lets you choose a reference ASS / SSA file and uses the file name as the default profile name.',
      create: 'New Style Profile',
      rename: 'Rename Style Profile',
      nameLabel: 'Profile Name',
      untitled: 'Untitled Profile',
      speakUnset: 'No dialogue style',
      screenUnset: 'No screen style'
    }
  },
  encoderOptions: {
    libx264: 'CPU libx264 (H.264, best compatibility, supports AVS)',
    libx265: 'CPU libx265 (H.265/HEVC, smaller files, slower)',
    h264Nvenc: 'NVIDIA h264_nvenc (GPU hardware encoding, fast, no AVS support)',
    h264Amf: 'AMD h264_amf (GPU hardware encoding, fast, no AVS support)',
    h264Videotoolbox: 'macOS h264_videotoolbox (Apple hardware encoding, no AVS support)'
  },
  encodeSettings: {
    quality: 'Quality',
    qualityTitle: 'Quality',
    qualityEmptyCommand: 'Empty: do not generate -crf / -cq / -qp parameters',
    qualityCommand: 'x264/x265: -crf {crf}  |  NVENC: -cq {crf}',
    qualityBody: 'Lower values improve quality and increase file size. Leave empty to omit quality parameters when bitrate-only control is desired.',
    qualityItems: {
      x264: 'Recommended for libx264 / libx265: 18-28. 18 is visually lossless, 23 is default, 28 is lower quality.',
      hardware: 'Recommended for NVENC / AMF: 18-28. 19-23 is usually balanced.',
      videotoolbox: 'VideoToolbox does not use this quality value. Control it with max bitrate instead.'
    },
    emptyPlaceholder: 'Empty',
    maxBitrate: 'Max Bitrate',
    maxBitrateTitle: 'Max Bitrate',
    maxBitrateCommand: '-maxrate {value}k -bufsize {value×2}k',
    maxBitrateBody: 'Limits peak video bitrate so complex scenes do not spike out of control.',
    maxBitrateItems: {
      none: 'Unlimited: follow the quality value completely.',
      auto: 'Auto: original video bitrate + 1000 Kbps.',
      custom: 'Custom: use the entered Kbps value directly.'
    },
    bitrateOptions: {
      none: 'Unlimited',
      auto: 'Auto (source bitrate + 1000 Kbps)',
      custom: 'Custom'
    },
    bitratePlaceholder: 'e.g. 3000',
    encoder: 'Encoder',
    encoderTitle: 'Encoder',
    encoderBody: 'Select the video encoder backend. This affects speed, file size, compatibility, and AVS support.',
    encoderItems: {
      x264: 'libx264: H.264 CPU software encoding, best compatibility, stable quality, supports AVS.',
      x265: 'libx265: H.265/HEVC CPU software encoding, smaller files, slower speed.',
      hardware: 'h264_nvenc / h264_amf: GPU hardware encoding, fast, does not support AVS.',
      videotoolbox: 'h264_videotoolbox: macOS hardware encoding, does not support AVS.'
    }
  },
  videoMeta: {
    dialog: {
      chooseVideo: 'Choose Video File',
      chooseSubtitle: 'Choose Subtitle File',
      chooseVideoAndSubtitle: 'Choose Video and Subtitle Files',
      videoFilter: 'Video',
      subtitleFilter: 'Subtitle',
      videoSubtitleFilter: 'Video and Subtitle'
    },
    dash: '-',
    dimensions: 'W {width} x H {height}',
    dimensionTitle: 'Frame pixel dimensions ({tags})',
    dimensionTitlePlain: 'Frame pixel dimensions (width x height)',
    durationWithStart: '{duration} (start {start}s)',
    fields: {
      resolution: 'Resolution',
      codec: 'Codec',
      pixel: 'Pixel',
      bitrate: 'Bitrate',
      fps: 'FPS',
      frameMode: 'Frame Mode',
      totalFrames: 'Total Frames',
      colorSpace: 'Color Space',
      colorRange: 'Color Range',
      sampleRate: 'Sample Rate',
      channels: 'Channels'
    },
    approx: 'approx. ',
    defaultTemplate: 'Naming Template',
    empty: {
      title: 'Drop a video to start',
      note: 'Supports single-video processing. Drop subtitles at the same time when subtitle burn-in is needed.',
      video: 'Video: mp4 / mkv / mov / ts / m4v / flv / avi / webm / wmv / mpg / 3gp / mts',
      subtitle: 'Optional subtitles: ass / ssa / srt / vtt / sub',
      choose: 'Choose Files'
    },
    title: 'Video Info',
    parsing: 'Parsing...',
    video: 'Video',
    subtitle: 'Subtitle',
    output: 'Output',
    file: 'File',
    audio: 'Audio',
    notImported: 'Not imported',
    unset: 'Not set',
    clearAndReimport: 'Clear and re-import',
    clearVideo: 'Clear video',
    clearSubtitle: 'Clear subtitle',
    chooseVideo: 'Choose video file',
    chooseSubtitle: 'Choose subtitle file',
    outputPlaceholder: 'Example: E:\\path\\to\\output.mp4',
    applyTemplate: 'Apply naming template',
    applyTemplateLabel: 'Apply naming template: {name}',
    editOutputPath: 'Edit output path',
    editOutput: 'Edit output',
    fileSize: 'File size',
    size: 'Size',
    duration: 'Duration',
    durationTip: 'Media duration. Non-zero start time is shown in parentheses.',
    container: 'Container',
    tips: {
      videoCodec: 'Video codec and profile: determines compression algorithm and profile, such as h264 High or hevc Main10.',
      pixelFormat: 'Pixel format (color sampling / bit depth): common yuv420p is 8-bit 4:2:0; yuv420p10le is 10-bit.',
      videoBitrate: 'Video bitrate: data per second. Higher values improve quality and increase file size.',
      fps: 'Frame rate: frames per second (fps).',
      cfr: 'Constant frame rate (CFR): frame intervals are even. In ffprobe, r_frame_rate is approximately avg_frame_rate.',
      vfr: 'Variable frame rate (VFR): uneven frame intervals, common in screen recordings and some web videos. Consider remapping if stable frame rate is required.',
      estimatedFrames: 'Estimated total frames. ffprobe did not provide nb_frames, so this is duration x frame rate without slow per-frame counting.',
      exactFrames: 'Total frames recorded in the container, from ffprobe nb_frames.',
      colorSpace: 'Color space / primaries: common values include bt709 (SDR) and bt2020 (HDR).',
      colorRange: 'Luma range: tv/limited is 16-235; pc/full is 0-255.',
      audioCodec: 'Audio codec and profile: for example aac LC, HE-AAC, ac3, eac3, opus.',
      sampleRate: 'Sample rate: samples per second, commonly 44.1 kHz / 48 kHz.',
      channels: 'Channel layout: mono, stereo, 5.1 surround, 7.1, etc.',
      audioBitrate: 'Audio bitrate: data per second.',
      demuxer: 'ffmpeg demuxer: {format}. All compatible extensions handled by the same demuxer are listed.'
    }
  },
  subtitleCheck: {
    matrix: {
      assRaw: 'ASS declaration',
      videoStandard: 'Video color space',
      videoRange: 'Video range',
      undeclared: 'Undeclared',
      unknown: 'Unknown'
    },
    missingImage: {
      title: 'Subtitle image path does not exist',
      detail: 'ASS/SSA \\img / \\1img-\\4img image fill tags reference files that do not exist locally. AVS/VSFilterMod rendering may miss images or fail.',
      suggestion: 'Put the image files back at the original paths, or update the img paths in the subtitle and scan again.'
    },
    missingFont: {
      title: 'Subtitle font is not installed',
      detail: 'Missing fonts trigger system or renderer font fallback, which may change glyphs, weight, layout width, and effect positions.',
      suggestion: 'Install the fonts bundled with the subtitle package, or change ASS Style Fontname to an installed local font.'
    },
    missingStyle: {
      title: 'Subtitle line references a missing style',
      detail: 'A Dialogue/Comment line in Events references a style not defined in Styles. Rendering may fall back to default style or produce abnormal effects.',
      suggestion: 'Add the matching Style in [V4+ Styles], or change the event line Style field to an existing style.'
    },
    lineTag: 'Line {line} {tag}',
    line: 'Line {line}',
    effectSuggestion: 'Enable AVS encoding mode, or verify whether non-AVS output is acceptable.',
    titleChecking: 'Checking Subtitles',
    title: 'Subtitle Check',
    analyzingSummary: 'Analyzing subtitle effects, fonts, and resource references',
    summary: '{count} item(s) need review',
    level: {
      error: 'Error',
      warn: 'Warning',
      info: 'Suggestion',
      ok: 'OK'
    },
    effects: {
      title: 'VSFilterMod tags detected; AVS encoding mode is recommended',
      banner: 'ASS Effect Banner scrolling was detected, with fadeawaywidth or lowercase banner. ffmpeg libass does not support this effect; AVS+VSFilterMod is required to restore it correctly.',
      image: 'These tags usually depend on AVS/VSFilterMod rendering. Verify resource files and enable AVS encoding to restore subtitle effects as closely as possible.',
      modTag: 'These tags usually depend on AVS/VSFilterMod rendering. Enable AVS encoding to restore subtitle effects as closely as possible.',
      fallback: 'The subtitle contains tags that are recommended for AVS encoding. Confirm AVS mode is enabled before encoding.'
    },
    collapse: 'Collapse',
    expand: 'Expand',
    collapseDetail: 'Hide Details',
    viewDetail: 'View Details',
    hitTags: 'Matched tags'
  },
  colorMatrix: {
    standard: {
      unknown: 'Unknown'
    },
    missingBt2020: {
      title: 'ASS does not declare YCbCr Matrix, but the video is BT.2020 (HDR/4K)',
      detail: 'libass chooses a matrix by resolution heuristics by default (PlayResY >= 720 -> BT.709). Burning subtitles into BT.2020 video can shift the colors.'
    },
    missingFullRange: {
      title: 'ASS does not declare YCbCr Matrix, but the video is full range',
      detail: 'libass renders subtitles as limited range by default. On full-range video, black can become gray or white can clip.'
    },
    none: {
      detail: 'The subtitle author explicitly skipped RGB to YUV conversion. Unless you know this is intentional, this setting is usually unnecessary.'
    },
    unrecognized: {
      title: 'ASS YCbCr Matrix value is not recognized: {value}',
      detail: 'Standard values are TV.601 / TV.709 / TV.2020 / PC.601 / PC.709 / PC.2020 / None'
    },
    matrixMismatch: {
      title: 'ASS matrix ({ass}) does not match video ({video})',
      detail: 'Burned subtitle colors may shift globally, such as red/blue offsets, because libass / VSFilterMod applies RGB to YUV conversion using the ASS-declared matrix.'
    },
    rangeMismatch: {
      title: 'ASS range ({ass}) does not match video ({video})',
      detail: 'The matrix matches, but the range differs. Black/white levels can shift, often making subtitle black gray or white clipped.'
    },
    videoUnknown: {
      title: 'The video does not declare color_space, so strict comparison is unavailable'
    },
    ok: {
      title: 'Color matrix matches ({matrix})'
    },
    suggestion: {
      addMatrix: 'Add this under ASS [Script Info]: YCbCr Matrix: {matrix}',
      changeTo: 'Change it to {matrix}',
      changeHeader: 'Change the ASS header to: YCbCr Matrix: {matrix}'
    }
  },
  ffmpegPanel: {
    checkingTitle: 'Checking ffmpeg Environment',
    checkingSubtitle: 'Checking ffmpeg / ffprobe. Please wait.',
    checkingSubtitleWithLibass: 'Checking ffmpeg / ffprobe / subtitles/libass. Please wait.',
    incomplete: 'ffmpeg Features Incomplete',
    missing: 'ffmpeg Not Detected',
    missingHelp: 'Configure the ffmpeg path in Settings, or install it and add it to the system PATH.',
    refresh: 'Recheck'
  },
  home: {
    start: 'Start Encoding',
    cancel: 'Cancel Encoding',
    commandPreviewDisabledTip: 'The command is generated after the video and parameters are ready.',
    defaultOutputSuffix: ' subtitles',
    logoDisabled: {
      noVideo: 'Select a video file first',
      metaPending: 'Video resolution has not been parsed yet'
    },
    logs: {
      emptyVideoPath: 'Error: video path is empty. Fill it in or drop a video file first.',
      checkingAvsStaging: 'Checking video and AVS preprocessing requirements...',
      avsStagingCancelled: 'Encoding cancelled: VP9 AVS compatibility mode needs to copy the source video temporarily.',
      preparingAvsStaging: 'Preparing AVS temporary files. For VP9 source video, ffmpeg progress may stay at 0% while large files are copied.',
      runJobError: 'runJob error: {message}',
      cancelRequested: 'Cancel request sent'
    },
    avsStaging: {
      kicker: 'AVS Compatibility Mode',
      title: 'VP9 source video needs a temporary copy',
      summary: 'VP9 video was detected. This run copies the source video to an ASCII temporary path first, then reads it through the local 64-bit DirectShow decoding chain. 64-bit LAV Filters are usually required.',
      tempSize: 'Temporary Size',
      tempPath: 'Temporary Path',
      continue: 'Continue Encoding'
    }
  },
  presets: {
    import: 'Import',
    export: 'Export',
    resetBuiltIn: 'Restore Built-ins',
    copyName: '{name} Copy',
    outputDir: {
      sameAsVideo: 'Follow Video Folder',
      sameAsVideoDescription: 'Output to the source video folder',
      fixed: 'Fixed Folder',
      fixedDescription: 'Always output to the selected folder'
    },
    confirm: {
      deleteOutputTemplate: 'Delete template "{name}"? Existing generated output paths are not affected.',
      deleteEncodePreset: 'Delete encode preset "{name}"?',
      resetEncodePresets: 'Restoring built-in encode presets resets the five built-ins (x264 Balanced / NVENC Fast / AMF Fast / Apple Fast / x265 Size First). Your custom presets are kept. Continue?'
    },
    dialog: {
      fixedOutputDir: 'Choose Fixed Output Folder',
      exportOutputTemplates: 'Export Output Naming Templates',
      importOutputTemplates: 'Import Output Naming Templates',
      exportEncodePresets: 'Export Encode Presets',
      importEncodePresets: 'Import Encode Presets'
    },
    toast: {
      fixedDirRequired: 'Fixed folder cannot be empty. Choose a folder first.',
      saveFailed: 'Save failed: {message}',
      outputTemplateSaved: 'Naming template saved',
      outputTemplateCreated: 'Naming template created',
      outputTemplateDuplicated: 'Naming template duplicated',
      outputTemplateDeleted: 'Naming template deleted',
      outputTemplateDefault: 'Default naming template set',
      outputTemplateReordered: 'Naming template order updated',
      outputTemplateExported: 'Output naming templates exported',
      noOutputTemplatesImported: 'No importable output naming templates found',
      outputTemplateImported: 'Imported {count} output naming template(s)',
      encodePresetSaved: 'Encode preset saved',
      encodePresetExported: 'Encode presets exported',
      noEncodePresetsImported: 'No importable encode presets found',
      encodePresetImported: 'Imported {count} encode preset(s)',
      encodePresetCreated: 'Encode preset created',
      encodePresetDuplicated: 'Encode preset duplicated',
      encodePresetDeleted: 'Encode preset deleted',
      encodePresetReordered: 'Encode preset order updated',
      encodePresetReset: 'Built-in encode presets restored; custom presets were kept'
    },
    encodePreset: {
      title: 'Encode Presets',
      description: 'Manage common encoders, CRF, max bitrate, and advanced ffmpeg video arguments. The Encode page can select and apply them directly.',
      defaultName: 'Encode Preset {index}',
      nameLabel: 'Preset Name',
      advancedArgs: 'Advanced ffmpeg Video Args',
      applyPreview: 'Applied settings',
      encoderSummary: 'Encoder {encoder} / CRF {crf} / Bitrate {bitrate}',
      save: 'Save Preset'
    },
    outputTemplate: {
      title: 'Output Naming Templates',
      description: 'Create reusable naming rules. Select a template on the Encode page to apply it to the output path.',
      defaultName: 'Template {index}',
      untitled: 'Untitled Template',
      nameLabel: 'Template Name',
      patternLabel: 'File Name Template',
      variablesLabel: 'Available file name variables',
      insertVariable: 'Insert Variable',
      insertVariableHint: 'Click to insert at the cursor',
      outputDir: 'Output Folder',
      fixedDirPlaceholder: 'Click the button on the right to choose a folder',
      chooseDir: 'Choose Folder',
      fixedDirHint: 'Fixed folders must be set through the system folder picker',
      preview: 'Preview',
      save: 'Save Template',
      setDefault: 'Set Default'
    }
  },
  outputTemplates: {
    defaultName: 'Default',
    defaultPattern: '{video_name} subtitles.mp4',
    variables: {
      dateYmd: 'Date Format 1',
      dateShort: 'Date Format 2',
      videoName: 'Video File Name',
      resolution: 'Resolution',
      encoder: 'Encoder',
      crf: 'CRF'
    }
  },
  encodePresets: {
    balancedX264: 'x264 Balanced',
    fastNvenc: 'NVENC Fast',
    fastAmf: 'AMF Fast',
    fastVideotoolbox: 'Apple Fast',
    hevcSmall: 'x265 Size First'
  },
  logoEditor: {
    title: 'Configure LOGO Position',
    confirmUnsaved: 'The current LOGO configuration has not been saved. Close anyway?',
    dialog: {
      chooseLogo: 'Choose LOGO Image',
      imageFilter: 'Images'
    },
    errors: {
      imageLoadFailed: 'Failed to load LOGO image',
      chooseLogoFirst: 'Choose a LOGO image first'
    },
    drop: {
      release: 'Release to load image',
      choose: 'Choose LOGO Image',
      hint: 'Click to choose, or drop PNG / JPG / WEBP / BMP'
    },
    current: {
      title: 'Current LOGO',
      empty: 'No LOGO selected'
    },
    recent: {
      title: 'Recent',
      empty: 'No recent items',
      removeTip: 'Remove {name}'
    },
    status: {
      position: 'Position: ',
      size: 'Size: ',
      percent: 'Percent: ',
      video: 'Video: '
    },
    frame: {
      alt: 'Preview frame',
      loading: 'Extracting frame...',
      loadingAria: 'Extracting frame',
      waiting: 'Waiting for video frame...',
      refresh: 'Extract current frame again'
    },
    zoom: {
      aria: 'Frame zoom ratio',
      zoom: 'Zoom:',
      reset: 'Reset:',
      wheel: 'Wheel'
    }
  },
  compressForm: {
    title: 'Encoding Parameters',
    preset: {
      title: 'Encode Presets',
      apply: 'Apply Preset',
      applyTip: 'Choose an encode preset and apply it to the current parameters',
      lastApplied: 'Last applied: {name}',
      tip: 'Apply a saved encoder, quality, and max bitrate combination.\n\nAdd, edit, or delete presets on the Presets page in the sidebar.\nAfter applying a preset, quality, max bitrate, and encoder can still be adjusted manually and will not be written back to the preset.',
      itemManage: 'Add, edit, or delete presets on the Presets page in the sidebar.',
      itemUseCase: 'Useful for saving common platform specs as reusable profiles.',
      applied: 'Preset applied',
      appliedNamed: 'Applied to current parameters: {name}',
      qualitySummary: 'Quality {crf}',
      unlimitedBitrate: 'Unlimited bitrate',
      autoBitrate: 'Auto bitrate',
      sourceBitratePlus: 'source bitrate + 1000k',
      doubleMaxBitrate: '2x max bitrate'
    },
    quick: {
      title: 'Video Processing',
      off: 'Off',
      custom: 'Custom',
      noneSelected: 'No processing selected',
      customScaleSummary: 'Scale {value}',
      body: 'Compile rotation, mirroring, resolution, frame rate, and bitrate processing into the current encoding command. This re-encodes the video and outputs a new file.',
      itemUseCase: 'Useful for rotation, mirroring, scaling, frame dropping, or bitrate adjustment.',
      itemReuse: 'Subtitles, LOGO, encoder, and quality settings still reuse the current Encode page settings.',
      rotate: 'Rotate',
      rotateBody: 'Rotate the output image during encoding. This writes a video filter and re-encodes the picture.',
      rotateItemUseCase: 'Useful for phone vertical video, wrong screen recording orientation, and similar cases.',
      rotateItem180: '180° uses hflip,vflip and is equivalent to flipping the picture upside down.',
      mirrorLabel: 'Mirror',
      mirrorBody: 'Flip the picture horizontally or vertically during encoding. Can be combined with rotation.',
      mirrorItemH: 'Horizontal mirror flips left and right.',
      mirrorItemV: 'Vertical mirror flips top and bottom.',
      scaleLabel: 'Resolution',
      scaleBody: 'Scale the output image with a preset or custom expression while preserving the original aspect ratio as much as possible.',
      scaleItemLandscape: 'Landscape presets control height. For example, 1080 means output height 1080.',
      scaleItemPortrait: 'Portrait presets control width. For example, 1080 means output width 1080.',
      customScale: 'Custom Scale',
      customScaleCommand: 'scale=width:height',
      customScaleBody: 'Enter ffmpeg scale width and height expressions directly for sizes not covered by presets.',
      customScaleItemHeight: 'Example: -1:1080 means height 1080 and width is calculated proportionally.',
      customScaleItemWidth: 'Example: 1080:-1 means width 1080 and height is calculated proportionally.',
      customScalePlaceholder: 'e.g. -1:1080 or 1080:-1',
      frameRate: 'Frame Rate',
      frameRateBody: 'Limit output video frame rate, commonly used to reduce size or standardize publishing specs.',
      frameRateItemEmpty: 'Leave empty to keep the frame rate unchanged.',
      frameRateItemValues: 'Enter 30 for 30 fps output; enter 60 for 60 fps output.',
      videoBitrate: 'Video Bitrate',
      videoBitrateBody: 'Set a target bitrate for the video stream, mainly to control output size and platform specs.',
      videoBitrateItemEmpty: 'Leave empty to avoid specifying video bitrate and continue using the current encoder and quality value.',
      videoBitrateItemValue: 'Enter 5000 for a target video bitrate of about 5000 Kbps.',
      noChange: 'No change',
      rotation: {
        none: 'No rotation',
        cw: 'Clockwise 90°',
        ccw: 'Counterclockwise 90°',
        rotate180: 'Rotate 180°'
      },
      mirror: {
        none: 'No mirror',
        hflip: 'Horizontal mirror',
        vflip: 'Vertical mirror'
      },
      scale: {
        none: 'Do not adjust resolution',
        landscape4k: 'Landscape 4K (height 2160)',
        landscape1080: 'Landscape 1080 (height 1080)',
        landscape720: 'Landscape 720 (height 720)',
        portrait1080: 'Portrait 1080 (width 1080)',
        portrait720: 'Portrait 720 (width 720)'
      }
    },
    advanced: {
      show: 'Show Extra Args',
      hide: 'Hide Extra Args',
      label: 'Extra ffmpeg Video Args',
      note: 'These arguments are appended after video encoder arguments. Input, filters, encoder, audio, and output path are still managed by the workstation.'
    },
    options: {
      yadif: 'Use Deinterlacing',
      yadifTitle: 'Deinterlacing',
      yadifBody: 'Convert interlaced signals into continuous frames to remove horizontal jagged or comb artifacts.',
      yadifItemInterlaced: 'TV recordings, transcodes, DV, and digitized tape sources often contain interlacing and usually benefit from this.',
      yadifItemProgressive: 'Web publishing videos are usually already progressive and generally do not need it.'
    },
    logo: {
      title: 'Burn LOGO',
      body: 'Overlay a LOGO image on the video and visually configure its image, position, and size.',
      itemConfigure: 'Click Configure LOGO to open the editor.',
      itemKeepLayout: 'When disabled, saved LOGO layouts are kept but not used during encoding.',
      openTip: 'Open the LOGO editor to visually set image, position, and size',
      configure: 'Configure LOGO',
      reconfigure: 'Reconfigure LOGO',
      notConfigured: 'LOGO not configured',
      summary: 'Configured: {name} · {position} · {size}',
      sizePixels: '{width} x {height} px',
      layer: 'LOGO Layer',
      layerBody: 'Control the overlap order between subtitles and LOGO.',
      layerItemBottom: 'Subtitles above, LOGO below: subtitles can cover the LOGO.',
      layerItemTop: 'LOGO above, subtitles below: LOGO fully covers subtitles.',
      layerItemAvs: 'In AVS mode, subtitles are rendered by AVS, so the LOGO layer is locked above subtitles.',
      layerBottom: 'Subtitles Above, LOGO Below',
      layerBottomTitle: 'Subtitles can cover the LOGO',
      layerTop: 'LOGO Above, Subtitles Below',
      layerTopTitle: 'LOGO fully covers subtitles',
      position: {
        center: 'Center',
        topCenter: 'Top Center',
        bottomCenter: 'Bottom Center',
        leftCenter: 'Left Center',
        rightCenter: 'Right Center',
        topLeft: 'Top Left',
        topRight: 'Top Right',
        bottomLeft: 'Bottom Left',
        bottomRight: 'Bottom Right'
      }
    },
    avs: {
      title: 'AVS Encoding Mode',
      body: 'Use an AviSynth+ script as ffmpeg input and render subtitles with VSFilterMod. LOGO overlay and yadif still work.',
      itemWindows: 'Windows only. Requires AviSynth+ installed locally and ffmpeg built with --enable-avisynth.',
      itemUseCase: 'Best for complex ASS effect subtitles. Leave unchecked to use ffmpeg filter mode.',
      toggleTip: 'Use an AviSynth+ script as ffmpeg input. Subtitles are rendered by VSFilterMod TextSubMod.\nWindows only; requires AviSynth+ installed locally and ffmpeg built with --enable-avisynth, such as Gyan.dev full builds.\nWhen enabled, LOGO overlay and yadif still work, but the ffmpeg subtitles filter is skipped.\n\nEnable AVS encoding mode; leave unchecked to use ffmpeg filter mode.',
      windowsOnly: 'AVS encoding is Windows-only',
      checking: 'Checking AVS environment...',
      unavailable: 'AVS environment unavailable',
      lavResolving: 'Checking 64-bit LAV Filters. VP9 video must confirm DirectShow decoding support before AVS can be enabled...',
      lavCheckingTip: 'Checking 64-bit LAV Filters.\nVP9 video is read through the DirectShow decoding chain; AVS can be enabled after decoding support is confirmed.',
      lavMissingTip: '64-bit LAV Filters were not detected.\nWhen encoding VP9 video, it is read through the DirectShow decoding chain; missing LAV may cause decoding failure.\nInstall 64-bit LAV Filters and try again.',
      autoEnabled: 'Subtitle effects detected ({tags}); AVS encoding was enabled automatically',
      detectedSpecialTags: 'Special tags detected',
      lavChecking: 'Checking LAV...',
      lavMissing: 'LAV missing'
    }
  },
  subtitleFormat: {
    ready: 'Ready to convert',
    disabled: {
      checking: 'Checking ffmpeg',
      ffmpegUnavailable: 'Configure a usable ffmpeg in Settings first',
      input: 'Select an input subtitle',
      output: 'Select an output subtitle path',
      conflict: 'Output path cannot be the same as the input subtitle'
    },
    outputFile: '{stem} converted.{format}',
    dialog: {
      inputTitle: 'Choose Subtitle to Convert',
      subtitleFilter: 'Subtitle',
      outputTitle: 'Choose Output Subtitle Path'
    },
    outputLog: 'Output: {path}',
    dropzone: {
      title: 'Drop a subtitle to convert',
      note: 'Input supports ASS / SSA / SRT / VTT / TTML / SUB',
      description: 'The output path is generated after selecting the target format. TTML input only converts to SRT and simplifies unsupported styles.',
      choose: 'Choose Subtitle'
    },
    inputLabel: 'Input Subtitle',
    inputPlaceholder: 'Choose ass / ssa / srt / vtt / ttml / sub subtitle file',
    targetFormat: 'Target Format',
    targetFormatTitle: 'Choose output subtitle format',
    outputLabel: 'Output Subtitle',
    outputPlaceholder: 'Choose output subtitle path',
    conflictWarning: 'Output path cannot be the same as the input subtitle. Choose a new file.',
    noteTitle: 'Processing Notes',
    noteAss: 'ASS / SSA preserve richer styling. SRT / VTT are more universal; TTML input is converted to SRT with basic text and timing.',
    noteSimplify: 'If the source subtitle contains complex positioning, effects, or font styling, these details are simplified according to the target format.',
    start: 'Start Conversion',
    cancel: 'Cancel Conversion',
    running: 'Converting...',
    progressTitle: 'Conversion Progress',
    idleTitle: 'Conversion Not Started',
    idleTip: 'Choose input, target format, and output path, then click Start Conversion above.'
  },
  mediaTool: {
    ready: 'Ready to convert',
    disabled: {
      checking: 'Checking ffmpeg',
      ffmpegUnavailable: 'Configure a usable ffmpeg in Settings first',
      segmentFolder: 'Select a segment folder',
      inputVideo: 'Select an input video',
      cover: 'Select a cover image',
      audio: 'Select an audio source',
      output: 'Select an output {format} path',
      conflict: 'Output path cannot be the same as an input file',
      loadingSegments: 'Reading segment list',
      noSegments: 'The selected folder has no mergeable TS / M2TS / MTS segments'
    },
    description: {
      concatTsToMp4: 'Merge TS / M2TS / MTS segments in the current order and output {format} without re-encoding.',
      addCoverToMp4: 'Write JPG / PNG cover art into MP4 while copying the original video and audio.',
      mergeAudioVideo: 'Keep the video picture, merge a separate audio source, and output MP4.',
      remuxToMp4: 'Remux common video containers to MP4, copying audio and video streams by default.'
    },
    outputName: {
      concatTsToMp4: '{stem} merged.{extension}',
      addCoverToMp4: '{stem} cover.mp4',
      mergeAudioVideo: '{stem} merged audio.mp4',
      remuxToMp4: '{stem} MP4 remux.mp4'
    },
    dialog: {
      inputCoverVideo: 'Choose MP4 Video for Cover Art',
      inputVideo: 'Choose Video to Remux to MP4',
      videoFilter: 'Video',
      cover: 'Choose Cover Image',
      coverFilter: 'Cover Image',
      audio: 'Choose Audio Source to Merge',
      audioVideoFilter: 'Audio or Video',
      segmentFolder: 'Choose TS Segment Folder',
      output: 'Choose Output {format} File',
      outputFilter: '{format} Video'
    },
    dropzone: {
      concatTitle: 'Drop a TS segment folder to merge',
      videoTitle: 'Drop a video to start',
      concatNote: 'Supports TS / M2TS / MTS segment folders',
      coverNote: 'Supports MP4 / M4V / MOV',
      videoNote: 'Supports common video containers',
      concatDescription: 'Segments are merged in filename order after reading. Output can be MP4 or TS.',
      mergeDescription: 'Select a video, then add an audio source. Streams are copied by default without re-encoding.',
      coverDescription: 'Select a video, then add a JPG / PNG cover image. Audio and video are not re-encoded.',
      remuxDescription: 'An output MP4 path is generated after selecting a video. Streams are copied by default.'
    },
    chooseSegmentFolder: 'Choose Segment Folder',
    chooseVideo: 'Choose Video',
    input: {
      segmentFolder: 'Segment Folder',
      video: 'Input Video',
      segmentPlaceholder: 'Choose a folder containing .ts / .m2ts / .mts',
      coverVideoPlaceholder: 'Choose an mp4 / m4v / mov video file',
      mergeVideoPlaceholder: 'Choose the video file whose picture should be kept',
      videoPlaceholder: 'Choose an mkv / mov / ts / flv video file',
      audio: 'Input Audio',
      audioPlaceholder: 'Choose an audio or video file',
      cover: 'Cover Image',
      coverPlaceholder: 'Choose a jpg / png cover image'
    },
    outputFormat: 'Output Format',
    outputFormatTitle: 'Choose TS segment merge output format',
    outputLabel: 'Output {format}',
    outputPlaceholder: 'Choose output location',
    conflictWarning: 'Output path cannot be the same as an input file. Choose a new {format} file.',
    noteTitle: 'Processing Notes',
    notes: {
      remux1: 'Remux the video into an MP4 container. Video and audio streams are copied by default without re-encoding.',
      remux2: 'TS / M2TS / MTS input automatically normalizes AAC bitstream headers for MP4 compatibility without changing audio quality.',
      cover1: 'Write a cover image into MP4 so players and file managers can show custom artwork.',
      cover2: 'Original video and audio are copied without re-encoding. The cover image is written as a cover stream.',
      merge1: 'Keep the input video picture and use the first audio track from the audio source as the main output track.',
      merge2: 'Video and audio are copied by default without re-encoding. Output ends at the shorter stream to avoid a trailing gap.'
    },
    segments: {
      title: 'Segment Order Preview',
      loading: 'Reading...',
      count: '{count} file(s) · {size}',
      empty: 'No segments loaded yet',
      noSegments: 'The selected folder has no TS / M2TS / MTS segments.',
      truncated: 'Showing the first {visible}; all {total} segments are merged in the current order.',
      note: 'MP4 output normalizes AAC headers in TS segments. TS output keeps the TS container. Neither mode re-encodes.'
    },
    errors: {
      incompatibleContainer: 'The current audio/video streams may not be compatible with the {format} container. Re-encode on the Encode page before exporting.'
    },
    cancelRequested: 'Cancel request sent',
    start: 'Start Conversion',
    cancel: 'Cancel Conversion',
    previewDisabledTip: 'The command is generated after input and output are selected.',
    progressTitle: 'Conversion Progress',
    idleTitle: 'Conversion Not Started',
    idleTip: 'Choose input and output, then click Start Conversion above.'
  },
  ruleDictionary: {
    close: 'Close',
    tabsLabel: 'Dictionary edit mode',
    entryTab: 'Entry Editor',
    rawTab: 'Raw Text',
    add: 'Add',
    delete: 'Delete',
    invalidLine: 'This line format cannot be recognized. Switch to raw text to inspect it.',
    invalidPattern: 'The match rule does not look like a valid regular expression.',
    addFirst: 'Add First Rule',
    testMatch: 'Test Match',
    testPlaceholder: 'Enter a short subtitle sample to check whether the rules match.',
    noMatches: 'No entries matched.',
    summary: '{valid} valid rule(s)',
    invalidSummary: ', {invalid} need review',
    autoSaveHint: '. Changes are remembered automatically and reprocessed',
    captureHint: '; target text supports %1 capture groups',
    endPunctuation: '.',
    done: 'Done'
  }
} as const
