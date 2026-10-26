import { reactive } from 'vue'

let nextNoticeId = 1
let nextDialogId = 1

export const feedbackState = reactive({
  notices: [],
  dialogs: [],
})

function textFrom(value) {
  if (typeof value === 'string') return value
  if (value instanceof Error) return value.message
  if (value && typeof value.message === 'string') return value.message
  return String(value ?? '')
}

function notify(tone, value) {
  const id = nextNoticeId++
  feedbackState.notices.push({ id, tone, text: textFrom(value) })
  window.setTimeout(() => dismissNotice(id), 4200)
  return { close: () => dismissNotice(id) }
}

export function dismissNotice(id) {
  const index = feedbackState.notices.findIndex((notice) => notice.id === id)
  if (index >= 0) feedbackState.notices.splice(index, 1)
}

export const ElMessage = Object.assign(
  (value) => notify('info', value),
  {
    success: (value) => notify('success', value),
    warning: (value) => notify('warning', value),
    error: (value) => notify('error', value),
    info: (value) => notify('info', value),
    closeAll: () => feedbackState.notices.splice(0),
  },
)

function openDialog(kind, message, title, options = {}) {
  return new Promise((resolve, reject) => {
    feedbackState.dialogs.push({
      id: nextDialogId++,
      kind,
      title: textFrom(title || (kind === 'alert' ? 'Notice' : 'Confirm action')),
      message: textFrom(message),
      confirmText: options.confirmButtonText || 'Continue',
      cancelText: options.cancelButtonText || 'Cancel',
      resolve,
      reject,
    })
  })
}

export function settleDialog(dialog, confirmed) {
  const index = feedbackState.dialogs.findIndex((item) => item.id === dialog.id)
  if (index >= 0) feedbackState.dialogs.splice(index, 1)
  if (confirmed) dialog.resolve('confirm')
  else dialog.reject('cancel')
}

export const ElMessageBox = {
  confirm: (message, title, options) => openDialog('confirm', message, title, options),
  alert: (message, title, options) => openDialog('alert', message, title, options),
}
