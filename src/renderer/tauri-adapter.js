/**
 * Tauri API adapter: maps the legacy Electron IPC calls onto Tauri invoke.
 */

import { invoke } from '@tauri-apps/api/core'

// ─── Helpers ────────────────────────────────────────────

/** Invoke a command and return { success, data, message }. */
async function wrapResult(cmd, args) {
  try {
    const data = await invoke(cmd, args)
    return { success: true, data }
  } catch (error) {
    return { success: false, message: error }
  }
}

/** Invoke a command, returning the fallback on failure. */
async function wrapDefault(cmd, fallback, args) {
  try {
    return await invoke(cmd, args)
  } catch (error) {
    console.warn(`${cmd} failed:`, error)
    return fallback
  }
}

// ─── External URLs ───────────────────────────────────────

export function isAllowedExternalUrl(url) {
  return /^(https?:\/\/|ms-windows-store:\/\/)/i.test(String(url || '').trim())
}

export async function openExternalUrl(url) {
  try {
    if (!isAllowedExternalUrl(url)) return false
    return await invoke('open_external_url', { url })
  } catch (error) {
    console.error('Failed to open the external URL:', error)
    return false
  }
}

// ─── Virtual display settings ────────────────────────────

export const vdd = {
  getStatus: () => wrapResult('get_vdd_status'),
  install: () => wrapResult('install_vdd_driver'),
  uninstall: () => wrapResult('uninstall_vdd_driver'),
  setKeepEnabled: (enabled) => wrapResult('set_vdd_keep_enabled', { enabled }),
  setHeadlessCreateEnabled: (enabled) => wrapResult('set_vdd_headless_create_enabled', { enabled }),
  getGPUs: () => wrapResult('get_gpus'),
  loadSettings: () => wrapResult('load_vdd_settings'),
  saveSettings: (settings) => wrapResult('save_vdd_settings', { settings }),
  getEdidFilePath: () => wrapResult('get_vdd_edid_file_path'),
  uploadEdidFile: (fileData) => wrapResult('upload_edid_file', { fileData }),
  readEdidFile: () => wrapResult('read_edid_file'),
  deleteEdidFile: () => wrapResult('delete_edid_file'),
  getTraceStatus: () => wrapResult('get_vdd_trace_status'),
  startTrace: () => wrapResult('start_vdd_trace'),
  stopTrace: () => wrapResult('stop_vdd_trace'),
  openTraceFolder: () => wrapResult('open_vdd_trace_folder'),
  launchHdrCalibration: () => wrapResult('launch_windows_hdr_calibration'),

  async execVddCmd(command) {
    try {
      await invoke('exec_vdd_cmd', { command })
      return true
    } catch (error) {
      console.error('Virtual display command failed:', error)
      throw error instanceof Error
        ? error
        : new Error(typeof error === 'string' && error.trim() ? error : 'Virtual display command failed')
    }
  },
}

// ─── Virtual Mouse ───────────────────────────────────────

export const vmouse = {
  getStatus: () => wrapResult('get_vmouse_status'),
  install: () => wrapResult('install_vmouse_driver'),
  uninstall: () => wrapResult('uninstall_vmouse_driver'),
  setConfig: (enabled) => wrapResult('set_vmouse_config', { enabled }),
}

// ─── ViGEmBus Virtual Gamepad ──────────────────────

export const vigem = {
  getStatus: () => wrapResult('get_vigem_status'),
  install: (force = false) => wrapResult('install_vigem_driver', { force: !!force }),
  uninstall: () => wrapResult('uninstall_vigem_driver'),
}

// Controller and peripheral management.
export function controllerHubCommandArgs(patch = {}) {
  return {
    gamepad: patch.gamepad,
    gamepadDriver: patch.gamepad_driver,
    motionAsDs4: patch.motion_as_ds4,
    touchpadAsDs4: patch.touchpad_as_ds4,
    ds4BackAsTouchpadClick: patch.ds4_back_as_touchpad_click,
    ds5InputtinoRandomizeMac: patch.ds5_inputtino_randomize_mac,
    enableDsuServer: patch.enable_dsu_server,
    dsuServerPort: patch.dsu_server_port,
  }
}

export const controllerHub = {
  getConfig: () => wrapResult('get_controller_hub_config'),
  saveConfig: (patch = {}) => wrapResult(
    'save_controller_hub_config',
    controllerHubCommandArgs(patch),
  ),
}

export const virtualMicrophone = {
  getStatus: () => wrapResult('get_virtual_microphone_status'),
  test: () => wrapResult('test_virtual_microphone'),
}

export const dualsense = {
  getStatus: () => wrapResult('dualsense_get_status'),
  logPanelOpened: () => wrapResult('dualsense_log_panel_opened'),
  install: (packagePaths = []) => wrapResult('dualsense_install', { packagePaths }),
  uninstall: () => wrapResult('dualsense_uninstall'),
  setConfig: (audioHaptics, genshinCompatibility) => wrapResult('dualsense_set_config', {
    audioHaptics: !!audioHaptics,
    genshinCompatibility: !!genshinCompatibility,
  }),
  setHapticsTuning: (strength, curve, noiseGate) => wrapResult('dualsense_set_haptics_tuning', {
    strength,
    curve,
    noiseGate,
  }),
  selfTest: (profile) => wrapResult('dualsense_self_test', { profile }),
}

export const usbip = {
  getForwardingConfig: () => wrapResult('usbip_get_forwarding_config'),
  saveForwardingConfig: (enabled, port) => wrapResult('usbip_save_forwarding_config', { enabled, port }),
  getStatus: () => wrapResult('usbip_get_status'),
  installTransport: () => wrapResult('usbip_install_transport'),
  cleanupTransport: () => wrapResult('usbip_cleanup_transport'),
  listRemote: (remote, tcpPort = 3240) => wrapResult('usbip_list_remote', { remote, tcpPort }),
  attach: (remote, busId, tcpPort = 3240) => wrapResult('usbip_attach', { remote, busId, tcpPort }),
  detach: (port) => wrapResult('usbip_detach', { port }),
}

export const rtxHdr = {
  recover: () => wrapResult('native_component_recover', { componentId: 'alkaidlab.nvidia_rtx_video' }),
  setEnabled: (enabled) => wrapResult('hdr_enhanced_select_backend', { componentId: 'alkaidlab.nvidia_rtx_video', enabled }),
  getStatus: () => wrapResult('native_component_get_status', { componentId: 'alkaidlab.nvidia_rtx_video' }),
  install: (runtimePath) => wrapResult('native_component_import', {
    componentId: 'alkaidlab.nvidia_rtx_video',
    sources: { 'nvngx_truehdr.dll': runtimePath },
  }),
  uninstall: () => wrapResult('native_component_remove', { componentId: 'alkaidlab.nvidia_rtx_video' }),
}

export const dlssNr = {
  recover: () => wrapResult('native_component_recover', { componentId: 'alkaidlab.nvidia_dlssnr' }),
  setEnabled: (enabled) => wrapResult('hdr_enhanced_select_backend', { componentId: 'alkaidlab.nvidia_dlssnr', enabled }),
  getStatus: () => wrapResult('native_component_get_status', { componentId: 'alkaidlab.nvidia_dlssnr' }),
  install: (runtimePath) => wrapResult('native_component_import', {
    componentId: 'alkaidlab.nvidia_dlssnr',
    sources: { 'nvngx_dlssnr.dll': runtimePath },
  }),
  uninstall: () => wrapResult('native_component_remove', { componentId: 'alkaidlab.nvidia_dlssnr' }),
}

// ─── Sunshine configuration ──────────────────────────────

export const sunshine = {
  parseConfig: () => wrapDefault('parse_sunshine_config', {}),
  getUrl: () => wrapDefault('get_sunshine_url', 'https://localhost:47990/'),
  getCommandLineUrl: () => wrapDefault('get_command_line_url', null),
  getProxyUrl: () => wrapDefault('get_proxy_url_command', 'http://localhost:48081'),
  waitForProxyReady: () => wrapResult('wait_for_proxy_ready'),
  refreshTarget: () => wrapDefault('refresh_sunshine_target', null),
  getActiveSessions: () => wrapDefault('get_active_sessions', []),
  changeBitrate: (clientName, bitrate) => invoke('change_bitrate', { clientName, bitrate }),
  stopAllSessions: () => invoke('stop_all_sessions'),
}

// ─── System tools ────────────────────────────────────────

export const tools = {
  restartGraphicsDriver: () => invoke('restart_graphics_driver'),
  restartSunshineService: () => invoke('restart_sunshine_service'),
  restartSunshineInUserMode: () => invoke('restart_sunshine_in_user_mode'),
  uninstallVddDriver: () => invoke('uninstall_vdd_driver'),
  openNativeTool: (toolId) => invoke('open_native_tool', { toolId }),
}

// Folder Sharing

export const fileMapping = {
  list: () => invoke('list_file_mappings'),
  quickShareFolder: (path) => invoke('quick_share_folder', { path }),
  remove: (id) => invoke('delete_file_mapping', { id }),
  update: (id, patch) => invoke('update_file_mapping', { id, patch }),
  installMenu: () => invoke('install_file_mapping_menu'),
  uninstallMenu: () => invoke('uninstall_file_mapping_menu'),
}

// ─── Moonlight Web ───────────────────────────────────────

export const moonlightWeb = {
  getStatus: () => wrapDefault('moonlight_web_get_status',
    { installed: false, running: false, install_path: '', version: '', access_url: '', port: 8080 }),
  getConfig: () => wrapDefault('moonlight_web_get_config',
    { web_server: { bind_address: '0.0.0.0:8080' }, webrtc: null, default_settings: null }),
  start: () => invoke('moonlight_web_start'),
  stop: () => invoke('moonlight_web_stop'),
  saveConfig: (config) => invoke('moonlight_web_save_config', { config }),
  getInstallPath: () => invoke('moonlight_web_get_install_path'),
  generateCert: () => invoke('moonlight_web_generate_cert'),
}

// ─── ControllerMeta ──────────────────────────────────────

export const controllerMeta = {
  getStatus: () => wrapDefault('controllermeta_get_status',
    { installed: false, running: false, install_path: '', binary_path: '', version: '' }),
  launch: () => invoke('controllermeta_launch'),
  getInstallPath: () => invoke('controllermeta_get_install_path'),
  uninstall: () => invoke('controllermeta_uninstall'),
}

// ─── Exports ─────────────────────────────────────────────

export default {
  openExternalUrl,
  vdd,
  vmouse,
  vigem,
  controllerHub,
  virtualMicrophone,
  dualsense,
  usbip,
  rtxHdr,
  dlssNr,
  sunshine,
  tools,
  fileMapping,
  moonlightWeb,
  controllerMeta,
}
