<script setup lang="ts">
import { useWindowControls } from '@/composables/useWindowControls'

const store = useStore()
const { isWindowMaximized, minimizeWindow, toggleMaximizeWindow, closeWindow } = useWindowControls()

const canSubmit = computed(() => store.loginForm.username.trim() && store.loginForm.password.trim())
</script>

<template>
  <main class="login-page">
    <div class="hero-orb hero-orb-left"></div>
    <div class="hero-orb hero-orb-right"></div>

    <div class="login-layout">
      <header class="login-header window-drag-region" data-tauri-drag-region>
        <div class="flex items-center gap-3">
          <div class="size-8 text-[var(--brand)] flex items-center justify-center">
            <span class="i-mdi-view-grid text-[28px]"></span>
          </div>
          <h2 class="text-lg font-bold tracking-tight text-[var(--text-strong)]">聚合浏览器</h2>
        </div>

        <div class="window-controls">
          <button class="window-control-button login-window-button" type="button" @click="minimizeWindow">
            <span class="i-mdi-window-minimize"></span>
          </button>
          <button class="window-control-button login-window-button" type="button" @click="toggleMaximizeWindow">
            <span :class="isWindowMaximized ? 'i-mdi-window-restore' : 'i-mdi-checkbox-blank-outline'"></span>
          </button>
          <button class="window-control-button login-window-button window-control-button--close" type="button" @click="closeWindow">
            <span class="i-mdi-close"></span>
          </button>
        </div>
      </header>

      <section class="login-main">
        <div class="login-card">
          <div class="login-card__hero">
            <div class="login-avatar">
              <span class="i-mdi-account-circle text-5xl"></span>
            </div>
            <h1 class="text-2xl font-bold text-[var(--text-strong)]">欢迎回来</h1>
            <p class="text-sm text-[var(--text-soft)]">登录您的聚合浏览器账号以同步数据</p>
          </div>

          <form class="space-y-4" @submit.prevent="store.login">
            <label class="field">
              <span class="field-label">用户名 / 邮箱</span>
              <div class="field-input field-input--design">
                <span class="i-mdi-email-outline text-lg text-[var(--text-muted)]"></span>
                <input v-model="store.loginForm.username" type="text" placeholder="请输入您的用户名或邮箱" />
              </div>
            </label>

            <label class="field">
              <div class="flex justify-between items-center mb-2">
                <span class="field-label !mb-0">密码</span>
                <button type="button" class="text-xs text-[var(--brand)] font-medium hover:underline">忘记密码？</button>
              </div>
              <div class="field-input field-input--design">
                <span class="i-mdi-lock-outline text-lg text-[var(--text-muted)]"></span>
                <input v-model="store.loginForm.password" type="password" placeholder="请输入您的密码" />
                <button type="button" class="text-[var(--text-muted)] hover:text-[var(--text-strong)]">
                  <span class="i-mdi-eye-outline text-lg"></span>
                </button>
              </div>
            </label>

            <div class="flex items-center gap-2 py-1">
              <input v-model="store.loginForm.remember" id="remember-login" class="w-4 h-4 rounded accent-[var(--brand)]" type="checkbox" />
              <label class="text-sm text-[var(--text-soft)] cursor-pointer" for="remember-login">下次自动登录</label>
            </div>

            <button class="primary-button login-submit w-full" :disabled="!canSubmit || store.isLoginSubmitting" type="submit">
              <span v-if="store.isLoginSubmitting">登录中...</span>
              <span v-else>立即登录</span>
            </button>
          </form>

          <p class="text-center text-sm text-[var(--text-soft)]">
            还没有账号？ <button type="button" class="text-[var(--brand)] font-semibold hover:underline">立即注册</button>
          </p>
        </div>
      </section>

      <footer class="login-footer">
        <div>© 2024 聚合浏览器. 保留所有权利.</div>
        <div class="flex gap-4">
          <button type="button" class="hover:text-[var(--brand)] transition-colors">隐私政策</button>
          <button type="button" class="hover:text-[var(--brand)] transition-colors">服务条款</button>
          <button type="button" class="hover:text-[var(--brand)] transition-colors">帮助中心</button>
        </div>
      </footer>
    </div>
  </main>
</template>
