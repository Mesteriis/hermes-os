import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import App from './app/App.vue'
import router from './app/router'
import { initializeApiClient } from './platform/bootstrap/api'
import { initializeRealtime } from './platform/bootstrap/realtime'
import { loadFrontendConfig } from './platform/config/env'
import { useRealtimeStatusStore } from './shared/stores/realtimeStatus'
import './style.css'
import './styles/surfaces.css'
import './styles/theme-classes.css'

const app = createApp(App)
const pinia = createPinia()
const queryClient = new QueryClient()
let realtimeClient: ReturnType<typeof initializeRealtime> | null = null

app.use(pinia)
app.use(VueQueryPlugin, { queryClient })
app.use(router)

try {
	const config = loadFrontendConfig()
	const realtimeStatus = useRealtimeStatusStore(pinia)
	initializeApiClient(config)
	realtimeClient = initializeRealtime(config, queryClient, {
		onStatus: realtimeStatus.setRealtimeStatus
	})
} catch (error) {
	document.body.innerHTML = `<main class="startup-error"><h1>Hermes Hub cannot start</h1><p>${escapeHtml(error instanceof Error ? error.message : 'Unknown startup error')}</p></main>`
	throw error
}

app.mount('#app')

window.addEventListener('beforeunload', () => {
	realtimeClient?.disconnect()
})

function escapeHtml(value: string): string {
	return value
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&#39;')
}
