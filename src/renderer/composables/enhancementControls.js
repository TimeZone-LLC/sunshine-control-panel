import { displayGamepadShortcut } from './controllerButtons.js'

export const DEFAULT_NR_SHORTCUT = 'Ctrl+Alt+KeyN'

export function displayShortcut(value) {
  return (value || '')
    .split('+')
    .map((key) => key
      .replace(/^Key(?=[A-Z]$)/, '')
      .replace(/^Digit(?=\d$)/, '')
      .replace(/^control$/i, 'Ctrl')
      .replace(/^alt$/i, 'Alt')
      .replace(/^shift$/i, 'Shift'))
    .join(' + ')
}

export function nrShortcutLabel(status) {
  if (status?.nrRegistered) return displayShortcut(status.settings?.nrShortcut)
  return status?.gamepadSupported ? displayGamepadShortcut(status.settings?.nrGamepad) : ''
}

// Physical key codes match the native global-shortcut parser on non-US layouts.
export function shortcutFromEvent(event) {
  if (event.key === 'Escape' || event.key === 'Tab') return { cancel: true }
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return { modifier: true }
  if (event.metaKey || !(event.ctrlKey || event.altKey || event.shiftKey)) {
    return { error: 'nr_shortcut_invalid' }
  }
  if (!/^(Key[A-Z]|Digit[0-9]|F([1-9]|1[0-9]|2[0-4])|Arrow(Up|Down|Left|Right)|Space|Enter|Backspace|Delete|Insert|Home|End|PageUp|PageDown|Minus|Equal|BracketLeft|BracketRight|Backslash|Semicolon|Quote|Comma|Period|Slash|Backquote)$/.test(event.code)) {
    return { error: 'nr_shortcut_invalid' }
  }
  return {
    value: [event.ctrlKey && 'Ctrl', event.altKey && 'Alt', event.shiftKey && 'Shift', event.code]
      .filter(Boolean)
      .join('+'),
  }
}

const text = {
  stream: 'Stream',
  connected: 'Connected',
  maintenance: 'Installation and maintenance',
  replaceFile: 'Replace component file',
  installSetup: 'Install and set up',
  allowUse: 'Allow streams to use ',
  nrPermissionHint: 'Makes the feature available; it does not indicate an active effect. Set app defaults in application settings, or toggle the current stream live.',
  hdrPermissionHint: 'Allows apps to use SDR-to-HDR conversion. Per-app settings take effect on the next connection.',
  appDefaults: 'Application defaults',
  setupHint: 'Install the enhancement component, then choose which streams use it.',
  setupHelp: 'Installation requirements',
  usageNotes: 'Usage notes',
  permissionSaved: 'Feature setting saved. App defaults and live-stream effects are managed separately.',
  needsInstall: 'Not installed',
  needsSetup: 'Setup required',
  ready: 'Ready',
  installedDisabled: 'Installed · disabled',
  liveTitle: 'DLSS NR enhancement',
  liveHint: 'Apply now to this stream only',
  scaleHint: 'A lower scale reduces GPU cost, but may reduce detail.',
  title: 'Image enhancement',
  intro: 'Components, live effects and overlay controls',
  components: 'Enhancements',
  session: 'Current streams',
  settings: 'Overlay and shortcuts',
  componentHint: 'Default effects are configured per app. Manage components and live controls here.',
  hdrIntro: 'Convert SDR content to HDR output.',
  nrIntro: 'Enhance details in SDR and native HDR content.',
  show: 'Show enhancement overlay',
  showHint: 'Adjust DLSS NR during a stream',
  opacity: 'Default background opacity',
  keys: 'Global shortcuts',
  keyboard: 'Keyboard',
  gamepad: 'Gamepad',
  gamepadUnsupported: 'Background gamepad shortcuts are unavailable on this platform',
  cancel: 'Cancel recording',
  gamepadHint: 'Windows Xbox and XInput controllers, including virtual streaming controllers. Hold LB, RB and the selected X or Y button for half a second; release all buttons before using again. Games still receive these buttons. Once configured, a shortcut button used alone navigates on release in desktop mode.',
  reset: 'Restore defaults',
  visibilityKey: 'Show or hide overlay',
  nrKey: 'Toggle DLSS NR',
  unset: 'Not set',
  clear: 'Clear',
  record: 'Press a shortcut…',
  recordHint: 'Use Ctrl, Alt or Shift. Esc cancels; capture expires after 30 seconds.',
  keyHint: 'Works with the overlay hidden. Only affects the selected stream.',
  saved: 'Saved',
  local: 'Local settings · Saved automatically',
  preview: 'Overlay preview',
  hidden: 'Overlay hidden',
  previewHint: 'Shortcuts work independently of visibility. Appearance preview only.',
  previewState: 'Example NR state',
  choose: 'Select a target stream',
  expired: 'The selected stream ended. Select another stream.',
  noSession: 'Waiting for a stream',
  disconnected: 'Host disconnected',
  refresh: 'Retry',
  sessionHint: 'Changes affect the selected session only, not app defaults.',
  output: 'Output format',
  scale: 'Processing scale',
  enhancement: 'Enhancement',
  open: 'Open live overlay',
  statusUnavailable: 'Shortcut unavailable. Choose another combination or save to retry.',
  disabledKey: 'Shortcut not set',
  action: 'Toggle enhancement',
  errors: {
    nr_request_invalid: 'Processing scale must be 20%–100% and a multiple of 5.',
    nr_gamepad_invalid: 'Only LB+RB+X or LB+RB+Y is supported for gamepad shortcuts',
    nr_shortcut_invalid: 'Use a valid Ctrl, Alt or Shift shortcut',
    nr_shortcut_duplicate: 'The two actions need different shortcuts',
    nr_shortcut_conflict: 'Shortcut is already in use. Previous settings are retained.',
    nr_shortcut_restore_failed: 'Could not restore shortcuts. Reconfigure them; actual registration status is shown below.',
    nr_shortcut_unregister: 'Could not unregister the old shortcut. Settings were not saved.',
    nr_select_session: 'Select the target stream first',
    nr_session_ended: 'The selected stream ended. Select another stream.',
    nr_no_session: 'No active stream',
    nr_settings_pending: 'Settings are being applied. Try again shortly.',
    nr_toggle_unsupported: 'NR switching is unavailable for this stream',
  },
  failed: 'Operation failed. Try again.',
}

export function enhancementControlsText() {
  return text
}

export function enhancementError(messages, error) {
  return messages.errors[String(error)] || messages.failed
}
