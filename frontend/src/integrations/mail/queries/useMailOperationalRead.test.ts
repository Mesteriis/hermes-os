import { create } from '@bufbuild/protobuf'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
} from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import { MailFolderKindV1 } from '../../../gen/hermes/mail/operational/v1/client_pb'
import {
	getMailOperationalMessage,
	listMailOperationalFolders,
	listMailOperationalMessages,
	listMailOperationalThreads,
} from '../api/mailOperationalReadGateway'
import { useMailOperationalRead } from './useMailOperationalRead'

vi.mock('../api/mailOperationalReadGateway', () => ({
	getMailOperationalMessage: vi.fn(),
	listMailOperationalFolders: vi.fn(),
	listMailOperationalMessages: vi.fn(),
	listMailOperationalThreads: vi.fn(),
}))

describe('Mail operational read controller', () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.mocked(listMailOperationalFolders).mockResolvedValue({
			item: [{
				folderId: 'inbox',
				displayName: 'Inbox',
				kind: MailFolderKindV1.MAIL_FOLDER_KIND_INBOX,
				totalMessages: 1n,
				unreadMessages: 1n,
			}],
		} as never)
		vi.mocked(listMailOperationalThreads).mockResolvedValue({
			item: [{
				providerThreadId: 'thread-1',
				subject: 'Clean room',
				messageCount: 1n,
				unreadCount: 1n,
			}],
		} as never)
		vi.mocked(listMailOperationalMessages).mockResolvedValue({
			item: [{
				messageId: 'message-1',
				providerThreadId: 'thread-1',
				folderId: ['inbox'],
				subject: 'Clean room',
				flag: [],
				observationAnchorId: new Uint8Array(16),
			}],
		} as never)
		vi.mocked(getMailOperationalMessage).mockResolvedValue({
			summary: {
				messageId: 'message-1',
				providerThreadId: 'thread-1',
				folderId: ['inbox'],
				subject: 'Clean room',
				flag: [],
				recipient: [],
				observationAnchorId: new Uint8Array(16),
			},
		} as never)
	})

	it('loads folders, exact threads, messages, and detail from an admitted connection', async () => {
		const controller = useMailOperationalRead({
			canQuery: () => true,
			modules: () => [mailModule()],
		})

		await controller.reconcile()

		expect(listMailOperationalFolders).toHaveBeenCalledWith({
			connectionId: 'primary',
		})
		expect(listMailOperationalThreads).toHaveBeenCalledWith({
			connectionId: 'primary',
			folderId: 'inbox',
		})
		expect(listMailOperationalMessages).toHaveBeenCalledWith({
			connectionId: 'primary',
			folderId: 'inbox',
			providerThreadId: 'thread-1',
		})
		expect(getMailOperationalMessage).toHaveBeenCalledWith({
			connectionId: 'primary',
			messageId: 'message-1',
		})
		expect(controller.model.value).toMatchObject({
			status: 'ready',
			selectedConnectionId: 'primary',
		})
		expect(controller.model.value.detail?.subject).toBe('Clean room')
	})

	it('fails closed before transport when capability or effective connection is absent', async () => {
		const blocked = useMailOperationalRead({
			canQuery: () => false,
			modules: () => [mailModule()],
		})
		await blocked.reconcile()
		expect(blocked.model.value.status).toBe('blocked')

		const noConnection = useMailOperationalRead({
			canQuery: () => true,
			modules: () => [],
		})
		await noConnection.reconcile()
		expect(noConnection.model.value.status).toBe('empty')
		expect(listMailOperationalFolders).not.toHaveBeenCalled()
	})
})

function mailModule() {
	return create(ClientModuleBootstrapV1Schema, {
		registrationId: 'mail-primary',
		moduleId: 'hermes-mail-runtime',
		sectionsEnabled: true,
		capabilityIds: ['mail.operational.query.v1'],
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			values: [create(ClientSettingValueEntryV1Schema, {
				settingId: 'mail.connection_id',
				value: create(ClientSettingValueV1Schema, {
					value: { case: 'stringValue', value: 'primary' },
				}),
			})],
		}),
	})
}
