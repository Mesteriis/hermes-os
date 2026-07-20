import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import App from './app/App.vue'
import './style.css'
import './styles/surfaces.css'
import './styles/settings-background-jobs.css'
import './styles/settings-maintenance.css'
import './styles/settings-signal-hub.css'
import './styles/settings-trace-logs.css'
import './styles/theme-classes.css'
import './shared/ui/styles/index.css'
import './shared/ui/shell/app-layout.css'

const app = createApp(App)
const pinia = createPinia()
const queryClient = new QueryClient()

app.use(pinia)
app.use(VueQueryPlugin, { queryClient })

app.mount('#app')
