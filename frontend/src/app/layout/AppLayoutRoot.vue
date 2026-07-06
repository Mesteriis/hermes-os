<script setup lang="ts">
import { watch } from 'vue'
import Toast from '../../shared/ui/Toast.vue'
import CommunicationsWorkspaceView from '../../domains/communications/views/CommunicationsWorkspaceView.vue'
import SettingsPage from '../../domains/settings/views/SettingsPage.vue'
import { useAppLayoutNavbarSurface } from '../queries/useAppLayoutNavbarSurface'
import AppLayout from './AppLayout.vue'
import AppNavbar from './AppNavbar.vue'

const navbar = useAppLayoutNavbarSurface()
const breadcrumbs = navbar.breadcrumbs
const currentTheme = navbar.currentTheme
const currentThemeFamily = navbar.currentThemeFamily
const currentThemeMode = navbar.currentThemeMode
const healthChecks = navbar.healthChecks
const navigationLevels = navbar.navigationLevels
const notifications = navbar.notifications
const notificationsCount = navbar.notificationsCount
const notificationToasts = navbar.notificationToasts
const selectedRouteId = navbar.selectedRouteId
const selectedTopLevelRouteId = navbar.selectedTopLevelRouteId

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
						:health-checks="healthChecks"
						:health-status-label-visible-ms="navbar.healthStatusLabelVisibleMs"
						:current-language="navbar.currentLanguage"
						:current-theme-family="currentThemeFamily"
						:current-theme-mode="currentThemeMode"
						:language-options="navbar.languageOptions"
						:navigation-levels="navigationLevels"
						:notifications="notifications"
						:notifications-count="notificationsCount"
						:theme-family-options="navbar.themeFamilyOptions"
						:theme-mode-options="navbar.themeModeOptions"
						@navigation-select="navbar.selectNavigationItem"
						@notification-dismiss="navbar.dismissNotification"
						@notification-select="navbar.selectNotification"
						@notifications-clear="navbar.clearNotifications"
						@theme-family-change="navbar.selectThemeFamily"
						@theme-mode-change="navbar.selectThemeMode"
					/>
				</template>

				<CommunicationsWorkspaceView
					v-if="selectedTopLevelRouteId === 'communications'"
					:selected-route-id="selectedRouteId"
				/>
				<SettingsPage v-else-if="selectedTopLevelRouteId === 'settings'" />
			</AppLayout>
		</Toast>
	</section>
</template>
