<script setup lang="ts">
import DashboardView from '@/components/DashboardView.vue'
import LoginView from '@/components/LoginView.vue'

const store = useStore()
store.initApp()
</script>

<template>
  <div
    class="app-shell min-h-screen"
    :class="[store.themeClass, { 'has-acrylic': store.settings.acrylicEnabled }]"
    :style="{
      '--ui-radius': `${store.currentRadius}px`,
      '--ui-transparency': `${store.settings.transparency / 100}`,
    }"
  >
    <LoginView v-if="!store.isAuthenticated" />
    <DashboardView v-else />
  </div>
</template>
