export type GraphNodeKind =
	| 'person'
	| 'email_address'
	| 'message'
	| 'document'
	| 'project'
	| 'organization'
	| 'task'
	| 'event'
	| 'decision'
	| 'obligation'
	| 'knowledge';

export type GraphRelationshipType =
	| 'person_has_email_address'
	| 'person_sent_message'
	| 'person_received_message'
	| 'email_address_sent_message'
	| 'email_address_received_message'
	| 'project_has_message'
	| 'project_has_document'
	| 'project_involves_person'
	| 'project_involves_email_address'
	| 'entity_relationship';

export type GraphReviewState =
	| 'system_accepted'
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type GraphEvidenceSourceKind =
	| 'person'
	| 'message'
	| 'document'
	| 'raw_record'
	| 'relationship'
	| 'decision'
	| 'obligation';

export interface GraphNode {
	node_id: string;
	node_kind: GraphNodeKind;
	stable_key: string;
	label: string;
	properties: Record<string, unknown>;
	created_at: string;
	updated_at: string;
}

export interface GraphEdge {
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
}

export interface GraphCount {
	key: string;
	count: number;
}

export interface GraphSummary {
	node_counts: GraphCount[];
	edge_counts: GraphCount[];
	evidence_count: number;
	latest_projection_at: string | null;
	is_empty: boolean;
}

export interface GraphEvidenceSummary {
	edge_id: string;
	source_kind: GraphEvidenceSourceKind;
	source_id: string;
	excerpt: string | null;
	metadata: Record<string, unknown>;
}

export interface GraphNeighborhood {
	selected_node: GraphNode;
	nodes: GraphNode[];
	edges: GraphEdge[];
	evidence: GraphEvidenceSummary[];
	edge_limit: number;
	truncated: boolean;
	evidence_limit: number;
	evidence_truncated: boolean;
}

export type ContradictionSourceKind =
	| 'communication'
	| 'document'
	| 'event'
	| 'memory'
	| 'knowledge'
	| 'decision'
	| 'obligation'
	| 'task'
	| 'relationship'
	| 'raw_record';

export type ContradictionSeverity = 'low' | 'medium' | 'high' | 'critical';

export type ContradictionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export interface ContradictionObservation {
	observation_id: string;
	old_source_kind: ContradictionSourceKind;
	old_source_id: string;
	new_source_kind: ContradictionSourceKind;
	new_source_id: string;
	affected_entities: unknown;
	conflict_type: string;
	old_claim: string;
	new_claim: string;
	confidence: number;
	severity: ContradictionSeverity;
	review_state: ContradictionReviewState;
	metadata: Record<string, unknown>;
	reviewed_by: string | null;
	reviewed_at: string | null;
	resolution: string | null;
	created_at: string;
	updated_at: string;
}

export interface ContradictionListResponse {
	items: ContradictionObservation[];
}

export interface ContradictionReviewRequest {
	review_state: Exclude<ContradictionReviewState, 'suggested'>;
	resolution?: string;
}
