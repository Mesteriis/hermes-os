export interface AiStatus {
	runtime: string
	status: string
	version: string | null
	chat_model: string
	embedding_model: string
	embedding_dimension: number
	chat_model_available: boolean
	embedding_model_available: boolean
}

export interface AiAgent {
	agent_id: string
	display_name: string
	role: string
	default_model: string
	status: string
	persona_id?: string
	persona_type?: string
	persona_email?: string
}

export interface AiAgentListResponse {
	items: AiAgent[]
}

export interface AiCitation {
	source_kind: string
	source_id: string
	title: string
	excerpt: string
	score: number
	graph_node_id?: string
}

export interface AiRun {
	run_id: string
	agent_id: string
	status: string
	chat_model: string
	embedding_model: string
	prompt_template_version: string
	model_config: Record<string, unknown>
	query: string
	answer: string | null
	citations: AiCitation[] | unknown[]
	error_summary: string | null
	actor_id: string
	causation_id: string | null
	correlation_id: string | null
	requested_event_id: string | null
	completed_event_id: string | null
	failed_event_id: string | null
	started_at: string
	completed_at: string | null
	duration_ms: number | null
	created_at: string
	updated_at: string
}

export interface AiRunListResponse {
	items: AiRun[]
}

export interface OwnerPersona {
	person_id: string
	display_name: string
	email_address: string
	persona_type: string
	is_self: boolean
	created_at: string
	updated_at: string
}

export interface OwnerPersonaResponse {
	owner_persona: OwnerPersona | null
}

export interface AiAnswerRequest {
	command_id: string
	query: string
	agent_id?: string
	correlation_id?: string
}

export interface AiAnswerResponse {
	run_id: string
	agent_id: string
	status: string
	answer: string
	citations: AiCitation[]
	model: string
	embedding_model: string
	created_at: string
	duration_ms: number
}

export interface AiMeetingPrepRequest {
	command_id: string
	topic: string
	project_id?: string
	person_id?: string
	correlation_id?: string
}

export interface AiMeetingPrepResponse {
	run_id: string
	agent_id: string
	status: string
	briefing: string
	citations: AiCitation[]
	model: string
	embedding_model: string
	created_at: string
	duration_ms: number
}

export interface AiTaskCandidateRefreshRequest {
	command_id: string
	query: string
	correlation_id?: string
}

export interface AiTaskCandidateRefreshResponse {
	run_id: string
	agent_id: string
	status: string
	created_count: number
	citations: AiCitation[]
	model: string
	embedding_model: string
	created_at: string
	duration_ms: number
}

export interface AgentCard {
	agentId: string
	name: string
	icon: string
	tone: string
	summary: string
	status: string
	model: string
	tasks: number
	success: number
}
