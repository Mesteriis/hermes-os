import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import {
	mailOperationalConnectionFingerprint,
	mailOperationalConnections,
} from './mailOperationalConnections'

describe('Mail operational connection discovery', () => {
	it('reads only effective public Mail settings and returns stable deduplicated connections', () => {
		const modules = [
			mailModule('registration-b', ' secondary '),
			mailModule('registration-a', 'primary'),
			mailModule('registration-duplicate', 'primary'),
			create(ClientModuleBootstrapV1Schema, {
				registrationId: 'telegram',
				moduleId: 'hermes-telegram-runtime',
				sectionsEnabled: true,
				settings: mailSettings('not-mail'),
			}),
			mailModule('disabled', 'disabled', false),
		]

		expect(mailOperationalConnections(modules)).toEqual([
			{ connectionId: 'primary', registrationId: 'registration-a' },
			{ connectionId: 'secondary', registrationId: 'registration-b' },
		])
		expect(mailOperationalConnectionFingerprint(modules)).toBe(
			'registration-a:primary|registration-b:secondary',
		)
	})

	it('ignores absent, wrong-type, and control-character settings', () => {
		const wrongType = mailModule('wrong-type', 'ignored')
		wrongType.settings!.values[0]!.value = create(ClientSettingValueV1Schema, {
			value: { case: 'booleanValue', value: true },
		})

		expect(mailOperationalConnections([
			create(ClientModuleBootstrapV1Schema, {
				registrationId: 'missing',
				moduleId: 'hermes-mail-runtime',
				sectionsEnabled: true,
			}),
			wrongType,
			mailModule('control', 'bad\nconnection'),
		])).toEqual([])
	})
})

function mailModule(registrationId: string, connectionId: string, sectionsEnabled = true) {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId,
		moduleId: 'hermes-mail-runtime',
		sectionsEnabled,
		capabilityIds: ['mail.operational.query.v1'],
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
