<script setup lang="ts">
import { watch } from 'vue'
import Toast from '../../shared/ui/Toast.vue'
import SystemControlPage from '../../platform/system-control/SystemControlPage.vue'
import { useClientNavigationSurface } from '../queries/useClientNavigationSurface'
import AppLayout from '../../shared/ui/shell/AppLayout.vue'
import AppNavbar from '../../shared/ui/shell/AppNavbar.vue'
import { BrowserGatewayAccessModeV1 } from '../../gen/hermes/gateway/v1/browser_session_pb'

const props = defineProps<{ gatewayAccessMode: BrowserGatewayAccessModeV1 }>()

const navbar = useClientNavigationSurface()
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
const bootstrap = navbar.bootstrap
const routeDowngradeReason = navbar.routeDowngradeReason

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
						:current-language="navbar.currentLanguage.value"
						:current-theme-family="currentThemeFamily"
						:current-theme-mode="currentThemeMode"
						:language-options="navbar.languageOptions"
						:navigation-levels="navigationLevels"
						:notifications="notifications"
						:notifications-count="notificationsCount"
						:theme-family-options="navbar.themeFamilyOptions"
						:theme-mode-options="navbar.themeModeOptions"
						@navigation-select="navbar.selectNavigationItem"
						@language-change="navbar.selectLanguage"
						@notification-dismiss="navbar.dismissNotification"
						@notification-select="navbar.selectNotification"
						@notifications-clear="navbar.clearNotifications"
						@theme-family-change="navbar.selectThemeFamily"
						@theme-mode-change="navbar.selectThemeMode"
					/>
				</template>

				<SystemControlPage
					v-if="selectedTopLevelRouteId === 'settings'"
					:bootstrap="bootstrap"
					:route-downgrade-reason="routeDowngradeReason"
					:developer-mode="props.gatewayAccessMode === BrowserGatewayAccessModeV1.LAN_DEVELOPMENT"
					:current-language="navbar.currentLanguage.value"
					:language-options="navbar.languageOptions"
					@language-change="navbar.selectLanguage"
				/>
			</AppLayout>
		</Toast>
	</section>
</template>
