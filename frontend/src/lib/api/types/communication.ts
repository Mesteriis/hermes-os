export type CommunicationMessageSummary = {
	message_id: string;
	raw_record_id: string;
	account_id: string;
	provider_record_id: string;
	subject: string;
	sender: string;
	recipients: string[];
	body_text_preview: string;
	occurred_at: string | null;
	projected_at: string;
	channel_kind: string;
	conversation_id: string | null;
	sender_display_name: string | null;
	delivery_state: string;
	message_metadata: Record<string, unknown>;
	attachment_count: number;
	local_state: LocalMessageState;
	local_state_changed_at: string | null;
};

export type CommunicationMessageDetailItem = {
	message_id: string;
	raw_record_id: string;
	account_id: string;
	provider_record_id: string;
	subject: string;
	sender: string;
	recipients: string[];
	body_text: string;
	body_html: string | null;
	occurred_at: string | null;
	projected_at: string;
	channel_kind: string;
	conversation_id: string | null;
	sender_display_name: string | null;
	delivery_state: string;
	message_metadata: Record<string, unknown>;
	local_state: LocalMessageState;
	local_state_changed_at: string | null;
	local_state_reason: string | null;
};

export type CommunicationAttachment = {
	attachment_id: string;
	message_id: string;
	raw_record_id: string;
	blob_id: string;
	provider_attachment_id: string;
	filename: string | null;
	content_type: string;
	size_bytes: number;
	sha256: string;
	disposition: 'attachment' | 'inline' | 'unknown';
	scan_status: 'not_scanned' | 'clean' | 'suspicious' | 'malicious' | 'failed';
	scan_engine: string | null;
	scan_checked_at: string | null;
	scan_summary: string | null;
	scan_metadata: Record<string, unknown>;
	storage_kind: string;
	storage_path: string;
	created_at: string;
	updated_at: string;
};

export type CommunicationMessagesResponse = {
	items: CommunicationMessageSummary[];
};

export type WorkflowState = 'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam';
export type LocalMessageState = 'active' | 'trash' | 'all';

export type CommunicationMessageDetail = {
	message: CommunicationMessageDetailItem;
	attachments: CommunicationAttachment[];
};
