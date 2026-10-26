import assert from 'node:assert/strict'
import test from 'node:test'

import { rtxHdrMessages } from './rtxHdrMessages.js'
import { en } from '../i18n/en.js'

test('HDR management is vendor-neutral while the backend keeps its identity', () => {
  assert.ok(en.sidebar.hdrEnhanced.length > 0)
  assert.ok(!en.sidebar.hdrEnhanced.includes('RTX'))
  assert.equal(en.hdrEnhanced.description, undefined)
  assert.equal(rtxHdrMessages.en.title, 'NVIDIA RTX HDR')
})

test('RTX HDR component copy explains how to resolve missing files', () => {
  const text = rtxHdrMessages.en
  assert.ok(text.title.includes('RTX HDR'))
  assert.match(text.acquisitionDescription, /nvngx_truehdr\.dll/)
  assert.match(text.bridgeMissingNotice, /reinstall/i)
  assert.match(text.runtimeMissingNotice, /nvngx_truehdr\.dll/)
  assert.match(text.vcRuntimeMissingNotice, /Visual C\+\+/)
  assert.equal(text.ownershipTitle, undefined)
  assert.equal(text.securityTitle, undefined)
})

test('RTX HDR component copy covers every manager state', () => {
  assert.deepEqual(
    Object.keys(rtxHdrMessages.en.states).sort(),
    ['active', 'configured', 'degraded', 'in_use', 'loading', 'maintenance_other', 'not_installed', 'pending_removal', 'repair_required', 'selected', 'unavailable'],
  )
})
