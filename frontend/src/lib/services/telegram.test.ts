import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	fetchTelegramCapabilities: vi.fn(),
	fetchTelegramAccounts: vi.fn(),
	fetchTelegramChats: vi.fn(),
	fetchTelegramMessages: vi.fn(),
	fetchTelegramRuntimeStatus: vi.fn(),
	startTelegramRuntime: vi.fn(),
	syncTelegramChats: vi.fn(),
	syncTelegramHistory: vi.fn(),
	fetchAutomationTemplates: vi.fn(),
	fetchAutomationPolicies: vi.fn(),
	fetchTelegramCalls: vi.fn(),
	fetchCallTranscript: vi.fn(),
	fetchTelegramQrLoginStatus: vi.fn(),
	ingestTelegramFixtureMessage: vi.fn(),
	sendTelegramManualMessage: vi.fn(),
	downloadTelegramMedia: vi.fn(),
	saveAutomationPolicy: vi.fn(),
	saveAutomationTemplate: vi.fn(),
	dryRunTelegramSend: vi.fn(),
	saveTelegramCall: vi.fn(),
	saveCallTranscriptFixture: vi.fn(),
	setupTelegramAccount: vi.fn(),
	setupTelegramFixtureAccount: vi.fn(),
	startTelegramQrLogin: vi.fn(),
	submitTelegramQrLoginPassword: vi.fn()
}));

import {
	fetchAutomationPolicies,
	fetchAutomationTemplates,
	fetchCallTranscript,
	fetchTelegramCalls,
	fetchTelegramCapabilities,
	fetchTelegramAccounts,
	fetchTelegramChats,
	fetchTelegramMessages,
	fetchTelegramRuntimeStatus,
	downloadTelegramMedia,
	sendTelegramManualMessage,
	setupTelegramAccount,
	syncTelegramChats,
	syncTelegramHistory,
	startTelegramQrLogin
} from '$lib/api';
import {
	filterTelegramChatsByGroup,
	filterTelegramChats,
	loadTelegramWorkspace,
	saveTelegramAccountFromWizard,
	sendTelegramManualMessageFromUi,
	downloadTelegramMediaFromUi,
	shouldPollTelegramQrLoginStatus,
	syncTelegramChatsFromUi,
	syncTelegramOlderHistoryFromUi,
	syncTelegramSelectedHistoryFromUi,
	startTelegramQrLoginFromWizard,
	telegramAttachmentHintsForMessages,
	telegramChatFilterCounts,
	telegramChatGroupFilters,
	telegramLinkHintsForMessages,
	telegramOldestTdlibMessageId
} from './telegram';

const qrReadyCapabilities = {
	version: 'v4',
	runtime_mode: 'tdlib_qr',
	telegram_app_credentials_configured: true,
	tdjson_runtime_available: true,
	qr_login_ready: true,
	capabilities: [],
	unsupported_features: []
};

const accountForm = {
	account_id: 'telegram-primary',
	display_name: 'Primary Telegram',
	api_id: '12345',
	api_hash: 'hash-from-old-form',
	session_encryption_key: 'session-key-from-old-form',
	tdlib_data_path: 'docker/data/telegram/telegram-primary'
};

describe('Telegram service QR login', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('starts QR login through backend-configured app credentials without posting app secrets', async () => {
		vi.mocked(startTelegramQrLogin).mockResolvedValue({
			setup_id: 'setup-1',
			account_id: 'telegram-primary',
			status: 'waiting_qr_scan',
			qr_link: 'tg://login?token=fixture',
			qr_svg: '<svg />',
			telegram_user_id: null,
			telegram_username: null,
			suggested_account_id: null,
			suggested_display_name: null,
			suggested_external_account_id: null,
			expires_at: null,
			poll_after_ms: 2500,
			message: 'Scan this QR code'
		});

		const result = await startTelegramQrLoginFromWizard({
			accountForm,
			capabilities: qrReadyCapabilities,
			externalAccountId: 'qr-login:telegram-primary'
		});

		expect(result.error).toBe('');
		expect(result.qrLogin?.status).toBe('waiting_qr_scan');

		const request = vi.mocked(startTelegramQrLogin).mock.calls[0][0] as Record<string, unknown>;
		expect(request).toMatchObject({
			account_id: 'telegram-primary',
			display_name: 'Primary Telegram',
			external_account_id: 'qr-login:telegram-primary',
			session_encryption_key: 'session-key-from-old-form',
			tdlib_data_path: 'docker/data/telegram/telegram-primary',
			transcription_enabled: false
		});
		expect('api_id' in request).toBe(false);
		expect('api_hash' in request).toBe(false);
	});

	it('saves QR-ready accounts through the QR-authorized metadata path', async () => {
		vi.mocked(setupTelegramAccount).mockResolvedValue({
			account_id: 'telegram-user-second',
			provider_kind: 'telegram_user',
			runtime: 'tdlib_qr_authorized',
			transcription_enabled: false,
			credential_bindings: []
		});

		const result = await saveTelegramAccountFromWizard({
			accountForm: {
				account_id: 'telegram-user-second',
				provider_kind: 'telegram_user',
				display_name: '@second',
				external_account_id: 'telegram:100200300',
				api_id: '12345',
				api_hash: 'hash-from-old-form',
				bot_token: '',
				session_encryption_key: 'session-key-from-old-form',
				tdlib_data_path: 'docker/data/telegram/telegram-user-second',
				transcription_enabled: true
			},
			authMethod: 'qr',
			qrLogin: {
				setup_id: 'setup-2',
				account_id: 'telegram-user-second',
				status: 'ready',
				qr_link: null,
				qr_svg: null,
				telegram_user_id: '100200300',
				telegram_username: 'second',
				suggested_account_id: 'telegram-user-second',
				suggested_display_name: '@second',
				suggested_external_account_id: 'telegram:100200300',
				expires_at: null,
				poll_after_ms: 2500,
				message: null
			},
			isFixtureSetup: false
		});

		expect(result.error).toBe('');
		expect(result.accountId).toBe('telegram-user-second');

		const request = vi.mocked(setupTelegramAccount).mock.calls[0][0] as Record<string, unknown>;
		expect(request).toMatchObject({
			account_id: 'telegram-user-second',
			provider_kind: 'telegram_user',
			display_name: '@second',
			external_account_id: 'telegram:100200300',
			session_encryption_key: 'session-key-from-old-form',
			tdlib_data_path: 'docker/data/telegram/telegram-user-second',
			transcription_enabled: false,
			qr_authorized: true
		});
		expect('api_id' in request).toBe(false);
		expect('api_hash' in request).toBe(false);
	});

	it('does not start QR login when backend Telegram app credentials are missing', async () => {
		const result = await startTelegramQrLoginFromWizard({
			accountForm,
			capabilities: {
				...qrReadyCapabilities,
				telegram_app_credentials_configured: false,
				qr_login_ready: false
			},
			externalAccountId: 'qr-login:telegram-primary'
		});

		expect(result.qrLogin).toBeNull();
		expect(result.error).toBe('Telegram app credentials are not configured in the backend environment');
		expect(startTelegramQrLogin).not.toHaveBeenCalled();
	});

	it('keeps polling while Telegram is checking the 2-step verification password', () => {
		expect(shouldPollTelegramQrLoginStatus('waiting_qr_scan')).toBe(true);
		expect(shouldPollTelegramQrLoginStatus('waiting_password')).toBe(true);
		expect(shouldPollTelegramQrLoginStatus('ready')).toBe(false);
		expect(shouldPollTelegramQrLoginStatus('failed')).toBe(false);
	});
});

describe('Telegram service live workbench state', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads account-scoped runtime status for Telegram chat accounts', async () => {
		vi.mocked(fetchTelegramCapabilities).mockResolvedValue(qrReadyCapabilities);
		vi.mocked(fetchTelegramAccounts).mockResolvedValue({
			items: [
				{
					account_id: 'telegram-primary',
					provider_kind: 'telegram_user',
					display_name: 'Primary Telegram',
					external_account_id: 'telegram:primary',
					runtime: 'fixture',
					lifecycle_state: 'active',
					transcription_enabled: false,
					tdlib_data_path: 'docker/data/telegram/telegram-primary',
					created_at: '2026-06-06T12:00:00Z',
					updated_at: '2026-06-06T12:00:00Z'
				}
			]
		});
		vi.mocked(fetchTelegramChats).mockResolvedValue({
			items: [
				{
					telegram_chat_id: 'telegram-chat-1',
					account_id: 'telegram-primary',
					provider_chat_id: 'chat-1',
					chat_kind: 'private',
					title: 'Primary Chat',
					username: null,
					sync_state: 'synced',
					last_message_at: null,
					metadata: {},
					created_at: '2026-06-06T12:00:00Z',
					updated_at: '2026-06-06T12:00:00Z'
				}
			]
		});
		vi.mocked(fetchTelegramMessages).mockResolvedValue({ items: [] });
		vi.mocked(fetchAutomationTemplates).mockResolvedValue({ items: [] });
		vi.mocked(fetchAutomationPolicies).mockResolvedValue({ items: [] });
		vi.mocked(fetchTelegramCalls).mockResolvedValue({ items: [] });
		vi.mocked(fetchTelegramRuntimeStatus).mockResolvedValue({
			account_id: 'telegram-primary',
			provider_kind: 'telegram_user',
			runtime_kind: 'fixture',
			status: 'running',
			fixture_runtime: true,
			tdjson_runtime_available: false,
			telegram_app_credentials_configured: false,
			live_send_available: false,
			last_error: null,
			updated_at: '2026-06-06T12:00:00Z'
		});

		const result = await loadTelegramWorkspace('', '');

		expect(result.error).toBe('');
		expect(result.accounts).toHaveLength(1);
		expect(fetchTelegramAccounts).toHaveBeenCalledWith();
		expect(fetchTelegramChats).toHaveBeenCalledWith(undefined, 5000);
		expect(fetchTelegramRuntimeStatus).toHaveBeenCalledWith('telegram-primary');
		expect(fetchTelegramMessages).toHaveBeenNthCalledWith(1);
		expect(fetchTelegramMessages).toHaveBeenNthCalledWith(2, 'telegram-primary', 'chat-1', 5000);
		expect(result.runtimeStatuses['telegram-primary'].status).toBe('running');
	});

	it('maps unreachable backend fetch failures to a Telegram-specific workspace error', async () => {
		vi.mocked(fetchTelegramCapabilities).mockRejectedValue(new Error('Failed to fetch'));

		const result = await loadTelegramWorkspace('', '');

		expect(result.error).toBe('Telegram backend is unreachable at the configured API URL.');
		expect(result.chats).toEqual([]);
	});

	it('sends manual messages through the backend provider_write API', async () => {
		vi.mocked(sendTelegramManualMessage).mockResolvedValue({
			raw_record_id: 'raw:v4:telegram:123',
			message_id: 'message:v4:telegram:123',
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			delivery_state: 'sent',
			status: 'sent',
			runtime_kind: 'fixture',
			rendered_preview_hash: 'sha256:abc'
		});

		const result = await sendTelegramManualMessageFromUi({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			text: 'Hello from Hermes'
		});

		expect(result.error).toBe('');
		expect(result.nextText).toBe('');
		expect(result.providerChatId).toBe('chat-1');
		expect(result.message).toContain('sha256:abc');

		const request = vi.mocked(sendTelegramManualMessage).mock.calls[0][0] as Record<string, unknown>;
		expect(request).toMatchObject({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			text: 'Hello from Hermes'
		});
		expect(String(request.command_id)).toMatch(/^telegram-manual-send-/);
		expect('provider_message_id' in request).toBe(false);
		expect('sender_display_name' in request).toBe(false);
	});

	it('syncs chat metadata through the backend runtime API', async () => {
		vi.mocked(syncTelegramChats).mockResolvedValue({
			account_id: 'telegram-primary',
			runtime_kind: 'fixture',
			status: 'synced',
			synced_count: 2,
			items: []
		});

		const result = await syncTelegramChatsFromUi('telegram-primary');

		expect(result.error).toBe('');
		expect(result.message).toBe('Telegram chats synced: 2');
		expect(result.accountId).toBe('telegram-primary');
		expect(syncTelegramChats).toHaveBeenCalledWith({
			account_id: 'telegram-primary',
			limit: 100
		});
	});

	it('syncs selected chat history through the backend runtime API', async () => {
		vi.mocked(syncTelegramHistory).mockResolvedValue({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			runtime_kind: 'fixture',
			status: 'synced',
			synced_count: 3,
			has_more: false,
			next_from_message_id: null,
			items: []
		});

		const result = await syncTelegramSelectedHistoryFromUi({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			chat_kind: 'private'
		});

		expect(result.error).toBe('');
		expect(result.message).toBe('Telegram history synced: 3');
		expect(result.providerChatId).toBe('chat-1');
		expect(syncTelegramHistory).toHaveBeenCalledWith({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			mode: 'full',
			limit: 100
		});
	});

	it('syncs older selected history from the oldest visible TDLib message id', async () => {
		vi.mocked(syncTelegramHistory).mockResolvedValue({
			account_id: 'telegram-primary',
			provider_chat_id: '-100123',
			runtime_kind: 'tdlib_qr_authorized',
			status: 'synced',
			synced_count: 100,
			has_more: true,
			next_from_message_id: 12345,
			items: []
		});

		const result = await syncTelegramOlderHistoryFromUi({
			account_id: 'telegram-primary',
			provider_chat_id: '-100123',
			from_message_id: 67890
		});

		expect(result.error).toBe('');
		expect(result.hasMore).toBe(true);
		expect(result.nextFromMessageId).toBe(12345);
		expect(syncTelegramHistory).toHaveBeenCalledWith({
			account_id: 'telegram-primary',
			provider_chat_id: '-100123',
			from_message_id: 67890,
			mode: 'older',
			limit: 100
		});
	});

	it('requests on-demand media downloads through the backend runtime API', async () => {
		vi.mocked(downloadTelegramMedia).mockResolvedValue({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			provider_message_id: 'provider-message-2',
			runtime_kind: 'tdlib_qr_authorized',
			status: 'downloaded',
			tdlib_file_id: 42,
			local_path: 'docker/data/telegram/telegram-primary/file.zip',
			size_bytes: 2516582,
			expected_size_bytes: 2516582,
			downloaded_size_bytes: 2516582,
			is_downloading_active: false,
			is_downloading_completed: true,
			attachment_id: 'attachment-1',
			blob_id: 'blob-1',
			scan_status: 'not_scanned'
		});

		const result = await downloadTelegramMediaFromUi({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			provider_message_id: 'provider-message-2',
			tdlib_file_id: 42,
			provider_attachment_id: 'tdlib-file:42',
			filename: 'benchmarks_3.12.4.zip',
			content_type: 'application/zip',
			priority: 16
		});

		expect(result.error).toBe('');
		expect(result.response?.attachment_id).toBe('attachment-1');
		expect(result.message).toBe('Telegram media downloaded: attachment-1');
		expect(downloadTelegramMedia).toHaveBeenCalledWith({
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			provider_message_id: 'provider-message-2',
			tdlib_file_id: 42,
			provider_attachment_id: 'tdlib-file:42',
			filename: 'benchmarks_3.12.4.zip',
			content_type: 'application/zip',
			priority: 16
		});
	});
});

describe('Telegram workbench model helpers', () => {
	const chats = [
		{
			telegram_chat_id: 'chat-row-1',
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-1',
			chat_kind: 'group',
			title: 'Python Community',
			username: 'python_community',
			sync_state: 'synced',
			last_message_at: '2026-06-12T10:42:00Z',
			metadata: { unread_count: 3, pinned: true, category: 'projects' },
			created_at: '2026-06-12T09:00:00Z',
			updated_at: '2026-06-12T10:42:00Z'
		},
		{
			telegram_chat_id: 'chat-row-2',
			account_id: 'telegram-primary',
			provider_chat_id: 'chat-2',
			chat_kind: 'private',
			title: 'YouTrack Bot',
			username: 'youtrack_bot',
			sync_state: 'synced',
			last_message_at: '2026-06-12T08:50:00Z',
			metadata: { archived: true },
			created_at: '2026-06-12T08:00:00Z',
			updated_at: '2026-06-12T08:50:00Z'
		}
	] as const;

	const messages = [
		{
			message_id: 'message-1',
			raw_record_id: 'raw-1',
			account_id: 'telegram-primary',
			provider_message_id: 'provider-message-1',
			provider_chat_id: 'chat-1',
			chat_title: 'Python Community',
			sender: 'Alex',
			sender_display_name: 'Alex',
			text: 'Release notes https://www.python.org/downloads/',
			occurred_at: '2026-06-12T10:42:00Z',
			projected_at: '2026-06-12T10:42:01Z',
			channel_kind: 'telegram_user',
			delivery_state: 'received',
			metadata: {}
		},
		{
			message_id: 'message-2',
			raw_record_id: 'raw-2',
			account_id: 'telegram-primary',
			provider_message_id: 'provider-message-2',
			provider_chat_id: 'chat-1',
			chat_title: 'Python Community',
			sender: 'Maria',
			sender_display_name: 'Maria',
			text: 'Benchmarks attached',
			occurred_at: '2026-06-12T10:45:00Z',
			projected_at: '2026-06-12T10:45:01Z',
			channel_kind: 'telegram_user',
			delivery_state: 'received',
			metadata: {
				tdlib_raw: {
					'@type': 'message',
					content: {
						'@type': 'messageDocument',
						document: {
							file_name: 'benchmarks_3.12.4.zip',
							mime_type: 'application/zip',
							document: {
								id: 42,
								size: 2516582,
								local: {
									path: '',
									is_downloading_completed: false,
									is_downloading_active: false
								}
							}
						}
					}
				}
			}
		}
	] as const;

	it('filters chats by semantic Telegram workbench tabs and search text', () => {
		expect(filterTelegramChats([...chats], [...messages], 'python', 'all')).toHaveLength(1);
		expect(filterTelegramChats([...chats], [...messages], '', 'unread')).toHaveLength(1);
		expect(filterTelegramChats([...chats], [...messages], '', 'pinned')).toHaveLength(1);
		expect(filterTelegramChats([...chats], [...messages], '', 'projects')).toHaveLength(1);
		expect(filterTelegramChats([...chats], [...messages], '', 'bots')).toHaveLength(1);
		expect(filterTelegramChats([...chats], [...messages], '', 'archived')).toHaveLength(1);

		const counts = telegramChatFilterCounts([...chats], [...messages]);
		expect(counts.find((item) => item.filter === 'all')?.count).toBe(2);
		expect(counts.find((item) => item.filter === 'unread')?.count).toBe(1);
	});

	it('builds local and Telegram folder chat group filters', () => {
		const groupedChats = [
			...chats,
			{
				...chats[0],
				telegram_chat_id: 'chat-row-3',
				provider_chat_id: 'chat-3',
				chat_kind: 'channel',
				title: 'Release Channel',
				username: 'release_channel',
				metadata: { telegram_folder: 'Work' }
			},
			{
				...chats[1],
				telegram_chat_id: 'chat-row-4',
				provider_chat_id: 'chat-4',
				chat_kind: 'bot',
				title: 'Build Bot',
				username: 'build_bot',
				metadata: {
					tdlib_raw: {
						positions: [{ list: { '@type': 'chatListArchive' } }]
					}
				}
			}
		] as const;

		const groups = telegramChatGroupFilters([...groupedChats]);

		expect(groups.find((group) => group.id === 'local:private')?.count).toBe(1);
		expect(groups.find((group) => group.id === 'local:group')?.count).toBe(1);
		expect(groups.find((group) => group.id === 'local:channel')?.count).toBe(1);
		expect(groups.find((group) => group.id === 'local:bot')?.count).toBe(2);
		expect(groups.find((group) => group.id === 'telegram:Work')?.count).toBe(1);
		expect(groups.find((group) => group.id === 'telegram:Archived')?.count).toBe(1);
		expect(filterTelegramChatsByGroup([...groupedChats], 'telegram:Work')).toHaveLength(1);
	});

	it('extracts link and attachment hints from persisted Telegram metadata', () => {
		const links = telegramLinkHintsForMessages([...messages]);
		expect(links).toMatchObject([
			{
				url: 'https://www.python.org/downloads/',
				messageId: 'message-1'
			}
		]);

		const attachments = telegramAttachmentHintsForMessages([...messages]);
		expect(attachments).toMatchObject([
			{
				id: 'tdlib-file:42',
				kind: 'document',
				fileName: 'benchmarks_3.12.4.zip',
				mimeType: 'application/zip',
				sizeBytes: 2516582,
				tdlibFileId: 42,
				providerAttachmentId: 'tdlib-file:42',
				downloadState: 'remote',
				messageId: 'message-2'
			}
		]);
	});

	it('finds the oldest TDLib message id for paged history sync', () => {
		expect(telegramOldestTdlibMessageId([...messages])).toBeNull();
		expect(
			telegramOldestTdlibMessageId([
				{ ...messages[0], provider_message_id: '-100123:777' },
				{ ...messages[1], provider_message_id: '-100123:555' }
			])
		).toBe(555);
	});
});
