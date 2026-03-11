<script setup lang="ts">
import BrowserPane from '@/components/BrowserPane.vue'
import SettingsPanel from '@/components/SettingsPanel.vue'
import { useWindowControls } from '@/composables/useWindowControls'

const store = useStore()

const layoutOptions = ['1x2', '2x1', 'grid'] as const
const { isWindowMaximized, minimizeWindow, toggleMaximizeWindow, closeWindow, startDraggingWindow } = useWindowControls()

const getSessionIconClass = (sessionId: string) => {
  switch (sessionId) {
    case 'work':
      return 'i-mdi-briefcase'
    case 'research':
      return 'i-mdi-school'
    case 'personal':
      return 'i-mdi-account'
    default:
      return 'i-mdi-circle-outline'
  }
}

const handleHeaderMouseDown = (event: MouseEvent) => {
  const target = event.target as HTMLElement | null
  if (!target || target.closest('button, input, textarea, select, a')) {
    return
  }

  void startDraggingWindow()
}
</script>

<template>
  <div class="dashboard-shell">
    <aside class="dashboard-sidebar dashboard-sidebar--compact dashboard-sidebar--design">
      <div class="dashboard-brand">
        <div class="dashboard-brand__icon">
          <span class="i-mdi-view-grid text-xl"></span>
        </div>
        <h1 class="dashboard-brand__title">聚合浏览器</h1>
      </div>

      <nav class="flex-1 px-3 space-y-1">
        <p class="px-3 mb-2 text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">会话</p>
        <button
          v-for="session in store.sessions"
          :key="session.id"
          class="session-button session-button--design"
          :class="{ 'is-active': store.currentSession === session.id }"
          type="button"
          @click="store.setSession(session.id)"
        >
          <span :class="getSessionIconClass(session.id)" class="session-button__icon"></span>
          <span class="text-sm font-medium">{{ session.label }}</span>
        </button>

        <button class="session-button session-button--design" type="button" @click="store.addMockPanel()">
          <span class="i-mdi-plus session-button__icon"></span>
          <span class="text-sm font-medium">新增</span>
        </button>
      </nav>

      <div class="p-4 border-t border-[var(--line)]">
        <button class="nav-link nav-link--design" type="button" @click="store.openSettingsModal()">
          <span class="i-mdi-cog-outline"></span>
          <span class="text-sm font-medium">设置</span>
        </button>

        <div class="mt-4 flex items-center justify-between gap-3 px-3">
          <div class="flex items-center gap-3 min-w-0">
            <div class="w-8 h-8 rounded-full bg-[var(--brand-soft)] flex items-center justify-center overflow-hidden text-[var(--brand)]">
              <span class="i-mdi-account-outline text-sm"></span>
            </div>
            <div class="flex flex-col min-w-0">
              <span class="text-xs font-semibold text-[var(--text-strong)]">{{ store.user.name }}</span>
              <span class="text-[10px] text-[var(--text-soft)]">{{ store.user.plan }}</span>
            </div>
          </div>
          <button class="text-[11px] font-medium text-[var(--text-soft)] hover:text-[var(--brand)] transition-colors" type="button" @click="store.logout()">
            退出
          </button>
        </div>
      </div>
    </aside>

    <main class="dashboard-main dashboard-main--flat" :class="{ 'is-blurred': store.isSettingsModalOpen }">
      <header class="dashboard-header dashboard-header--flat">
        <div class="dashboard-header__content flex items-center justify-between w-full" @mousedown.left="handleHeaderMouseDown">
          <div class="window-drag-region flex items-center gap-2 ml-4 flex-1 min-w-0" data-tauri-drag-region>
            <div class="segmented-control segmented-control--flat">
              <button
                v-for="layout in layoutOptions"
                :key="layout"
                class="segment segment--flat"
                :class="{ 'is-selected': store.layoutMode === layout }"
                type="button"
                @click="store.setLayout(layout)"
              >
                {{ layout }}
              </button>
            </div>
            <button class="icon-button icon-button--plain icon-button--add" type="button" @click="store.addMockPanel()">
              <span class="i-mdi-plus-circle-outline text-[26px]"></span>
            </button>
          </div>

          <div class="window-controls">
            <button class="window-control-button" type="button" @click="minimizeWindow">
              <span class="i-mdi-window-minimize"></span>
            </button>
            <button class="window-control-button" type="button" @click="toggleMaximizeWindow">
              <span :class="isWindowMaximized ? 'i-mdi-window-restore' : 'i-mdi-checkbox-blank-outline'"></span>
            </button>
            <button class="window-control-button window-control-button--close" type="button" @click="closeWindow">
              <span class="i-mdi-close"></span>
            </button>
          </div>
        </div>
      </header>

      <section
        v-if="store.updater.available && store.updater.promptVisible"
        class="mx-6 mt-4 rounded-3xl border border-emerald-500/20 bg-emerald-500/8 px-5 py-4 text-sm text-[var(--text-strong)]"
      >
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-1">
            <p class="font-semibold text-emerald-600">发现新版本 v{{ store.updater.available.version }}</p>
            <p class="text-[var(--text-soft)]">已在后台完成更新检查，你可以现在打开设置安装更新，也可以稍后再处理。</p>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button class="nav-link nav-link--design" type="button" @click="store.openSettingsModal()">
              去安装
            </button>
            <button class="text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-strong)]" type="button" @click="store.dismissUpdatePrompt()">
              稍后
            </button>
          </div>
        </div>
      </section>

      <section class="browser-grid">
        <BrowserPane v-for="pane in store.panels.slice(0, 3)" :key="pane.id" :pane="pane" />

        <button class="add-pane-card add-pane-card--flat" type="button" @click="store.addMockPanel()">
          <div class="w-16 h-16 rounded-full bg-white shadow-sm flex items-center justify-center mb-4 text-[var(--brand)]">
            <span class="i-mdi-plus text-4xl"></span>
          </div>
          <span class="text-sm font-semibold text-[var(--text-soft)]">新增窗格</span>
          <span class="text-[10px] text-[var(--text-muted)] mt-1">点击以添加新浏览器窗口</span>
        </button>
      </section>

      <footer class="dashboard-footer dashboard-footer--flat">
        <div class="flex items-center gap-4">
          <span class="flex items-center gap-1">
            <span class="w-2 h-2 rounded-full bg-green-500"></span>
            所有系统运行正常
          </span>
          <span>{{ store.panels.length }} 个面板活动中</span>
        </div>
        <div class="flex items-center gap-4">
          <span>CPU: 12%</span>
          <span>{{ store.panels.length }} 个面板活动中</span>
        </div>
      </footer>
    </main>

    <div v-if="store.isSettingsModalOpen" class="modal-overlay" @click.self="store.closeSettingsModal()">
      <SettingsPanel mode="modal" @close="store.closeSettingsModal()" />
    </div>
  </div>
</template>
