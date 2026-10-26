import test from 'node:test'
import assert from 'node:assert/strict'
import {
  approvePendingPairing,
  cancelPendingPairing,
  choosePendingPairing,
  getPendingPairings,
  normalizePendingPairings,
  pairingRequestLabel,
} from './pinPairing.js'

const jsonResponse = (body, { ok = true, status = 200 } = {}) => ({
  ok,
  status,
  json: async () => body,
})

test('normalizes and retains only bounded pairing requests', () => {
  assert.deepEqual(
    normalizePendingPairings([
      { id: 'a'.repeat(32), name: ' Living room ', address: ' 192.0.2.10 ' },
      { id: 'not-an-id', name: 'Invalid' },
      null,
    ]),
    [{ id: 'a'.repeat(32), name: 'Living room', address: '192.0.2.10' }],
  )
})

test('keeps an existing selection and otherwise selects the first request', () => {
  const pairings = [
    { id: 'a'.repeat(32), name: 'First', address: '' },
    { id: 'b'.repeat(32), name: 'Second', address: '192.0.2.20' },
  ]

  assert.equal(choosePendingPairing(pairings, 'b'.repeat(32)).id, 'b'.repeat(32))
  assert.equal(choosePendingPairing(pairings, 'c'.repeat(32)).id, 'a'.repeat(32))
  assert.equal(choosePendingPairing([], 'a'.repeat(32)), null)
  assert.equal(pairingRequestLabel(pairings[1]), 'Second · 192.0.2.20')
})

test('loads pending requests with GET', async () => {
  const calls = []
  const pairings = await getPendingPairings('http://127.0.0.1:48081', async (...args) => {
    calls.push(args)
    return jsonResponse({ pairings: [{ id: 'c'.repeat(32), name: 'Laptop', address: '' }] })
  })

  assert.deepEqual(calls, [['http://127.0.0.1:48081/api/pin']])
  assert.equal(pairings[0].name, 'Laptop')
})

test('approves the selected request with the current host contract', async () => {
  let request
  await approvePendingPairing(
    'http://127.0.0.1:48081',
    { pairingId: 'd'.repeat(32), pin: '1234', name: 'Tablet' },
    async (url, options) => {
      request = { url, options }
      return jsonResponse({ status: true })
    },
  )

  assert.equal(request.url, 'http://127.0.0.1:48081/api/pin')
  assert.equal(request.options.method, 'POST')
  assert.deepEqual(JSON.parse(request.options.body), {
    pairing_id: 'd'.repeat(32),
    pin: '1234',
    name: 'Tablet',
  })
})

test('cancels only the selected pending request', async () => {
  let request
  assert.equal(
    await cancelPendingPairing('http://127.0.0.1:48081', 'e'.repeat(32), async (url, options) => {
      request = { url, options }
      return jsonResponse({ status: 'true' })
    }),
    true,
  )

  assert.equal(request.options.method, 'DELETE')
  assert.deepEqual(JSON.parse(request.options.body), { pairing_id: 'e'.repeat(32) })
})
