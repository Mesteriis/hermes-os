import type { PersonaType } from './persons';

export type AiStatus = {
	runtime: string;
	status: string;
	version: string | null;
	chat_model: string;
	embedding_model: string;
	embedding_dimension: number;
	chat_model_available: boolean;
	embedding_model_available: boolean;
};

export type AiAgent = {
	agent_id: 'HESTIA' | 'HERMES' | 'MNEMOSYNE' | 'ATHENA' | string;
	display_name: string;
	role: string;
	default_model: string;
	status: string;
	persona_id?: string;
	persona_type?: PersonaType;
	persona_email?: string;
};

export type AiAgentListResponse = {
	items: AiAgent[];
};

export type AiCitation = {
	source_kind: string;
	source_id: string;
	title: string;
	excerpt: string;
	score: number;
	graph_node_id?: string;
};

export type AiRun = {
	run_id: string;
	agent_id: string;
	status: 'requested' | 'completed' | 'failed' | string;
	chat_model: string;
	embedding_model: string;
	prompt_template_version: string;
	model_config: Record<string, unknown>;
	query: string;
	answer: string | null;
	citations: AiCitation[] | unknown[];
	error_summary: string | null;
	actor_id: string;
	causation_id: string | null;
	correlation_id: string | null;
	requested_event_id: string | null;
	completed_event_id: string | null;
	failed_event_id: string | null;
	started_at: string;
	completed_at: string | null;
	duration_ms: number | null;
	created_at: string;
	updated_at: string;
};

export type AiRunListResponse = {
	items: AiRun[];
};

export type AiProviderKind = 'built_in' | 'cli' | 'api' | string;

export type AiProviderAccount = {
	provider_id: string;
	provider_kind: AiProviderKind;
	provider_key: string;
	display_name: string;
	status: 'ready' | 'disabled' | 'needs_setup' | 'unavailable' | string;
	consent_state: 'not_required' | 'required' | 'granted' | string;
	consented_at: string | null;
	config: Record<string, unknown>;
	capabilities: string[];
	created_at: string;
	updated_at: string;
};

export type AiProviderPreset = {
	provider_kind: AiProviderKind;
	provider_key: string;
	display_name: string;
	privacy: 'local' | 'cli' | 'remote' | string;
	base_url: string | null;
	command_preset: string | null;
	capabilities: string[];
};

export type AiCapabilitySlot = {
	slot: string;
	label: string;
	description: string;
	requires_embedding_dimension: number | null;
};

export type AiModelCatalogItem = {
	model_key: string;
	provider_id: string;
	display_name: string;
	category: string;
	privacy: 'local' | 'cli' | 'remote' | string;
	capabilities: string[];
	context_window: number | null;
	embedding_dimension: number | null;
	is_available: boolean;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type AiModelRoute = {
	capability_slot: string;
	provider_id: string;
	model_key: string;
	created_at: string;
	updated_at: string;
};

export type AiPromptTemplate = {
	prompt_id: string;
	name: string;
	entity_scope: string;
	capability_slot: string;
	description: string | null;
	is_system: boolean;
	active_version_id: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type AiPromptVersion = {
	prompt_version_id: string;
	prompt_id: string;
	version_label: string;
	body_template: string;
	variables: string[];
	status: 'active' | 'draft' | string;
	created_by_actor_id: string;
	created_at: string;
	updated_at: string;
};

export type AiPromptEvalRun = {
	eval_run_id: string;
	prompt_id: string;
	prompt_version_id: string;
	provider_id: string;
	model_key: string;
	source_refs: Record<string, unknown>[];
	variables: Record<string, unknown>;
	output_text: string;
	score: number | null;
	notes: string | null;
	actor_id: string;
	created_at: string;
};

export type AiSettingsOverviewResponse = {
	providers: AiProviderAccount[];
	models: AiModelCatalogItem[];
	routes: AiModelRoute[];
	prompts: AiPromptTemplate[];
	eval_runs: AiPromptEvalRun[];
	capability_slots: AiCapabilitySlot[];
	provider_presets: AiProviderPreset[];
};

export type AiProviderListResponse = { items: AiProviderAccount[] };
export type AiModelListResponse = { items: AiModelCatalogItem[] };
export type AiPromptListResponse = { items: AiPromptTemplate[] };

export type AiProviderCreateRequest = {
	provider_id?: string;
	provider_kind: AiProviderKind;
	provider_key: string;
	display_name: string;
	base_url?: string;
	command_preset?: string;
	config?: Record<string, unknown>;
	capabilities?: string[];
	enabled?: boolean;
	remote_context_consent?: boolean;
	api_key?: string;
};

export type AiProviderPatchRequest = {
	display_name?: string;
	base_url?: string;
	config?: Record<string, unknown>;
	enabled?: boolean;
	api_key?: string;
};

export type AiProviderConsentRequest = {
	consented: boolean;
};

export type AiProviderCommandResponse = {
	provider_id: string;
	command: 'test' | 'sync_models' | string;
	status: string;
	message: string;
};

export type AiModelRouteUpdateRequest = {
	provider_id: string;
	model_key: string;
};

export type AiPromptCreateRequest = {
	prompt_id?: string;
	name: string;
	entity_scope: string;
	capability_slot: string;
	description?: string;
	metadata?: Record<string, unknown>;
};

export type AiPromptVersionCreateRequest = {
	prompt_version_id?: string;
	version_label?: string;
	body_template: string;
	variables?: string[];
	metadata?: Record<string, unknown>;
};

export type AiPromptActivateRequest = {
	prompt_version_id: string;
};

export type AiPromptTestRequest = {
	prompt_version_id?: string;
	provider_id: string;
	model_key: string;
	variables?: Record<string, unknown>;
	source_refs?: Record<string, unknown>[];
	score?: number;
	notes?: string;
};

export type AiAnswerRequest = {
	command_id: string;
	query: string;
	agent_id?: string;
	correlation_id?: string;
};

export type AiAnswerResponse = {
	run_id: string;
	agent_id: string;
	status: string;
	answer: string;
	citations: AiCitation[];
	model: string;
	embedding_model: string;
	created_at: string;
	duration_ms: number;
};

export type AiTaskCandidateRefreshRequest = {
	command_id: string;
	query: string;
	correlation_id?: string;
};

export type AiTaskCandidateRefreshResponse = {
	run_id: string;
	agent_id: string;
	status: string;
	created_count: number;
	citations: AiCitation[];
	model: string;
	embedding_model: string;
	created_at: string;
	duration_ms: number;
};

export type AiMeetingPrepRequest = {
	command_id: string;
	topic: string;
	project_id?: string;
	person_id?: string;
	correlation_id?: string;
};

export type AiMeetingPrepResponse = {
	run_id: string;
	agent_id: string;
	status: string;
	briefing: string;
	citations: AiCitation[];
	model: string;
	embedding_model: string;
	created_at: string;
	duration_ms: number;
};
