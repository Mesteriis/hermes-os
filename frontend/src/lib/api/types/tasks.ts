export type TaskCandidateReviewState =
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type TaskCandidate = {
	task_candidate_id: string;
	source_kind: 'message' | 'document';
	source_id: string;
	project_id: string | null;
	title: string;
	due_text: string | null;
	assignee_label: string | null;
	confidence: number;
	review_state: TaskCandidateReviewState;
	evidence_excerpt: string;
	generated_at: string;
	reviewed_at: string | null;
	updated_at: string;
};

export type TaskCandidateListResponse = {
	items: TaskCandidate[];
};

export type Task = {
	task_id: string;
	task_candidate_id: string | null;
	title: string;
	description: string | null;
	source_kind: string;
	source_id: string;
	source_type: string;
	project_id: string | null;
	status: string;
	hermes_status: string;
	priority_score: number | null;
	risk_score: number | null;
	readiness_score: number | null;
	area: string | null;
	why: string | null;
	outcome: string | null;
	due_at: string | null;
	completed_at: string | null;
	archived_at: string | null;
	waiting_reason: string | null;
	energy_type: string | null;
	confidentiality: string;
	tags: unknown[];
	task_metadata: Record<string, unknown>;
	linked_person_id: string | null;
	linked_organization_id: string | null;
	created_from_event_id: string | null;
	created_by_actor_id: string | null;
	created_at: string;
	updated_at: string;
};

export type TaskRecordsResponse = { items: Task[] };

export type TaskContextPack = {
	id: string;
	task_id: string;
	summary: string | null;
	source_summary: string | null;
	open_questions: unknown[];
	blockers: unknown[];
	risks: unknown[];
	suggested_next_action: string | null;
	generated_at: string;
	model: string | null;
};

export type TaskEvidence = {
	id: string; task_id: string; source_type: string;
	source_id: string; quote: string | null; confidence: number;
	created_at: string;
};

export type TaskEvidenceResponse = { items: TaskEvidence[] };

export type TaskRelation = {
	id: string; task_id: string; entity_type: string;
	entity_id: string; relation_type: string;
	source: string; confidence: number; created_at: string;
};

export type TaskRelationsResponse = { items: TaskRelation[] };

export type TaskChecklist = {
	id: string; task_id: string; items: unknown[];
	source: string; created_at: string; updated_at: string;
};

export type TaskSubtask = {
	id: string; parent_task_id: string; child_task_id: string;
	sort_order: number; created_at: string;
};

export type TaskSubtasksResponse = { items: TaskSubtask[] };

export type TaskProviderAccount = {
	account_id: string; provider: string; account_name: string;
	credentials_reference: string | null; sync_mode: string;
	capabilities: Record<string, unknown>;
};

export type TaskProvidersResponse = { items: TaskProviderAccount[] };

export type TaskRule = {
	rule_id: string; name: string; natural_language_description: string | null;
	compiled_dsl: Record<string, unknown>; enabled: boolean;
	approval_mode: string; last_run_at: string | null;
};

export type TaskRulesResponse = { items: TaskRule[] };

export type TaskTemplate = {
	template_id: string; name: string; description: string | null;
	default_fields: Record<string, unknown>; default_checklist: unknown[];
	default_priority: string; default_energy_type: string | null;
};

export type TaskTemplatesResponse = { items: TaskTemplate[] };
