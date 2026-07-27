<script setup lang="ts">
import { computed, watch } from 'vue'
import Toast from '../../shared/ui/Toast.vue'
import AppSettingsPage from '../settings/AppSettingsPage.vue'
import { useClientNavigationSurface } from '../queries/useClientNavigationSurface'
import AppLayout from '../../shared/ui/shell/AppLayout.vue'
import AppNavbar from '../../shared/ui/shell/AppNavbar.vue'
import { BrowserGatewayAccessModeV1 } from '../../gen/hermes/gateway/v1/browser_session_pb'
import { compiledClientSurfaceAdapterIds } from '../client-surfaces/compiledClientSurfaceAdapters'
import CanonicalCommunicationsRoute from '../../domains/communications/views/CanonicalCommunicationsRoute.vue'
import TelegramOperationalRoute from '../../integrations/telegram/views/TelegramOperationalRoute.vue'
import { hasClientModuleCapability } from '../client-surfaces/clientModuleCapabilities'
import WhatsAppOperationalRoute from '../../integrations/whatsapp/views/WhatsAppOperationalRoute.vue'
import MailOperationalRoute from '../../integrations/mail/views/MailOperationalRoute.vue'
import ZulipOperationalRoute from '../../integrations/zulip/views/ZulipOperationalRoute.vue'

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
const telegramCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.command.v1'),
)
const telegramAuthorizationAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.authorization.v1'),
)
const telegramLifecycleAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.lifecycle.v1'),
)
const telegramReconfigurationAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.reconfiguration.v1'),
)
const telegramQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'telegram.query.v1'),
)
const whatsAppCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'whatsapp.command.v1'),
)
const mailDeliveryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.delivery.v1'),
)
const mailSyncAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.sync.v1'),
)
const mailOperationalQueryAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.operational.query.v1'),
)
const mailSyncHealthAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'mail.sync.health.query.v1'),
)
const zulipCommandAvailable = computed(() =>
	hasClientModuleCapability(bootstrap.value, 'zulip.command.v1'),
)

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

				<CanonicalCommunicationsRoute
					v-if="selectedRouteId === 'communications-all'"
				/>
				<MailOperationalRoute
					v-else-if="selectedRouteId === 'communications-mail'"
					:can-deliver="mailDeliveryAvailable"
					:can-query="mailOperationalQueryAvailable"
					:can-sync="mailSyncAvailable"
					:can-sync-health="mailSyncHealthAvailable"
					:modules="bootstrap.modules"
				/>
				<TelegramOperationalRoute
					v-else-if="selectedRouteId === 'communications-telegram'"
					:can-authorize="telegramAuthorizationAvailable"
					:can-manage-lifecycle="telegramLifecycleAvailable"
					:can-query="telegramQueryAvailable"
					:can-reconfigure="telegramReconfigurationAvailable"
					:can-send="telegramCommandAvailable"
				/>
				<WhatsAppOperationalRoute
					v-else-if="selectedRouteId === 'communications-whatsapp'"
					:can-send="whatsAppCommandAvailable"
				/>
				<ZulipOperationalRoute
					v-else-if="selectedRouteId === 'communications-zulip'"
					:can-command="zulipCommandAvailable"
				/>
				<AppSettingsPage
					v-else-if="selectedTopLevelRouteId === 'settings'"
					:bootstrap="bootstrap"
					:route-downgrade-reason="routeDowngradeReason"
					:development-profile="props.gatewayAccessMode === BrowserGatewayAccessModeV1.LAN_DEVELOPMENT
						? 'private-lan'
						: props.gatewayAccessMode === BrowserGatewayAccessModeV1.LOCAL_DEVELOPMENT
							? 'loopback-full-stack'
							: 'disabled'"
					:current-language="navbar.currentLanguage.value"
					:language-options="navbar.languageOptions"
					:compiled-adapter-ids="compiledClientSurfaceAdapterIds"
					@language-change="navbar.selectLanguage"
				/>
			</AppLayout>
		</Toast>
	</section>
</template>
