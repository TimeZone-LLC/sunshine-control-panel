import { enhancementMessages } from './enhancementMessages.js'

export const rtxHdrMessages = {
  en: {
    ...enhancementMessages.en,
    acquisitionDescription: 'Get and extract the NVIDIA RTX Video SDK, then choose bin\\Windows\\x64\\rel\\nvngx_truehdr.dll on this page.',
    adapterMissingNotice: 'The Sunshine HDR adapter is missing. Reinstall or update Sunshine, then refresh the component status.',
    bridge: 'Sunshine HDR adapter',
    bridgeMissingNotice: 'This Sunshine installation does not include HDR enhancement support. Reinstall or update Sunshine, then try again.',
    componentTitle: 'NVIDIA RTX HDR',
    enable: 'Enable RTX HDR',
    enableHint: 'Changes take effect the next time you connect a stream.',
    installConfirm: 'Install the selected nvngx_truehdr.dll? Administrator permission may be required.',
    installFailed: 'Unable to install the NVIDIA RTX HDR component.',
    installSuccess: 'The component is installed. You can now enable RTX HDR.',
    installTitle: 'Install NVIDIA RTX HDR component',
    intro: 'Install the required NVIDIA component, then enable RTX HDR for supported streams.',
    repairTitle: 'Replace NVIDIA RTX HDR component',
    runtime: 'NVIDIA RTX HDR component',
    runtimeMissingNotice: 'The NVIDIA RTX HDR component is missing. Choose nvngx_truehdr.dll from the NVIDIA RTX Video SDK to install it.',
    selectRuntime: 'Choose nvngx_truehdr.dll',
    title: 'NVIDIA RTX HDR',
    uninstallConfirm: 'Disable RTX HDR and remove the installed NVIDIA component? Your original file will not be changed.',
    uninstallSuccess: 'The component was removed and RTX HDR was disabled.',
    uninstallTitle: 'Remove NVIDIA RTX HDR component',
  },
}
