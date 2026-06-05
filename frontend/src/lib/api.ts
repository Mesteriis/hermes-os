export type V1Status = {
	version: string;
	surfaces: {
		messages: boolean;
		contacts: boolean;
		search: boolean;
		documents: boolean;
		account_setup: boolean;
	};
};

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
	attachment_count: number;
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
	occurred_at: string | null;
	projected_at: string;
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

export type CommunicationMessageDetail = {
	message: CommunicationMessageDetailItem;
	attachments: CommunicationAttachment[];
};

export type GraphNodeKind = 'person' | 'email_address' | 'message' | 'document';

export type GraphRelationshipType =
	| 'person_has_email_address'
	| 'person_sent_message'
	| 'person_received_message'
	| 'email_address_sent_message'
	| 'email_address_received_message';

export type GraphReviewState =
	| 'system_accepted'
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type GraphEvidenceSourceKind = 'contact' | 'message' | 'document' | 'raw_record';

export type GraphNode = {
	node_id: string;
	node_kind: GraphNodeKind;
	stable_key: string;
	label: string;
	properties: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type GraphEdge = {
	edge_id: string;
	source_node_id: string;
	target_node_id: string;
	relationship_type: GraphRelationshipType;
	confidence: number;
	review_state: GraphReviewState;
	properties: Record<string, unknown>;
	valid_from: string | null;
	valid_to: string | null;
	created_at: string;
	updated_at: string;
};

export type GraphCount = {
	key: string;
	count: number;
};

export type GraphSummary = {
	node_counts: GraphCount[];
	edge_counts: GraphCount[];
	evidence_count: number;
	latest_projection_at: string | null;
	is_empty: boolean;
};

export type GraphEvidenceSummary = {
	edge_id: string;
	source_kind: GraphEvidenceSourceKind;
	source_id: string;
	excerpt: string | null;
	metadata: Record<string, unknown>;
};

export type GraphNeighborhood = {
	selected_node: GraphNode;
	nodes: GraphNode[];
	edges: GraphEdge[];
	evidence: GraphEvidenceSummary[];
	edge_limit: number;
	truncated: boolean;
	evidence_limit: number;
	evidence_truncated: boolean;
};

export type GmailOAuthStartRequest = {
	account_id: string;
	display_name: string;
	external_account_id: string;
	client_id: string;
	client_secret?: string;
	redirect_uri: string;
};

export type GmailOAuthStartResponse = {
	setup_id: string;
	authorization_url: string;
	state: string;
	redirect_uri: string;
};

export type GmailOAuthCompleteRequest = {
	setup_id: string;
	state: string;
	authorization_code: string;
};

export type EmailAccountSetupResponse = {
	account_id: string;
	secret_ref: string;
	secret_kind: 'oauth_token' | 'app_password' | 'password';
	store_kind: 'encrypted_vault';
};

export type ImapAccountSetupRequest = {
	account_id: string;
	provider_kind: 'icloud' | 'imap';
	display_name: string;
	external_account_id: string;
	host: string;
	port: number;
	tls: boolean;
	mailbox: string;
	username: string;
	password: string;
	secret_kind: 'app_password' | 'password';
};

export async function fetchV1Status(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<V1Status> {
	return getJson(baseUrl, token, actorId, '/api/v1/status', 'V1 status request failed');
}

export async function fetchCommunicationMessages(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 50
): Promise<CommunicationMessagesResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v1/communications/messages?${params.toString()}`,
		'Communication messages request failed'
	);
}

export async function fetchCommunicationMessage(
	baseUrl: string,
	token: string,
	actorId: string,
	messageId: string
): Promise<CommunicationMessageDetail> {
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}`,
		'Communication message detail request failed'
	);
}

export async function fetchGraphSummary(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<GraphSummary> {
	return getJson(baseUrl, token, actorId, '/api/v2/graph/summary', 'Graph summary request failed');
}

export async function searchGraphNodes(
	baseUrl: string,
	token: string,
	actorId: string,
	query: string,
	limit = 20
): Promise<GraphNode[]> {
	const normalizedQuery = query.trim();
	if (!normalizedQuery) {
		return [];
	}

	const params = new URLSearchParams({
		q: normalizedQuery,
		limit: String(Math.trunc(limit))
	});

	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/graph/search?${params.toString()}`,
		'Graph search request failed'
	);
}

export async function fetchGraphNeighborhood(
	baseUrl: string,
	token: string,
	actorId: string,
	nodeId: string,
	depth = 1
): Promise<GraphNeighborhood> {
	const params = new URLSearchParams({
		node_id: nodeId,
		depth: String(depth)
	});

	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/graph/neighborhood?${params.toString()}`,
		'Graph neighborhood request failed'
	);
}

export async function startGmailOAuthSetup(
	baseUrl: string,
	token: string,
	actorId: string,
	request: GmailOAuthStartRequest
): Promise<GmailOAuthStartResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/gmail/oauth/start', request);
}

export async function completeGmailOAuthSetup(
	baseUrl: string,
	token: string,
	actorId: string,
	request: GmailOAuthCompleteRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/gmail/oauth/complete', request);
}

export async function setupImapAccount(
	baseUrl: string,
	token: string,
	actorId: string,
	request: ImapAccountSetupRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/imap', request);
}

async function getJson<TResponse>(
	baseUrl: string,
	token: string,
	actorId: string,
	path: string,
	fallbackMessage: string
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		headers: {
			Authorization: `Bearer ${token}`,
			'X-Hermes-Actor-Id': actorId
		}
	});

	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as
			| { message?: string }
			| null;
		throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
	}

	return (await response.json()) as TResponse;
}

async function postJson<TResponse>(
	baseUrl: string,
	token: string,
	actorId: string,
	path: string,
	body: unknown
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		method: 'POST',
		headers: {
			Authorization: `Bearer ${token}`,
			'Content-Type': 'application/json',
			'X-Hermes-Actor-Id': actorId
		},
		body: JSON.stringify(body)
	});

	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as
			| { message?: string }
			| null;
		throw new Error(error?.message ?? `Account setup request failed: ${response.status}`);
	}

	return (await response.json()) as TResponse;
}
