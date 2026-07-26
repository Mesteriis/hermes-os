import { create } from '@bufbuild/protobuf'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import type { Component } from 'vue'

import AppSettingsPage from '../../src/app/settings/AppSettingsPage.vue'
import { compiledClientSurfaceAdapterIds } from '../../src/app/client-surfaces/compiledClientSurfaceAdapters'
import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
	ClientSettingsApplyStateV1,
} from '../../src/gen/hermes/gateway/v1/client_bootstrap_pb'
import {
	recoveryClientBootstrap,
	type ClientBootstrapSnapshot,
} from '../../src/platform/gateway/clientBootstrap'

const meta = {
	title: 'Hermes App/Settings/Clean Room',
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => createSettingsStory('system'),
}

export const Mail: Story = {
	render: () => createSettingsStory('mail'),
}

function createSettingsStory(initialOwner: 'mail' | 'system'): Component {
	return {
		components: { AppSettingsPage },
		setup() {
			return {
				bootstrap: settingsBootstrap(),
				compiledAdapterIds: compiledClientSurfaceAdapterIds,
				initialOwner,
				languageOptions: [
					{ value: 'en', label: 'English' },
					{ value: 'ru', label: 'Русский' },
				],
			}
		},
		template: `
			<AppSettingsPage
				:bootstrap="bootstrap"
				:compiled-adapter-ids="compiledAdapterIds"
				current-language="en"
				:developer-mode="true"
				:initial-owner="initialOwner"
				:language-options="languageOptions"
			/>
		`,
	}
}

function settingsBootstrap(): ClientBootstrapSnapshot {
	const recovery = recoveryClientBootstrap()
	const mail = create(ClientModuleBootstrapV1Schema, {
		registrationId: 'mail.owner.local',
		moduleId: 'hermes-mail-runtime',
		grantEpoch: 4n,
		capabilityIds: ['mail.delivery.v1', 'mail.sync.v1'],
		sectionsEnabled: true,
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			schemaMajor: 1,
			schemaRevision: 3,
			desiredRevision: 8n,
			effectiveRevision: 8n,
			applyState: ClientSettingsApplyStateV1.CURRENT,
			values: [
				setting('sync_interval', 'Sync interval', {
					case: 'durationMillis',
					value: 300000n,
				}),
				setting('content_egress', 'Content egress', {
					case: 'booleanValue',
					value: false,
				}),
			],
		}),
	})
	const providerModules = [
		mail,
		...[
			['telegram', 'hermes-telegram-runtime'],
			['whatsapp', 'hermes-whatsapp-runtime'],
			['zulip', 'hermes-zulip-runtime'],
		].map(([provider, moduleId]) => create(ClientModuleBootstrapV1Schema, {
			registrationId: `${provider}.owner.local`,
			moduleId,
			grantEpoch: 2n,
			sectionsEnabled: true,
		})),
	]
	return Object.assign(new Map(recovery), {
		modules: providerModules,
		systemStatus: recovery.systemStatus,
	}) as ClientBootstrapSnapshot
}

function setting(
	settingId: string,
	displayName: string,
	value: { case: 'durationMillis'; value: bigint } | { case: 'booleanValue'; value: boolean },
) {
	return create(ClientSettingValueEntryV1Schema, {
		settingId,
		displayName,
		editable: true,
		value: create(ClientSettingValueV1Schema, { value }),
	})
}
