import { invoke, isTauri } from '@tauri-apps/api/core'
import { acceptHMRUpdate, defineStore } from 'pinia'

type ThemeMode = 'light' | 'dark' | 'system'
type RoundnessMode = 'sharp' | 'rounded' | 'soft'
type LayoutMode = '1x2' | '2x1' | 'grid'
type SessionKey = 'work' | 'research' | 'personal'

interface UserProfile {
  name: string
  plan: string
  email: string
}

interface SessionItem {
  id: SessionKey
  label: string
  icon: string
  hint: string
}

interface BrowserPanel {
  id: number
  title: string
  url: string
  summary: string
  tag: string
  accent: string
  status: 'online' | 'syncing'
}

interface SettingsState {
  theme: ThemeMode
  roundness: RoundnessMode
  acrylicEnabled: boolean
  transparency: number
  launchAtStartup: boolean
  homeUrl: string
}

interface UpdaterStatusPayload {
  enabled: boolean
  reason: string | null
  currentVersion: string
  endpoints: string[]
}

interface UpdatePayload {
  version: string
  currentVersion: string
  date: string | null
  body: string | null
}

interface UpdaterState {
  enabled: boolean
  reason: string | null
  endpoints: string[]
  checking: boolean
  installing: boolean
  available: UpdatePayload | null
  lastCheckedAt: string | null
  lastError: string | null
  promptVisible: boolean
}

interface LogStatusPayload {
  directory: string
  filePath: string
}

interface LogState {
  directory: string | null
  filePath: string | null
  lastError: string | null
}

const versionString =
  import.meta.env.MODE === 'development' ? `${import.meta.env.VITE_APP_VERSION}-dev` : import.meta.env.VITE_APP_VERSION
const AUTO_UPDATE_INTERVAL_MS = 2 * 60 * 60 * 1000

let autoUpdateTimer: ReturnType<typeof setInterval> | null = null

const basePanels: BrowserPanel[] = [
  {
    id: 1,
    title: '飞书文档',
    url: 'https://feishu.cn/docx/team-space',
    summary: '项目周报、迭代安排与成员同步集中在这个窗格，适合工作模式快速切换。',
    tag: '工作空间',
    accent: 'from-sky-500/30 via-cyan-400/10 to-transparent',
    status: 'online',
  },
  {
    id: 2,
    title: '中国知网',
    url: 'https://scholar.cnki.net/advanced-search',
    summary: '研究资料检索与参考文献整理入口，模拟学术搜索结果页的阅读场景。',
    tag: '研究检索',
    accent: 'from-indigo-500/30 via-violet-400/10 to-transparent',
    status: 'syncing',
  },
  {
    id: 3,
    title: 'Gitee 探索',
    url: 'https://gitee.com/explore',
    summary: '开源项目趋势、Issue 追踪与代码片段浏览，用于模拟开发者工作台。',
    tag: '开源协作',
    accent: 'from-emerald-500/30 via-teal-400/10 to-transparent',
    status: 'online',
  },
]

export const useStore = defineStore('main', {
  state: () => ({
    debug: import.meta.env.MODE === 'development',
    version: versionString,
    isInitialized: false,
    isAuthenticated: false,
    isLoginSubmitting: false,
    isSettingsModalOpen: false,
    currentSession: 'work' as SessionKey,
    layoutMode: 'grid' as LayoutMode,
    loginForm: {
      username: 'admin@polybrowser.local',
      password: '123456',
      remember: true,
    },
    user: {
      name: '张伟',
      plan: '专业版方案',
      email: 'admin@polybrowser.local',
    } as UserProfile,
    sessions: [
      { id: 'work', label: '工作', icon: 'briefcase', hint: '4 个活跃面板' },
      { id: 'research', label: '研究', icon: 'school', hint: '2 个同步任务' },
      { id: 'personal', label: '个人', icon: 'account', hint: '收藏和订阅' },
    ] as SessionItem[],
    panels: basePanels as BrowserPanel[],
    settings: {
      theme: 'system',
      roundness: 'rounded',
      acrylicEnabled: true,
      transparency: 85,
      launchAtStartup: false,
      homeUrl: 'https://hub.polybrowser.local/start',
    } as SettingsState,
    updater: {
      enabled: false,
      reason: null,
      endpoints: [],
      checking: false,
      installing: false,
      available: null,
      lastCheckedAt: null,
      lastError: null,
      promptVisible: false,
    } as UpdaterState,
    logs: {
      directory: null,
      filePath: null,
      lastError: null,
    } as LogState,
    notifications: [
      '模拟登录已启用，可直接使用默认账号进入。',
      '所有统计和浏览器内容均为本地假数据，用于 UI 演示。',
    ],
  }),

  getters: {
    currentRadius(state) {
      switch (state.settings.roundness) {
        case 'sharp':
          return 8
        case 'soft':
          return 24
        default:
          return 16
      }
    },
    activeSession(state) {
      return state.sessions.find((item) => item.id === state.currentSession) ?? state.sessions[0]
    },
    themeClass(state) {
      return state.settings.theme === 'dark' ? 'theme-dark' : 'theme-light'
    },
  },

  actions: {
    initApp() {
      if (this.isInitialized) {
        return
      }
      this.isInitialized = true
      void this.loadUpdaterStatus()
      void this.loadLogStatus()
    },
    async login() {
      this.isLoginSubmitting = true
      await new Promise((resolve) => setTimeout(resolve, 600))
      this.isAuthenticated = true
      this.isLoginSubmitting = false
    },
    logout() {
      this.isAuthenticated = false
      this.isSettingsModalOpen = false
    },
    setSession(sessionId: SessionKey) {
      this.currentSession = sessionId
    },
    setLayout(layout: LayoutMode) {
      this.layoutMode = layout
    },
    openSettingsModal() {
      this.isSettingsModalOpen = true
    },
    closeSettingsModal() {
      this.isSettingsModalOpen = false
    },
    updateTheme(theme: ThemeMode) {
      this.settings.theme = theme
    },
    updateRoundness(roundness: RoundnessMode) {
      this.settings.roundness = roundness
    },
    setTransparency(value: number) {
      this.settings.transparency = value
    },
    addMockPanel() {
      const nextId = this.panels.length + 1
      this.panels.push({
        id: nextId,
        title: `新窗格 ${nextId}`,
        url: `${this.settings.homeUrl}?panel=${nextId}`,
        summary: '新建的模拟窗格，可用于展示更多站点或分屏内容。',
        tag: '新建内容',
        accent: 'from-amber-500/30 via-orange-400/10 to-transparent',
        status: 'online',
      })
    },
    async loadUpdaterStatus() {
      if (!isTauri()) {
        this.updater.enabled = false
        this.updater.reason = '仅 Tauri 桌面应用支持检查更新。'
        this.updater.endpoints = []
        return
      }

      try {
        const status = await invoke<UpdaterStatusPayload>('updater_status')
        this.version = import.meta.env.MODE === 'development' ? `${status.currentVersion}-dev` : status.currentVersion
        this.updater.enabled = status.enabled
        this.updater.reason = status.reason
        this.updater.endpoints = status.endpoints
      } catch (error) {
        this.updater.enabled = false
        this.updater.reason = `读取更新配置失败：${String(error)}`
        this.updater.endpoints = []
      }
    },
    async loadLogStatus() {
      if (!isTauri()) {
        this.logs.directory = null
        this.logs.filePath = null
        this.logs.lastError = '仅 Tauri 桌面应用支持本地日志。'
        return
      }

      try {
        const status = await invoke<LogStatusPayload>('logger_status')
        this.logs.directory = status.directory
        this.logs.filePath = status.filePath
        this.logs.lastError = null
      } catch (error) {
        this.logs.directory = null
        this.logs.filePath = null
        this.logs.lastError = String(error)
      }
    },
    async checkForUpdates(options?: { background?: boolean }) {
      if (!isTauri()) {
        this.updater.lastError = '当前不在 Tauri 桌面环境中，无法检查更新。'
        return
      }

      if (this.updater.checking || this.updater.installing) {
        return
      }

      this.updater.checking = true
      this.updater.lastError = null

      try {
        const update = await invoke<UpdatePayload | null>('check_for_update')
        this.updater.available = update
        this.updater.lastCheckedAt = new Date().toISOString()
        this.updater.promptVisible = Boolean(update)

        if (update && options?.background) {
          const message = `发现新版本 v${update.version}，可在设置中安装更新。`
          if (!this.notifications.includes(message)) {
            this.notifications.unshift(message)
          }
        }
      } catch (error) {
        this.updater.available = null
        this.updater.lastError = String(error)
      } finally {
        this.updater.checking = false
      }
    },
    async installAvailableUpdate() {
      if (!isTauri()) {
        this.updater.lastError = '当前不在 Tauri 桌面环境中，无法安装更新。'
        return
      }

      if (!this.updater.available || this.updater.installing) {
        return
      }

      this.updater.installing = true
      this.updater.lastError = null

      try {
        await invoke('install_update')
      } catch (error) {
        this.updater.lastError = String(error)
        this.updater.installing = false
      }
    },
    dismissUpdatePrompt() {
      this.updater.promptVisible = false
    },
    startAutoUpdateChecks() {
      if (!isTauri()) {
        return
      }

      this.stopAutoUpdateChecks()
      void this.checkForUpdates({ background: true })

      autoUpdateTimer = setInterval(() => {
        void this.checkForUpdates({ background: true })
      }, AUTO_UPDATE_INTERVAL_MS)
    },
    stopAutoUpdateChecks() {
      if (!autoUpdateTimer) {
        return
      }

      clearInterval(autoUpdateTimer)
      autoUpdateTimer = null
    },
    async openLogsDirectory() {
      if (!isTauri()) {
        this.logs.lastError = '当前不在 Tauri 桌面环境中，无法打开日志目录。'
        return
      }

      try {
        await invoke('open_logs_directory')
        this.logs.lastError = null
      } catch (error) {
        this.logs.lastError = String(error)
      }
    },
    async openDevtools() {
      if (!isTauri()) {
        return
      }

      try {
        await invoke('open_devtools')
      } catch (error) {
        this.logs.lastError = String(error)
      }
    },
  },
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useStore, import.meta.hot))
}
