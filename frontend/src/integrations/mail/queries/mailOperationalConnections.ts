import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'

const MAIL_MODULE_ID = 'hermes-mail-runtime'
const MAIL_CONNECTION_ID_SETTING = 'mail.connection_id'
const MAIL_OPERATIONAL_QUERY_CAPABILITY = 'mail.operational.query.v1'
const MAX_CONNECTION_ID_BYTES = 512
const textEncoder = new TextEncoder()

export type MailOperationalConnection = {
	connectionId: string
	registrationId: string
}

export function mailOperationalConnections(
	modules: readonly ClientModuleBootstrapV1[],
): readonly MailOperationalConnection[] {
	const connections = new Map<string, MailOperationalConnection>()
	for (const module of modules) {
		if (
			module.moduleId !== MAIL_MODULE_ID
			|| !module.sectionsEnabled
			|| !module.capabilityIds.includes(MAIL_OPERATIONAL_QUERY_CAPABILITY)
		) continue
		const setting = module.settings?.values.find(
			(entry) => entry.settingId === MAIL_CONNECTION_ID_SETTING,
		)
		if (setting?.value?.value.case !== 'stringValue') continue
		const connectionId = setting.value.value.value.trim()
		if (!validConnectionId(connectionId) || connections.has(connectionId)) continue
		connections.set(connectionId, {
			connectionId,
			registrationId: module.registrationId,
		})
	}
	return [...connections.values()].sort(
		(left, right) => left.connectionId.localeCompare(right.connectionId),
	)
}

export function mailOperationalConnectionFingerprint(
	modules: readonly ClientModuleBootstrapV1[],
): string {
	return mailOperationalConnections(modules)
		.map((connection) => `${connection.registrationId}:${connection.connectionId}`)
		.join('|')
}

function validConnectionId(value: string): boolean {
	if (!value || textEncoder.encode(value).length > MAX_CONNECTION_ID_BYTES) return false
	for (let index = 0; index < value.length; index += 1) {
		const code = value.charCodeAt(index)
		if (code <= 0x1f || code === 0x7f) return false
	}
	return true
}
