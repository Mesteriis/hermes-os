<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClientSurfaceAdapterId } from '../../platform/client-runtime/clientSurfaces'
import type { ClientBootstrapSnapshot } from '../../platform/gateway/clientBootstrap'
import SystemControlPage from '../../platform/system-control/SystemControlPage.vue'
import Icon from '../../shared/ui/Icon.vue'
import MailSettingsPanel from '../../integrations/mail/presentation/MailSettingsPanel.vue'
import TelegramSettingsPanel from '../../integrations/telegram/presentation/TelegramSettingsPanel.vue'
import TelegramAutomationSettingsRoute from '../../integrations/telegram/views/TelegramAutomationSettingsRoute.vue'
import WhatsAppSettingsPanel from '../../integrations/whatsapp/presentation/WhatsAppSettingsPanel.vue'
import ZulipSettingsPanel from '../../integrations/zulip/presentation/ZulipSettingsPanel.vue'
import {
	clientSettingsModule,
	type SettingsOwnerId,
} from './clientSettingsModules'
import { hasClientModuleCapability } from '../client-surfaces/clientModuleCapabilities'
import './appSettingsPage.css'

const props = defineProps<{
	bootstrap: ClientBootstrapSnapshot
	routeDowngradeReason?: string
	developerMode: boolean
	currentLanguage: string
	languageOptions: readonly { value: string; label: string }[]
	compiledAdapterIds: readonly ClientSurfaceAdapterId[]
	initialOwner?: SettingsOwnerId
}>()
const emit = defineEmits<{ languageChange: [value: string] }>()
const selectedOwner = ref<SettingsOwnerId>(props.initialOwner ?? 'system')

const mailModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'mail'))
const telegramModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'telegram'))
const telegramAutomationCommandAvailable = computed(() =>
	hasClientModuleCapability(props.bootstrap, 'telegram.automation.command.v1'),
)
const telegramAutomationQueryAvailable = computed(() =>
	hasClientModuleCapability(props.bootstrap, 'telegram.automation.query.v1'),
)
const whatsAppModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'whatsapp'))
const zulipModule = computed(() => clientSettingsModule(props.bootstrap.modules, 'zulip'))

const providerNavigation = [
	{ id: 'mail', label: 'Mail', icon: 'tabler:mail' },
	{ id: 'telegram', label: 'Telegram', icon: 'tabler:brand-telegram' },
	{ id: 'whatsapp', label: 'WhatsApp', icon: 'tabler:brand-whatsapp' },
	{ id: 'zulip', label: 'Zulip', icon: 'tabler:brand-zulip' },
] as const
</script>

<template>
	<section class="app-settings-page">
		<div class="app-settings-workbench">
			<nav class="app-settings-navigation" aria-label="Settings owners">
				<header class="app-settings-navigation__header">
					<span>Settings</span>
					<strong>Owner workbench</strong>
				</header>
				<section class="app-settings-navigation__group">
					<h2>Platform</h2>
					<button
						type="button"
						:class="{ active: selectedOwner === 'system' }"
						@click="selectedOwner = 'system'"
					>
						<Icon class="tree-icon" icon="tabler:heart-rate-monitor" />
						<span class="app-settings-navigation__copy">
							<strong>System Control</strong>
							<small>Kernel recovery and admission</small>
						</span>
					</button>
				</section>
				<section class="app-settings-navigation__group">
					<h2>Integrations</h2>
					<button
						v-for="owner in providerNavigation"
						:key="owner.id"
						type="button"
						:class="{ active: selectedOwner === owner.id }"
						@click="selectedOwner = owner.id"
					>
						<Icon class="tree-icon" :icon="owner.icon" />
						<span class="app-settings-navigation__copy">
							<strong>{{ owner.label }}</strong>
							<small>Provider-owned settings</small>
						</span>
					</button>
				</section>
			</nav>

			<main class="app-settings-content">
				<SystemControlPage
					v-if="selectedOwner === 'system'"
					:bootstrap="bootstrap"
					:route-downgrade-reason="routeDowngradeReason"
					:developer-mode="developerMode"
					:current-language="currentLanguage"
					:language-options="languageOptions"
					:compiled-adapter-ids="compiledAdapterIds"
					@language-change="emit('languageChange', $event)"
				/>
				<MailSettingsPanel v-else-if="selectedOwner === 'mail'" :module="mailModule" />
				<div v-else-if="selectedOwner === 'telegram'">
					<TelegramSettingsPanel :module="telegramModule" />
					<TelegramAutomationSettingsRoute
						:can-command="telegramAutomationCommandAvailable"
						:can-query="telegramAutomationQueryAvailable"
					/>
				</div>
				<WhatsAppSettingsPanel v-else-if="selectedOwner === 'whatsapp'" :module="whatsAppModule" />
				<ZulipSettingsPanel v-else :module="zulipModule" />
			</main>
		</div>
	</section>
</template>
