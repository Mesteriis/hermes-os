export type TelegramChatFilter =
	| 'all'
	| 'unread'
	| 'mentions'
	| 'pinned'
	| 'projects'
	| 'bots'
	| 'archived';

export type TelegramThreadTab = 'messages' | 'files' | 'links' | 'topics' | 'pinned' | 'timeline';
export type TelegramRailTab = 'context' | 'members' | 'about';

export type TelegramChatFilterCount = {
	filter: TelegramChatFilter;
	count: number;
};

export type TelegramChatGroupFilter = {
	id: string;
	label: string;
	source: 'local' | 'telegram';
	count: number;
	icon: string;
};

export type TelegramAttachmentHint = {
	id: string;
	kind: 'document' | 'photo' | 'video' | 'audio' | 'voice' | 'file';
	fileName: string;
	mimeType: string | null;
	sizeBytes: number | null;
	tdlibFileId: number | null;
	providerAttachmentId: string;
	downloadState: 'remote' | 'downloading' | 'downloaded' | 'unknown';
	localPath: string | null;
	messageId: string;
};

export type TelegramLinkHint = {
	url: string;
	label: string;
	messageId: string;
	occurredAt: string | null;
};
