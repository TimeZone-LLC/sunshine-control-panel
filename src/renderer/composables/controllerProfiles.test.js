import assert from 'node:assert/strict'
import test from 'node:test'

import {
  availableGamepadProfiles,
  isGamepadProfileAvailable,
  reconcileGamepadProfile,
} from './controllerProfiles.js'

test('ViGEm-only mode hides Virtual HID-only controller profiles', () => {
  assert.deepEqual(
    availableGamepadProfiles('vigembus'),
    ['auto', 'x360', 'ds4', 'ds5'],
  )
  assert.equal(isGamepadProfileAvailable('xseries', 'vigembus'), false)
  assert.equal(isGamepadProfileAvailable('ds5', 'vigembus'), true)
})

test('Virtual HID policies expose every supported controller profile', () => {
  const expected = ['auto', 'generic', 'x360', 'xone', 'xseries', 'ds4', 'ds5', 'switch']
  assert.deepEqual(availableGamepadProfiles('virtualhid'), expected)
  assert.deepEqual(availableGamepadProfiles('all'), expected)
})

test('switching to ViGEm-only mode reconciles unsupported selections', () => {
  assert.equal(reconcileGamepadProfile('switch', 'vigembus'), 'auto')
  assert.equal(reconcileGamepadProfile('ds4', 'vigembus'), 'ds4')
  assert.equal(reconcileGamepadProfile('xseries', 'virtualhid'), 'xseries')
})
