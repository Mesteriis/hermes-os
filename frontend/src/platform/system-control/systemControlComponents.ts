import {
	ClientSettingsApplyStateV1,
	ClientSystemComponentIdV1,
	ClientSystemComponentStateV1,
	type ClientModuleBootstrapV1,
	type ClientSettingValueV1,
	type ClientSystemComponentStatusV1,
} from '../../gen/hermes/gateway/v1/client_bootstrap_pb'

type SystemComponentDefinition = {
	id: ClientSystemComponentIdV1
	label: string
	icon: string
}

export type SystemControlComponentRow = SystemComponentDefinition & {
	state: ClientSystemComponentStateV1
	stateLabel: string
	reasonCode: string
	disabled: boolean
}

export type PublicModuleSettingRow = {
	key: string
	moduleId: string
	settingId: string
	label: string
	value: string
	editable: boolean
	applyState: string
	blocked: boolean
}

export const schedulerComponents: readonly SystemComponentDefinition[] = [
	{ id: ClientSystemComponentIdV1.SCHEDULER, label: 'Scheduler runtime', icon: 'tabler:calendar-time' },
	{ id: ClientSystemComponentIdV1.CLOCK, label: 'Clock', icon: 'tabler:clock' },
	{ id: ClientSystemComponentIdV1.STORAGE_CONTROL, label: 'Storage Control', icon: 'tabler:database-cog' },
	{ id: ClientSystemComponentIdV1.POSTGRESQL, label: 'PostgreSQL', icon: 'tabler:database' },
	{ id: ClientSystemComponentIdV1.EVENT_HUB, label: 'Event Hub', icon: 'tabler:route' },
	{ id: ClientSystemComponentIdV1.NATS, label: 'NATS', icon: 'tabler:arrows-exchange' },
]

export const eventComponents: readonly SystemComponentDefinition[] = [
	{ id: ClientSystemComponentIdV1.EVENT_HUB, label: 'Event Hub', icon: 'tabler:route' },
	{ id: ClientSystemComponentIdV1.NATS, label: 'NATS', icon: 'tabler:arrows-exchange' },
	{ id: ClientSystemComponentIdV1.SSE, label: 'Client SSE', icon: 'tabler:activity-heartbeat' },
]

export function systemControlComponentRows(
	definitions: readonly SystemComponentDefinition[],
	statuses: readonly ClientSystemComponentStatusV1[],
): readonly SystemControlComponentRow[] {
	const statusByComponent = new Map(statuses.map((status) => [status.componentId, status]))
	return definitions.map((definition) => {
		const status = statusByComponent.get(definition.id)
		const state = status?.state ?? ClientSystemComponentStateV1.UNAVAILABLE
		return {
			...definition,
			state,
			stateLabel: systemComponentStateLabel(state),
			reasonCode: status?.sanitizedReasonCode || 'status_unavailable',
			disabled: state === ClientSystemComponentStateV1.UNAVAILABLE
				|| state === ClientSystemComponentStateV1.NOT_ADMITTED,
		}
	})
}

export function publicModuleSettingRows(
	modules: readonly ClientModuleBootstrapV1[],
): readonly PublicModuleSettingRow[] {
	return modules.flatMap((module) => {
		const settings = module.settings
		if (!settings) return []
		const applyState = settingsApplyStateLabel(settings.applyState)
		const blocked = settings.applyState !== ClientSettingsApplyStateV1.CURRENT
		return settings.values.flatMap((entry) => entry.value
			? [{
				key: `${module.registrationId}:${entry.settingId}`,
				moduleId: module.moduleId,
				settingId: entry.settingId,
				label: entry.displayName || entry.settingId,
				value: settingValueLabel(entry.value),
				editable: entry.editable,
				applyState,
				blocked,
			}]
			: [])
	})
}

function systemComponentStateLabel(state: ClientSystemComponentStateV1): string {
	if (state === ClientSystemComponentStateV1.HEALTHY) return 'Healthy'
	if (state === ClientSystemComponentStateV1.DEGRADED) return 'Degraded'
	if (state === ClientSystemComponentStateV1.NOT_ADMITTED) return 'Not admitted'
	return 'Unavailable'
}

function settingsApplyStateLabel(state: ClientSettingsApplyStateV1): string {
	if (state === ClientSettingsApplyStateV1.CURRENT) return 'Current'
	if (state === ClientSettingsApplyStateV1.PENDING_VALIDATION) return 'Pending validation'
	if (state === ClientSettingsApplyStateV1.PENDING_APPLY) return 'Pending apply'
	if (state === ClientSettingsApplyStateV1.APPLYING) return 'Applying'
	if (state === ClientSettingsApplyStateV1.AWAITING_EXTERNAL_RESTART) return 'Awaiting restart'
	return 'Blocked configuration'
}

function settingValueLabel(value: ClientSettingValueV1): string {
	if (value.value.case === 'booleanValue') return value.value.value ? 'Enabled' : 'Disabled'
	if (value.value.case === 'durationMillis') return `${value.value.value} ms`
	if (value.value.case === 'timestampUnixMillis') return new Date(Number(value.value.value)).toLocaleString()
	return String(value.value.value)
}
