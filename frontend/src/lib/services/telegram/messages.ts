import type { TelegramChat, TelegramMessage } from '$lib/api';
import { formatDateTime } from '../formatting';
import type {
	TelegramAttachmentHint,
	TelegramChatFilter,
	TelegramChatFilterCount,
	TelegramChatGroupFilter,
	TelegramLinkHint
} from './types';

export function telegramMessageTime(message: TelegramMessage) {
	return formatDateTime(message.occurred_at ?? message.projected_at);
}

export function telegramMessagesForChat(
	messages: TelegramMessage[],
	providerChatId: string | null | undefined
): TelegramMessage[] {
	if (!providerChatId) return [];
	return messages.filter((message) => message.provider_chat_id === providerChatId);
}

export function telegramMessagesChronological(messages: TelegramMessage[]): TelegramMessage[] {
	return messages.slice().sort((left, right) => {
		const leftTime = new Date(left.occurred_at ?? left.projected_at).getTime();
		const rightTime = new Date(right.occurred_at ?? right.projected_at).getTime();
		return leftTime - rightTime;
	});
}

export function telegramOldestTdlibMessageId(messages: TelegramMessage[]): number | null {
	const ids = messages
		.map((message) => telegramTdlibMessageId(message.provider_message_id))
		.filter((value): value is number => value !== null);
	return ids.length ? Math.min(...ids) : null;
}

function telegramTdlibMessageId(providerMessageId: string): number | null {
	const id = providerMessageId.split(':').at(-1)?.trim() ?? '';
	if (!id) return null;
	const parsed = Number.parseInt(id, 10);
	return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

export function telegramChatPreview(chat: TelegramChat, messages: TelegramMessage[]): string {
	const latestMessage = telegramMessagesForChat(messages, chat.provider_chat_id)
		.slice()
		.sort((left, right) => {
			const leftTime = new Date(left.occurred_at ?? left.projected_at).getTime();
			const rightTime = new Date(right.occurred_at ?? right.projected_at).getTime();
			return rightTime - leftTime;
		})[0];
	if (!latestMessage) {
		return `${chat.account_id} · ${chat.sync_state}`;
	}
	const sender = latestMessage.sender_display_name ?? latestMessage.sender;
	const text = latestMessage.text.trim();
	return text ? `${sender}: ${text}` : `${sender}: ${latestMessage.delivery_state}`;
}

export function telegramChatUnreadCount(chat: TelegramChat): number {
	return metadataNumber(chat.metadata, [
		'unread_count',
		'unread_message_count',
		'unread_messages',
		'tdlib_raw.unread_count'
	]);
}

export function telegramChatMentionCount(chat: TelegramChat, messages: TelegramMessage[]): number {
	const metadataCount = metadataNumber(chat.metadata, [
		'mention_count',
		'mentions_count',
		'unread_mention_count',
		'tdlib_raw.unread_mention_count'
	]);
	if (metadataCount > 0) return metadataCount;
	return telegramMessagesForChat(messages, chat.provider_chat_id).filter((message) =>
		message.text.includes('@')
	).length;
}

export function telegramChatIsPinned(chat: TelegramChat): boolean {
	return metadataBoolean(chat.metadata, ['pinned', 'is_pinned', 'tdlib_raw.positions.0.is_pinned']);
}

export function telegramChatIsArchived(chat: TelegramChat): boolean {
	return metadataBoolean(chat.metadata, ['archived', 'is_archived', 'tdlib_raw.is_archived']);
}

export function telegramChatIsProject(chat: TelegramChat): boolean {
	return metadataBoolean(chat.metadata, ['project', 'is_project', 'hermes_project']) ||
		metadataString(chat.metadata, ['category', 'folder', 'label']).toLowerCase().includes('project');
}

export function telegramChatIsBot(chat: TelegramChat): boolean {
	return chat.chat_kind === 'bot' ||
		Boolean(chat.username?.toLowerCase().endsWith('bot')) ||
		/\bbot\b/i.test(chat.title);
}

export function telegramChatFilterCounts(
	chats: TelegramChat[],
	messages: TelegramMessage[]
): TelegramChatFilterCount[] {
	return [
		{ filter: 'all', count: chats.length },
		{ filter: 'unread', count: chats.filter((chat) => telegramChatUnreadCount(chat) > 0).length },
		{
			filter: 'mentions',
			count: chats.filter((chat) => telegramChatMentionCount(chat, messages) > 0).length
		},
		{ filter: 'pinned', count: chats.filter(telegramChatIsPinned).length },
		{ filter: 'projects', count: chats.filter(telegramChatIsProject).length },
		{ filter: 'bots', count: chats.filter(telegramChatIsBot).length },
		{ filter: 'archived', count: chats.filter(telegramChatIsArchived).length }
	];
}

export function telegramChatGroupFilters(chats: TelegramChat[]): TelegramChatGroupFilter[] {
	const localGroups: TelegramChatGroupFilter[] = [
		{
			id: 'local:all',
			label: 'All Chats',
			source: 'local',
			count: chats.length,
			icon: 'tabler:messages'
		},
		{
			id: 'local:private',
			label: 'Direct',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'private').length,
			icon: 'tabler:user'
		},
		{
			id: 'local:group',
			label: 'Groups',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'group').length,
			icon: 'tabler:users-group'
		},
		{
			id: 'local:channel',
			label: 'Channels',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'channel').length,
			icon: 'tabler:speakerphone'
		},
		{
			id: 'local:bot',
			label: 'Bots',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'bot' || telegramChatIsBot(chat)).length,
			icon: 'tabler:robot'
		}
	];

	const telegramFolders = new Map<string, TelegramChatGroupFilter>();
	for (const chat of chats) {
		for (const label of telegramFolderLabels(chat)) {
			const id = `telegram:${label}`;
			const current = telegramFolders.get(id);
			telegramFolders.set(id, {
				id,
				label,
				source: 'telegram',
				count: (current?.count ?? 0) + 1,
				icon: label === 'Archived' ? 'tabler:archive' : 'tabler:folder'
			});
		}
	}

	return [...localGroups, ...Array.from(telegramFolders.values()).sort((left, right) => left.label.localeCompare(right.label))];
}

export function filterTelegramChatsByGroup(
	chats: TelegramChat[],
	groupFilterId: string
): TelegramChat[] {
	if (!groupFilterId || groupFilterId === 'local:all') return chats;
	if (groupFilterId === 'local:private') return chats.filter((chat) => chat.chat_kind === 'private');
	if (groupFilterId === 'local:group') return chats.filter((chat) => chat.chat_kind === 'group');
	if (groupFilterId === 'local:channel') return chats.filter((chat) => chat.chat_kind === 'channel');
	if (groupFilterId === 'local:bot') {
		return chats.filter((chat) => chat.chat_kind === 'bot' || telegramChatIsBot(chat));
	}
	if (groupFilterId.startsWith('telegram:')) {
		const label = groupFilterId.slice('telegram:'.length);
		return chats.filter((chat) => telegramFolderLabels(chat).includes(label));
	}
	return chats;
}

export function filterTelegramChats(
	chats: TelegramChat[],
	messages: TelegramMessage[],
	query: string,
	filter: TelegramChatFilter
): TelegramChat[] {
	const normalizedQuery = query.trim().toLowerCase();
	return chats.filter((chat) => {
		if (!matchesTelegramChatFilter(chat, messages, filter)) return false;
		if (!normalizedQuery) return true;
		const searchable = [
			chat.title,
			chat.username ?? '',
			chat.account_id,
			chat.provider_chat_id,
			chat.chat_kind,
			telegramChatPreview(chat, messages)
		]
			.join(' ')
			.toLowerCase();
		return searchable.includes(normalizedQuery);
	});
}

export function telegramAttachmentHintsForMessages(
	messages: TelegramMessage[]
): TelegramAttachmentHint[] {
	return messages.flatMap(telegramMessageAttachmentHints);
}

export function telegramMessageAttachmentHints(message: TelegramMessage): TelegramAttachmentHint[] {
	const explicit = explicitAttachmentHints(message);
	if (explicit.length) return explicit;

	const tdlibRaw = metadataRecord(message.metadata, ['tdlib_raw']);
	const content = tdlibRaw ? valueRecord(tdlibRaw.content) : null;
	if (!content) return [];

	const contentType = valueString(content['@type']);
	const baseId = `${message.message_id}:${contentType || 'attachment'}`;
	if (contentType === 'messageDocument') {
		const document = valueRecord(content.document);
		const file = document ? valueRecord(document.document) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'document', {
				fileName: valueString(document?.file_name) || 'document',
				mimeType: valueString(document?.mime_type),
				file
			})
		];
	}
	if (contentType === 'messagePhoto') {
		const photo = valueRecord(content.photo);
		const sizes = valueArray(photo?.sizes);
		const largest = valueRecord(sizes.at(-1));
		const file = largest ? valueRecord(largest.photo) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'photo', {
				fileName: `photo-${message.provider_message_id}.jpg`,
				mimeType: 'image/jpeg',
				file
			})
		];
	}
	if (contentType === 'messageVideo') {
		const video = valueRecord(content.video);
		const file = video ? valueRecord(video.video) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'video', {
				fileName: valueString(video?.file_name) || `video-${message.provider_message_id}.mp4`,
				mimeType: valueString(video?.mime_type),
				file
			})
		];
	}
	if (contentType === 'messageAudio') {
		const audio = valueRecord(content.audio);
		const file = audio ? valueRecord(audio.audio) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'audio', {
				fileName: valueString(audio?.file_name) || valueString(audio?.title) || 'audio',
				mimeType: valueString(audio?.mime_type),
				file
			})
		];
	}
	if (contentType === 'messageVoiceNote') {
		const voice = valueRecord(content.voice_note);
		const file = voice ? valueRecord(voice.voice) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'voice', {
				fileName: `voice-${message.provider_message_id}.ogg`,
				mimeType: valueString(voice?.mime_type),
				file
			})
		];
	}
	return [];
}

export function telegramLinkHintsForMessages(messages: TelegramMessage[]): TelegramLinkHint[] {
	const seen = new Set<string>();
	const links: TelegramLinkHint[] = [];
	for (const message of messages) {
		for (const url of message.text.match(/https?:\/\/[^\s<>()]+/g) ?? []) {
			const normalized = url.replace(/[.,!?;:]+$/, '');
			if (seen.has(`${message.message_id}:${normalized}`)) continue;
			seen.add(`${message.message_id}:${normalized}`);
			links.push({
				url: normalized,
				label: normalized.replace(/^https?:\/\//, ''),
				messageId: message.message_id,
				occurredAt: message.occurred_at ?? message.projected_at
			});
		}
	}
	return links;
}

export function telegramPinnedMessages(messages: TelegramMessage[]): TelegramMessage[] {
	return messages.filter((message) =>
		metadataBoolean(message.metadata, ['pinned', 'is_pinned', 'tdlib_raw.is_pinned'])
	);
}

function matchesTelegramChatFilter(
	chat: TelegramChat,
	messages: TelegramMessage[],
	filter: TelegramChatFilter
): boolean {
	switch (filter) {
		case 'unread':
			return telegramChatUnreadCount(chat) > 0;
		case 'mentions':
			return telegramChatMentionCount(chat, messages) > 0;
		case 'pinned':
			return telegramChatIsPinned(chat);
		case 'projects':
			return telegramChatIsProject(chat);
		case 'bots':
			return telegramChatIsBot(chat);
		case 'archived':
			return telegramChatIsArchived(chat);
		case 'all':
		default:
			return true;
	}
}

function telegramFolderLabels(chat: TelegramChat): string[] {
	const labels = new Set<string>();
	for (const key of ['folder', 'folder_name', 'telegram_folder', 'chat_folder', 'chat_folder_title', 'tdlib_folder']) {
		const value = valueString(chat.metadata[key]);
		if (value) labels.add(value);
	}

	const tdlibRaw = metadataRecord(chat.metadata, ['tdlib_raw']);
	for (const position of valueArray(tdlibRaw?.positions)) {
		const list = valueRecord(valueRecord(position)?.list);
		const listType = valueString(list?.['@type']);
		if (listType === 'chatListArchive') labels.add('Archived');
		if (listType === 'chatListMain') labels.add('Main');
		if (listType === 'chatListFolder') {
			const folderId = valueNumber(list?.chat_folder_id);
			labels.add(folderId == null ? 'Folder' : `Folder ${folderId}`);
		}
	}

	return Array.from(labels);
}

function explicitAttachmentHints(message: TelegramMessage): TelegramAttachmentHint[] {
	const attachments = metadataArray(message.metadata, ['attachments', 'files', 'media']);
	return attachments
		.map((value, index) => {
			const attachment = valueRecord(value);
			if (!attachment) return null;
			const fileName = valueString(attachment.filename) || valueString(attachment.file_name);
			if (!fileName) return null;
			const sizeBytes = valueNumber(attachment.size_bytes) ?? valueNumber(attachment.size);
			const tdlibFileId =
				valueNumber(attachment.tdlib_file_id) ??
				valueNumber(attachment.file_id) ??
				valueNumber(attachment.id);
			const providerAttachmentId =
				valueString(attachment.attachment_id) ||
				(tdlibFileId !== null ? `tdlib-file:${tdlibFileId}` : `${message.message_id}:attachment:${index}`);
			return {
				id: providerAttachmentId,
				kind: telegramAttachmentKind(valueString(attachment.kind) || valueString(attachment.content_type)),
				fileName,
				mimeType: valueString(attachment.content_type) || valueString(attachment.mime_type),
				sizeBytes,
				tdlibFileId,
				providerAttachmentId,
				downloadState: telegramDownloadState(attachment),
				localPath: valueString(attachment.storage_path) || valueString(attachment.local_path),
				messageId: message.message_id
			} satisfies TelegramAttachmentHint;
		})
		.filter((value): value is TelegramAttachmentHint => value !== null);
}

function attachmentFromTdlibFile(
	message: TelegramMessage,
	id: string,
	kind: TelegramAttachmentHint['kind'],
	params: {
		fileName: string;
		mimeType: string | null;
		file: Record<string, unknown> | null;
	}
): TelegramAttachmentHint {
	const file = params.file;
	const local = file ? valueRecord(file.local) : null;
	const remote = file ? valueRecord(file.remote) : null;
	const localPath = valueString(local?.path);
	const isDownloading = valueBoolean(local?.is_downloading_active);
	const isDownloaded = valueBoolean(local?.is_downloading_completed) || Boolean(localPath);
	const sizeBytes = valueNumber(file?.size) ?? valueNumber(file?.expected_size);
	const tdlibFileId = valueNumber(file?.id);
	const providerAttachmentId =
		tdlibFileId !== null ? `tdlib-file:${tdlibFileId}` : valueString(remote?.unique_id) || id;
	return {
		id: providerAttachmentId,
		kind,
		fileName: params.fileName,
		mimeType: params.mimeType,
		sizeBytes,
		tdlibFileId,
		providerAttachmentId,
		downloadState: isDownloaded ? 'downloaded' : isDownloading ? 'downloading' : 'remote',
		localPath,
		messageId: message.message_id
	};
}

function telegramAttachmentKind(value: string | null): TelegramAttachmentHint['kind'] {
	const normalized = value?.toLowerCase() ?? '';
	if (normalized.includes('image') || normalized.includes('photo')) return 'photo';
	if (normalized.includes('video')) return 'video';
	if (normalized.includes('audio')) return 'audio';
	if (normalized.includes('voice')) return 'voice';
	if (normalized.includes('document')) return 'document';
	return 'file';
}

function telegramDownloadState(
	attachment: Record<string, unknown>
): TelegramAttachmentHint['downloadState'] {
	const state = valueString(attachment.download_state) || valueString(attachment.status);
	if (state === 'downloaded' || state === 'complete' || state === 'completed') return 'downloaded';
	if (state === 'downloading' || state === 'in_progress') return 'downloading';
	if (state === 'remote' || state === 'pending') return 'remote';
	if (valueString(attachment.storage_path) || valueString(attachment.local_path)) return 'downloaded';
	return 'unknown';
}

function metadataNumber(metadata: Record<string, unknown>, paths: string[]): number {
	for (const path of paths) {
		const value = metadataPath(metadata, path);
		const parsed = valueNumber(value);
		if (parsed !== null) return parsed;
	}
	return 0;
}

function metadataString(metadata: Record<string, unknown>, paths: string[]): string {
	for (const path of paths) {
		const value = valueString(metadataPath(metadata, path));
		if (value) return value;
	}
	return '';
}

function metadataBoolean(metadata: Record<string, unknown>, paths: string[]): boolean {
	for (const path of paths) {
		const value = valueBoolean(metadataPath(metadata, path));
		if (value !== null) return value;
	}
	return false;
}

function metadataArray(metadata: Record<string, unknown>, paths: string[]): unknown[] {
	for (const path of paths) {
		const value = valueArray(metadataPath(metadata, path));
		if (value.length) return value;
	}
	return [];
}

function metadataRecord(
	metadata: Record<string, unknown>,
	paths: string[]
): Record<string, unknown> | null {
	for (const path of paths) {
		const value = valueRecord(metadataPath(metadata, path));
		if (value) return value;
	}
	return null;
}

function metadataPath(metadata: Record<string, unknown>, path: string): unknown {
	return path.split('.').reduce<unknown>((current, segment) => {
		if (Array.isArray(current)) {
			const index = Number.parseInt(segment, 10);
			return Number.isFinite(index) ? current[index] : undefined;
		}
		const record = valueRecord(current);
		if (!record) return undefined;
		return record[segment];
	}, metadata);
}

function valueRecord(value: unknown): Record<string, unknown> | null {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
		? (value as Record<string, unknown>)
		: null;
}

function valueArray(value: unknown): unknown[] {
	return Array.isArray(value) ? value : [];
}

function valueString(value: unknown): string | null {
	return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function valueNumber(value: unknown): number | null {
	if (typeof value === 'number' && Number.isFinite(value)) return value;
	if (typeof value === 'string' && value.trim()) {
		const parsed = Number.parseInt(value.trim(), 10);
		return Number.isFinite(parsed) ? parsed : null;
	}
	return null;
}

function valueBoolean(value: unknown): boolean | null {
	if (typeof value === 'boolean') return value;
	if (typeof value === 'string') {
		if (value === 'true') return true;
		if (value === 'false') return false;
	}
	return null;
}
