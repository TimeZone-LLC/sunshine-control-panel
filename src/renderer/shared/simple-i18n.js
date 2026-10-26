/**
 * English-only message helper.
 *
 * This panel used to negotiate a locale from localStorage, the browser and the
 * Sunshine host, and shipped a parallel Chinese catalogue. The app is English
 * only now, so there is NO language switching left: every export below is a
 * shim that hard-returns English.
 *
 * The shims are deliberately kept rather than deleted, because call sites
 * across the renderer still import them. Anything here is safe to call; nothing
 * here can change the language.
 *
 * Shape you can rely on:
 *   LOCALE                       -> 'en'
 *   normalizeSimpleLocale(any)   -> 'en'
 *   getStoredSimpleLocale()      -> 'en'
 *   getSystemSimpleLocale()      -> 'en'
 *   getInitialSimpleLocale()     -> 'en'
 *   setDocumentLocale()          -> sets <html lang="en">, returns 'en'
 *   pickSimpleMessages(msgs)     -> msgs.en (or msgs itself if it has no .en)
 *   syncSimpleLocaleFromSunshine(msgs, apply) -> calls apply('en'), resolves
 *   registerMessages(msgs)       -> installs the catalogue used by t()
 *   t(key, params)               -> English string, '{name}' interpolation
 *   translate(key, params)       -> alias of t
 *   useSimpleI18n(msgs?)         -> { locale: 'en', t, translate, messages }
 */

export const LOCALE = 'en'

/** Every locale normalises to English. */
export function normalizeSimpleLocale() {
  return LOCALE
}

export function getStoredSimpleLocale() {
  return LOCALE
}

export function getSystemSimpleLocale() {
  return LOCALE
}

export function getInitialSimpleLocale() {
  return LOCALE
}

/** Stamps <html lang="en">. Accepts and ignores a locale argument. */
export function setDocumentLocale() {
  if (typeof document !== 'undefined' && document.documentElement) {
    document.documentElement.lang = LOCALE
  }
  return LOCALE
}

/**
 * Selects the English branch of a `{ en: {...} }` catalogue. A flat catalogue
 * with no `en` key is returned unchanged, so both shapes work.
 */
export function pickSimpleMessages(messages) {
  if (!messages) return {}
  return messages[LOCALE] ?? messages
}

/**
 * There is nothing to negotiate with the host any more; this just makes sure
 * the caller still applies its own catalogue.
 */
export async function syncSimpleLocaleFromSunshine(messages, apply) {
  if (typeof apply === 'function') apply(LOCALE)
  return LOCALE
}

// ---------------------------------------------------------------------------
// t() shim
// ---------------------------------------------------------------------------

let catalogue = {}

/** Installs the catalogue that t() reads. Accepts `{ en: {...} }` or a flat object. */
export function registerMessages(messages) {
  catalogue = pickSimpleMessages(messages) || {}
  return catalogue
}

function lookup(source, key) {
  if (!source || typeof key !== 'string') return undefined
  if (key in source) return source[key]
  // Dot paths, e.g. t('pin.title').
  return key.split('.').reduce((acc, part) => (acc == null ? undefined : acc[part]), source)
}

/**
 * Resolves `key` against the registered catalogue and interpolates `{name}`
 * placeholders. Unknown keys fall back to the key itself, so a missing entry
 * shows up as readable text instead of blanking the UI.
 */
export function t(key, params) {
  const found = lookup(catalogue, key)
  let out = typeof found === 'string' ? found : key

  if (params && typeof params === 'object') {
    out = out.replace(/\{(\w+)\}/g, (match, name) =>
      Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : match
    )
  }

  return out
}

/** Alias, for call sites that import `translate`. */
export const translate = t

/**
 * Composable-shaped accessor for components that expect one. The returned
 * `locale` is a plain string, not a ref - it can never change.
 */
export function useSimpleI18n(messages) {
  if (messages) registerMessages(messages)
  return { locale: LOCALE, messages: catalogue, t, translate: t }
}

export default {
  LOCALE,
  normalizeSimpleLocale,
  getStoredSimpleLocale,
  getSystemSimpleLocale,
  getInitialSimpleLocale,
  setDocumentLocale,
  pickSimpleMessages,
  syncSimpleLocaleFromSunshine,
  registerMessages,
  t,
  translate,
  useSimpleI18n,
}
