export const GAMEPAD_PROFILES = Object.freeze([
  'auto',
  'generic',
  'x360',
  'xone',
  'xseries',
  'ds4',
  'ds5',
  'switch',
])

const VIGEM_ONLY_PROFILES = new Set(['auto', 'x360', 'ds4', 'ds5'])

export function availableGamepadProfiles(driver) {
  return driver === 'vigembus'
    ? GAMEPAD_PROFILES.filter((profile) => VIGEM_ONLY_PROFILES.has(profile))
    : [...GAMEPAD_PROFILES]
}

export function isGamepadProfileAvailable(profile, driver) {
  return availableGamepadProfiles(driver).includes(profile)
}

export function reconcileGamepadProfile(profile, driver) {
  return isGamepadProfileAvailable(profile, driver) ? profile : 'auto'
}
