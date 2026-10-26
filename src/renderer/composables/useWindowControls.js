import { ElMessage } from '../components/ui/compact/feedback.js'

/**
 * Window controls composable.
 */
export function useWindowControls(isMaximized) {
  /**
   * Run a window action.
   * @param {string} action - Action name (minimize/hide).
   * @param {string} actionName - Display name for the action.
   */
  const performWindowAction = async (action, actionName) => {
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
      const currentWindow = getCurrentWebviewWindow()
      await currentWindow[action]()
      console.log(`Window ${actionName}`)
    } catch (error) {
      console.error(`Window action "${actionName}" failed:`, error)
      ElMessage.error(`${actionName} failed: ${error.message}`)
    }
  }

  /**
   * Minimise the window.
   */
  const minimizeWindow = async () => {
    await performWindowAction('minimize', 'minimized')
  }

  /**
   * Toggle the maximised state.
   */
  const toggleMaximize = async () => {
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
      const window = getCurrentWebviewWindow()

      const maximized = await window.isMaximized()

      if (maximized) {
        await window.unmaximize()
        isMaximized.value = false
        console.log('Window restored')
      } else {
        await window.maximize()
        isMaximized.value = true
        console.log('Window maximized')
      }
    } catch (error) {
      console.error('Failed to toggle maximize:', error)
      ElMessage.error(`Failed to toggle maximize: ${error}`)
    }
  }

  /**
   * Close the window by hiding it.
   */
  const closeWindow = async () => {
    await performWindowAction('hide', 'hidden')
  }

  return {
    minimizeWindow,
    toggleMaximize,
    closeWindow,
  }
}


