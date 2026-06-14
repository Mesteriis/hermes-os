import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './app/App.vue'
import router from './app/router'
import { initializeApiClient } from './platform/bootstrap/api'
import { loadFrontendConfig } from './platform/config/env'
import './style.css'
import './styles/surfaces.css'
import './styles/theme-classes.css'

const app = createApp(App)

try {
	initializeApiClient(loadFrontendConfig())
} catch (error) {
	document.body.innerHTML = `<main class="startup-error"><h1>Hermes Hub cannot start</h1><p>${escapeHtml(error instanceof Error ? error.message : 'Unknown startup error')}</p></main>`
	throw error
}

app.use(createPinia())
app.use(VueQueryPlugin)
app.use(router)

app.mount('#app')

function escapeHtml(value: string): string {
	return value
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&#39;')
}
