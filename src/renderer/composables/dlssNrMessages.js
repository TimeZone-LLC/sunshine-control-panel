import { enhancementMessages } from './enhancementMessages.js'

export const dlssNrMessages = {
  en: {
    ...enhancementMessages.en,
    acquisitionDescription: 'Choose a locally obtained NVIDIA nvngx_dlssnr.dll. This is separate from RTX HDR; do not select nvngx_truehdr.dll. Import records the file digest. Use Choose again to update it.',
    adapterMissingNotice: 'The Sunshine DLSS NR adapter is missing. Reinstall or update Sunshine, then refresh the component status.',
    bridge: 'Sunshine DLSS NR adapter',
    bridgeMissingNotice: 'This Sunshine installation does not support DLSS NR. Update Sunshine and try again.',
    componentTitle: 'NVIDIA DLSS NR',
    enable: 'Enable DLSS NR',
    enableHint: 'After enabling the component, enable DLSS NR in application settings and reconnect. Native HDR retains HDR output; RTX HDR takes priority when enabled.',
    installConfirm: 'Install the selected nvngx_dlssnr.dll? Administrator permission may be required.',
    installFailed: 'Unable to install the NVIDIA DLSS NR component.',
    installSuccess: 'The component is installed. You can now enable DLSS NR.',
    installTitle: 'Install NVIDIA DLSS NR component',
    intro: 'Experimental enhancement before encoding; compatible Sunshine builds support SDR and native HDR. Adds GPU work and may change faces and textures. Disabled by default; compatible Sunshine builds offer optional optical-flow motion estimation in application settings.',
    repairTitle: 'Replace NVIDIA DLSS NR component',
    runtime: 'NVIDIA DLSS NR component',
    runtimeMissingNotice: 'The NVIDIA DLSS NR runtime is missing. Choose component to import a local nvngx_dlssnr.dll.',
    selectRuntime: 'Choose nvngx_dlssnr.dll',
    title: 'NVIDIA DLSS NR',
    uninstallConfirm: 'Disable DLSS NR and remove the installed NVIDIA component? Your original file will not be changed.',
    uninstallSuccess: 'The component was removed and DLSS NR was disabled.',
    uninstallTitle: 'Remove NVIDIA DLSS NR component',
    appSettingsFailed: 'Could not open application settings. Check that Sunshine is running and try again.',
    appSettings: 'Open application settings',
  },
}

export function dlssNrText() {
  return dlssNrMessages.en
}
