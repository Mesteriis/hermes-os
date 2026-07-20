<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AppLayoutRoot from './layout/AppLayoutRoot.vue'
import BrowserGatewayAccessGate from '../platform/gateway/BrowserGatewayAccessGate.vue'
import { BrowserGatewayAccessModeV1 } from '../gen/hermes/gateway/v1/browser_session_pb'
import { fetchBrowserGatewaySessionStatus } from '../platform/gateway/browserGatewaySession'

const authenticated = ref(false)
const checkingSession = ref(true)
const accessMode = ref<BrowserGatewayAccessModeV1.PAIRED | BrowserGatewayAccessModeV1.LAN_DEVELOPMENT>(BrowserGatewayAccessModeV1.PAIRED)
const viteDeveloperMode = import.meta.env.DEV

function redirectViteToDeveloperGateway(): boolean {
	if (!viteDeveloperMode || window.location.port !== '5173') return false
	const gateway = new URL(window.location.href)
	gateway.port = '9444'
	window.location.replace(gateway.toString())
	return true
}

async function enterAuthenticatedShell(): Promise<void> {
	const status = await fetchBrowserGatewaySessionStatus()
	accessMode.value = status.accessMode
	authenticated.value = true
}

onMounted(async () => {
	if (redirectViteToDeveloperGateway()) return
	try { await enterAuthenticatedShell() } catch { authenticated.value = false } finally { checkingSession.value = false }
})
</script>

<template>
	<AppLayoutRoot v-if="authenticated" :gateway-access-mode="accessMode" />
	<main v-else-if="checkingSession" class="browser-access-gate" data-ui-theme="base-light" aria-busy="true"><section class="browser-access-gate__card"><p>Checking Gateway session…</p></section></main>
	<BrowserGatewayAccessGate v-else @authenticated="enterAuthenticatedShell" />
</template>
