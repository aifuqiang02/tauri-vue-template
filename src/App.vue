<script setup lang="ts">
import DashboardView from '@/components/DashboardView.vue'
import LoginView from '@/components/LoginView.vue'
import { onBeforeUnmount, onMounted, watch } from 'vue'

const store = useStore()
store.initApp()

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key !== 'F12') {
    return
  }

  event.preventDefault()
  void store.openDevtools()
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  store.stopAutoUpdateChecks()
})

watch(
  () => store.isAuthenticated,
  (isAuthenticated) => {
    if (isAuthenticated) {
      store.startAutoUpdateChecks()
      return
    }

    store.stopAutoUpdateChecks()
  },
  { immediate: true },
)
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
