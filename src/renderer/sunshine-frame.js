import { createApp } from 'vue'
import './app/app.css'
import './components/ui/compact/compact-ui.css'
import RendererRoot from './app/RendererRoot.vue'
import { compactUi } from './components/ui/compact/index.js'

document.documentElement.setAttribute('data-bs-theme', 'dark')

const app = createApp(RendererRoot)
app.use(compactUi)
app.mount('#app')
