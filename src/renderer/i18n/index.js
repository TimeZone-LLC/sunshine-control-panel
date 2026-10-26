import { computed } from 'vue'
import { en } from './en.js'

document.documentElement.lang = 'en'

export function useI18n() {
  return {
    t: computed(() => en),
    locale: computed(() => 'en'),
  }
}
