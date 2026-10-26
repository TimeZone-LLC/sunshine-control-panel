const PAIRING_ID_PATTERN = /^[0-9a-f]{32}$/i
const PIN_PATTERN = /^\d{4}$/

export function isSuccessfulPairingStatus(status) {
  const normalized = String(status ?? '').toLowerCase()
  return normalized === '1' || normalized === 'true'
}

export function normalizePendingPairings(value) {
  if (!Array.isArray(value)) return []

  return value
    .filter((pairing) => pairing && PAIRING_ID_PATTERN.test(String(pairing.id ?? '')))
    .map((pairing) => ({
      id: String(pairing.id),
      name: String(pairing.name ?? '').trim(),
      address: String(pairing.address ?? '').trim(),
    }))
}

export function choosePendingPairing(pairings, currentId = '') {
  return pairings.find((pairing) => pairing.id === currentId) ?? pairings[0] ?? null
}

export function pairingRequestLabel(pairing) {
  if (!pairing) return ''
  const name = pairing.name || 'Moonlight client'
  return pairing.address ? `${name} · ${pairing.address}` : name
}

async function readJsonResponse(response, action) {
  if (!response.ok) throw new Error(`${action} failed (${response.status})`)
  return response.json()
}

export async function getPendingPairings(proxyUrl, fetcher = fetch) {
  const response = await fetcher(`${proxyUrl}/api/pin`)
  const result = await readJsonResponse(response, 'Loading pairing requests')
  return normalizePendingPairings(result.pairings)
}

export async function approvePendingPairing(
  proxyUrl,
  { pairingId, pin, name },
  fetcher = fetch,
) {
  if (!PAIRING_ID_PATTERN.test(pairingId)) throw new Error('The pairing request is no longer available.')
  if (!PIN_PATTERN.test(pin)) throw new Error('Enter the four-digit PIN shown by Moonlight.')

  const response = await fetcher(`${proxyUrl}/api/pin`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      pairing_id: pairingId,
      pin,
      name: String(name || 'Moonlight client').trim() || 'Moonlight client',
    }),
  })
  const result = await readJsonResponse(response, 'Pairing request')
  if (!isSuccessfulPairingStatus(result.status)) throw new Error(result.error || 'The PIN was rejected.')
  return result
}

export async function cancelPendingPairing(proxyUrl, pairingId, fetcher = fetch) {
  if (!PAIRING_ID_PATTERN.test(pairingId)) return false

  const response = await fetcher(`${proxyUrl}/api/pin`, {
    method: 'DELETE',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ pairing_id: pairingId }),
  })
  const result = await readJsonResponse(response, 'Cancelling pairing request')
  return isSuccessfulPairingStatus(result.status)
}
