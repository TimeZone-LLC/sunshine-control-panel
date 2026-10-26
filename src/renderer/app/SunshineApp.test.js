import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const shell = readFileSync(new URL('./SunshineApp.vue', import.meta.url), 'utf8')
const shellStyles = readFileSync(new URL('./app.css', import.meta.url), 'utf8')
const controllers = readFileSync(new URL('../components/controllersHub/ControllersHub.vue', import.meta.url), 'utf8')
const displays = readFileSync(new URL('../components/VddSettings.vue', import.meta.url), 'utf8')
const image = readFileSync(new URL('../components/HdrEnhancedManager.vue', import.meta.url), 'utf8')

test('the main shell exposes every destination in one persistent navigation surface', () => {
  assert.equal(shell.match(/<nav\b/g)?.length, 1)
  assert.match(shell, /class="primary-navigation"/)
  assert.match(shell, /class="primary-navigation-group"/)
  assert.match(shell, /v-for="item in section\.items"/)
  assert.match(shell, /@click="selectView\(item\)"/)
  assert.doesNotMatch(shell, /class="task-index"/)
  assert.doesNotMatch(shell, /selectSection|activeSectionId|activeView\.value = null/)
})

test('the status area has no reload slot or decorative separators', () => {
  assert.doesNotMatch(shell, /app-reload|reloadFrame|RefreshCw/)
  assert.doesNotMatch(shellStyles, /\.app-reload/)
  assert.doesNotMatch(shellStyles, /\.host-summary\s*\{[^}]*border-left/s)
  assert.doesNotMatch(shellStyles, /\.host-session-count\s*\{[^}]*border-left/s)
})

test('the main shell does not expose upstream build identity', () => {
  assert.doesNotMatch(shell, /getVersion|brand-version|app_version|build_time/)
})

test('major tool surfaces do not restore internal tab bars', () => {
  for (const surface of [controllers, displays, image]) {
    assert.doesNotMatch(surface, /role="tablist"|class="[^"]*tabs[^"]*"/)
  }
})

test('controllers exposes every top-level section in one lifecycle-safe document flow', () => {
  assert.doesNotMatch(controllers, /device-hub-section|activeTab/)
  assert.equal(controllers.match(/class="chub-workflow-section"/g)?.length, 5)
  assert.match(controllers, /@manage-components="focusRuntimeComponents"/)
  assert.match(controllers, /:settings="dualSenseSettings"/)
  assert.match(controllers, /:component="dualSenseSettings\.component"/)
  assert.match(controllers, /:transport="usbTransport"/)
  assert.equal(controllers.match(/useUsbTransport\(\)/g)?.length, 1)
  assert.equal(controllers.match(/useDualSenseSettings\(\)/g)?.length, 1)
})

test('image tools expose every top-level section in one document flow', () => {
  assert.doesNotMatch(image, /quality-section-control|v-show="tab ===|switchTab\(/)
  for (const id of ['quality-components', 'quality-session', 'quality-settings']) {
    assert.match(image, new RegExp(`id="${id}"`))
  }
})

test('display tools expose every top-level section in one document flow', () => {
  assert.doesNotMatch(displays, /vdd-section-control|activeSection|sectionOptions/)
  for (const id of [
    'vdd-panel-overview',
    'vdd-panel-calibration',
    'vdd-panel-display',
    'vdd-panel-driver',
  ]) {
    assert.match(displays, new RegExp(`id="${id}"`))
  }
})
