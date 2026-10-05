import { computed } from 'vue'
import { rtxHdrMessages } from './rtxHdrMessages.js'

export function useRtxHdrI18n() {
  return computed(() => rtxHdrMessages.en)
}
