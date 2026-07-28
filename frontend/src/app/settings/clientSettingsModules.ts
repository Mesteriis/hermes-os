import type { ClientModuleBootstrapV1 } from '../../gen/hermes/gateway/v1/client_bootstrap_pb'

export type ProviderSettingsOwnerId = 'mail' | 'telegram' | 'whatsapp' | 'zulip'
export type SettingsOwnerId = 'system' | 'recovery' | ProviderSettingsOwnerId

export const providerModuleIds = {
	mail: 'hermes-mail-runtime',
	telegram: 'hermes-telegram-runtime',
	whatsapp: 'hermes-whatsapp-runtime',
	zulip: 'hermes-zulip-runtime',
} as const

export function clientSettingsModule(
	modules: readonly ClientModuleBootstrapV1[],
	owner: ProviderSettingsOwnerId,
): ClientModuleBootstrapV1 | null {
	const moduleId = providerModuleIds[owner]
	return modules.find((module) => module.moduleId === moduleId) ?? null
}
