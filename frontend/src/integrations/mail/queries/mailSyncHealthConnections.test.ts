import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import {
	mailSyncHealthConnectionFingerprint,
	mailSyncHealthConnections,
} from './mailSyncHealthConnections'

describe('Mail sync health connection discovery', () => {
	it('requires the exact health capability and returns stable effective connections', () => {
		const modules = [
			mailModule('registration-b', ' secondary '),
			mailModule('registration-a', 'primary'),
			mailModule('duplicate', 'primary'),
			mailModule('wrong-capability', 'operational-only', true, ['mail.operational.query.v1']),
			create(ClientModuleBootstrapV1Schema, {
				registrationId: 'telegram',
				moduleId: 'hermes-telegram-runtime',
				sectionsEnabled: true,
				capabilityIds: ['mail.sync.health.query.v1'],
				settings: mailSettings('not-mail'),
			}),
			mailModule('disabled', 'disabled', false),
		]

		expect(mailSyncHealthConnections(modules)).toEqual([
			{ connectionId: 'primary', registrationId: 'registration-a' },
			{ connectionId: 'secondary', registrationId: 'registration-b' },
		])
		expect(mailSyncHealthConnectionFingerprint(modules)).toBe(
			'registration-a:primary|registration-b:secondary',
		)
	})

	it('ignores wrong-type and unsafe public setting values', () => {
		const wrongType = mailModule('wrong-type', 'ignored')
		wrongType.settings!.values[0]!.value = create(ClientSettingValueV1Schema, {
			value: { case: 'booleanValue', value: true },
		})

		expect(mailSyncHealthConnections([
			wrongType,
			mailModule('control', 'bad\nconnection'),
			create(ClientModuleBootstrapV1Schema, {
				registrationId: 'missing',
				moduleId: 'hermes-mail-runtime',
				sectionsEnabled: true,
				capabilityIds: ['mail.sync.health.query.v1'],
			}),
		])).toEqual([])
	})
})

function mailModule(
	registrationId: string,
	connectionId: string,
	sectionsEnabled = true,
	capabilityIds = ['mail.sync.health.query.v1'],
) {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId,
		moduleId: 'hermes-mail-runtime',
		sectionsEnabled,
		capabilityIds,
		settings: mailSettings(connectionId),
	})
}

function mailSettings(connectionId: string) {
	return create(ClientModuleSettingsBootstrapV1Schema, {
		values: [create(ClientSettingValueEntryV1Schema, {
			settingId: 'mail.connection_id',
			value: create(ClientSettingValueV1Schema, {
				value: { case: 'stringValue', value: connectionId },
			}),
		})],
	})
}
