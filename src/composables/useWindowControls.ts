import { getCurrentWindow } from '@tauri-apps/api/window'

export function useWindowControls() {
  const isWindowMaximized = ref(false)
  let unlistenResize: null | (() => void) = null

  const syncWindowState = async () => {
    try {
      isWindowMaximized.value = await getCurrentWindow().isMaximized()
    } catch {
      isWindowMaximized.value = false
    }
  }

  const minimizeWindow = async () => {
    await getCurrentWindow().minimize()
  }

  const toggleMaximizeWindow = async () => {
    const currentWindow = getCurrentWindow()
    const maximized = await currentWindow.isMaximized()

    if (maximized) {
      await currentWindow.unmaximize()
    } else {
      await currentWindow.maximize()
    }

    await syncWindowState()
  }

  const closeWindow = async () => {
    await getCurrentWindow().close()
  }

  const startDraggingWindow = async () => {
    await getCurrentWindow().startDragging()
  }

  onMounted(async () => {
    await syncWindowState()
    unlistenResize = await getCurrentWindow().onResized(async () => {
      await syncWindowState()
    })
  })

  onBeforeUnmount(() => {
    unlistenResize?.()
  })

  return {
    isWindowMaximized,
    minimizeWindow,
    toggleMaximizeWindow,
    closeWindow,
    startDraggingWindow,
  }
}
