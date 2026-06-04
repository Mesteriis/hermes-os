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
