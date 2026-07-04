<script setup lang="ts">
import { watch } from 'vue'
import Toast from '../../shared/ui/Toast.vue'
import { useAppLayoutNavbarSurface } from '../queries/useAppLayoutNavbarSurface'
import AppLayout from './AppLayout.vue'
import AppNavbar from './AppNavbar.vue'

const navbar = useAppLayoutNavbarSurface()
const breadcrumbs = navbar.breadcrumbs
const currentTheme = navbar.currentTheme
const currentThemeFamily = navbar.currentThemeFamily
const currentThemeMode = navbar.currentThemeMode
const navigationLevels = navbar.navigationLevels
const notificationToasts = navbar.notificationToasts

watch([currentTheme, currentThemeFamily, currentThemeMode], ([theme, family, mode]) => {
	document.documentElement.setAttribute('data-ui-theme', theme)
	document.documentElement.setAttribute('data-ui-theme-family', family)
	document.documentElement.setAttribute('data-ui-theme-mode', mode)
}, { immediate: true })
</script>

<template>
	<section
		class="app-layout-root"
		:data-ui-theme="currentTheme"
		:data-ui-theme-family="currentThemeFamily"
		:data-ui-theme-mode="currentThemeMode"
	>
		<Toast
			class="app-layout-notification-toasts"
			close-label="Закрыть уведомление"
			:default-toasts="notificationToasts"
			:duration="navbar.notificationToastVisibleMs"
		>
			<AppLayout>
				<template #topbar>
					<AppNavbar
						:breadcrumbs="breadcrumbs"
						:health-checks="navbar.healthChecks"
						:health-status-label-visible-ms="navbar.healthStatusLabelVisibleMs"
						:current-language="navbar.currentLanguage"
						:current-theme-family="currentThemeFamily"
						:current-theme-mode="currentThemeMode"
						:language-options="navbar.languageOptions"
						:navigation-levels="navigationLevels"
						:notifications="navbar.notifications"
						:notifications-count="navbar.notificationsCount"
						:theme-family-options="navbar.themeFamilyOptions"
						:theme-mode-options="navbar.themeModeOptions"
						@navigation-select="navbar.selectNavigationItem"
						@theme-family-change="navbar.selectThemeFamily"
						@theme-mode-change="navbar.selectThemeMode"
					/>
				</template>
			</AppLayout>
		</Toast>
	</section>
</template>
