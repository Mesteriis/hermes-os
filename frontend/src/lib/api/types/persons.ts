export type PersonIdentityReviewState =
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type PersonIdentityCandidate = {
	identity_candidate_id: string;
	candidate_kind: 'merge_persons' | 'attach_email_address' | 'split_person';
	left_person_id: string;
	right_person_id: string | null;
	email_address: string | null;
	evidence_summary: string;
	confidence: number;
	review_state: PersonIdentityReviewState;
	generated_at: string;
	reviewed_at: string | null;
	updated_at: string;
};

export type PersonIdentityCandidateListResponse = {
	items: PersonIdentityCandidate[];
};

export type PersonaType = 'human' | 'ai_agent' | 'organization_proxy' | 'system';

export type PersonaReadModel = {
	persona_id: string;
	persona_type: PersonaType;
	is_self: boolean;
	identity: {
		display_name: string;
		email_address: string;
	};
	communication: {
		primary_email: string;
	};
	compatibility: {
		legacy_person_id: string;
		legacy_route: string;
	};
	created_at: string;
	updated_at: string;
};

export type PersonaListResponse = {
	items: PersonaReadModel[];
};

export type PersonaUpdateRequest = {
	identity?: {
		display_name?: string;
	};
	is_self?: boolean;
};

export type OwnerPersona = {
	person_id: string;
	display_name: string;
	email_address: string;
	persona_type: PersonaType;
	is_self: boolean;
	created_at: string;
	updated_at: string;
};

export type OwnerPersonaResponse = {
	owner_persona: OwnerPersona | null;
};

export type DossierSectionItem = {
	label: string;
	value: string;
	source_refs: string[];
	confidence: number | null;
};

export type PersonDossierPerson = {
	person_id: string;
	display_name: string;
	email_address: string;
	[key: string]: unknown;
};

export type PersonDossier = {
	person: PersonDossierPerson;
	facts?: unknown[];
	memory_cards?: unknown[];
	timeline?: unknown[];
	identities?: unknown[];
	expertise?: unknown[];
	promises?: unknown[];
	risks?: unknown[];
	summary: string;
	interests: DossierSectionItem[];
	projects: DossierSectionItem[];
	organizations: DossierSectionItem[];
	skills: DossierSectionItem[];
	communication_patterns: DossierSectionItem[];
	ai_observations: DossierSectionItem[];
	source_refs: string[];
	generated_at: string;
};

export type PersonIdentity = {
	id: string;
	person_id: string | null;
	identity_type: string;
	identity_value: string;
	source: string;
	confidence: number;
	last_verified_at: string | null;
	status: string;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type PersonIdentityTraceListResponse = {
	items: PersonIdentity[];
};

export type IdentityTraceListStatus = 'unattached';

export type NewIdentityTraceRequest = {
	identity_type: string;
	identity_value: string;
	source?: string;
};

export type IdentityTraceAssignmentRequest = {
	person_id: string;
};

export type EnrichedPerson = {
	person_id: string;
	display_name: string;
	email_address: string;
	language: string | null;
	tone: string | null;
	trust_score: number | null;
	avg_response_hours: number | null;
	preferred_channel: string | null;
	last_interaction_at: string | null;
	interaction_count: number;
	frequent_topics: string[];
	writing_style: string | null;
	person_metadata: Record<string, unknown>;
	is_favorite: boolean;
	notes: string | null;
	linked_projects: string[];
	linked_documents: string[];
	created_at: string;
	updated_at: string;
};

export type PersonListResponse = {
	items: EnrichedPerson[];
};
