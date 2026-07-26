import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
	ClientSettingsApplyStateV1,
} from '../../gen/hermes/gateway/v1/client_bootstrap_pb'
import {
	publicModuleSettingRows,
	publicModuleSettingsReasonCode,
} from './publicModuleSettings'

describe('public module settings projection', () => {
	it('projects only typed sanitized bootstrap values', () => {
		const rows = publicModuleSettingRows([create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail.local',
			moduleId: 'hermes-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				applyState: ClientSettingsApplyStateV1.CURRENT,
				values: [create(ClientSettingValueEntryV1Schema, {
					settingId: 'sync_interval',
					displayName: 'Sync interval',
					editable: true,
					value: create(ClientSettingValueV1Schema, {
						value: { case: 'durationMillis', value: 15000n },
					}),
				})],
			}),
		})])

		expect(rows).toEqual([expect.objectContaining({
			moduleId: 'hermes-mail-runtime',
			label: 'Sync interval',
			value: '15000 ms',
			editable: true,
			applyState: 'Current',
			blocked: false,
		})])
	})

	it('does not project entries whose typed value is absent', () => {
		const rows = publicModuleSettingRows([create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail.local',
			moduleId: 'hermes-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				values: [create(ClientSettingValueEntryV1Schema, {
					settingId: 'missing',
				})],
			}),
		})])

		expect(rows).toEqual([])
	})

	it('distinguishes absent modules, absent schemas, and current schemas', () => {
		const withoutSchema = create(ClientModuleBootstrapV1Schema, {
			moduleId: 'hermes-mail-runtime',
		})
		const current = create(ClientModuleBootstrapV1Schema, {
			moduleId: 'hermes-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema),
		})
		const blocked = create(ClientModuleBootstrapV1Schema, {
			moduleId: 'hermes-mail-runtime',
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				sanitizedReasonCode: 'owner_action_required',
			}),
		})

		expect(publicModuleSettingsReasonCode(null)).toBe('module_not_registered')
		expect(publicModuleSettingsReasonCode(withoutSchema)).toBe('settings_schema_unavailable')
		expect(publicModuleSettingsReasonCode(current)).toBe('current')
		expect(publicModuleSettingsReasonCode(blocked)).toBe('owner_action_required')
	})
})
