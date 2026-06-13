export type CalendarAccount = {
	account_id: string;
	provider: string;
	account_name: string;
	email: string | null;
	credentials_reference: string | null;
	sync_status: string;
	capabilities: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type CalendarAccountsResponse = { items: CalendarAccount[] };

export type CalendarSource = {
	source_id: string;
	account_id: string;
	provider_calendar_id: string | null;
	name: string;
	color: string | null;
	timezone: string | null;
	visibility: string;
	read_only: boolean;
	sync_enabled: boolean;
	capabilities: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type CalendarSourcesResponse = { items: CalendarSource[] };

export type CalendarEvent = {
	event_id: string;
	source_event_id: string | null;
	account_id: string | null;
	source_id: string | null;
	title: string;
	description: string | null;
	location: string | null;
	start_at: string;
	end_at: string;
	timezone: string | null;
	all_day: boolean;
	recurrence_rule: string | null;
	status: string;
	visibility: string;
	event_type: string | null;
	importance_score: number | null;
	readiness_score: number | null;
	sync_status: string;
	created_at: string;
	updated_at: string;
};

export type CalendarEventsResponse = { items: CalendarEvent[] };

export type EventParticipant = {
	id: string;
	event_id: string;
	person_id: string | null;
	email: string;
	display_name: string | null;
	role: string;
	response_status: string;
	organization_id: string | null;
	timezone: string | null;
	confidence: number;
	created_at: string;
};

export type EventParticipantsResponse = { items: EventParticipant[] };

export type EventRelation = {
	id: string;
	event_id: string;
	entity_type: string;
	entity_id: string;
	relation_type: string;
	source: string;
	confidence: number;
	created_at: string;
};

export type EventRelationsResponse = { items: EventRelation[] };

export type EventContextPack = {
	id: string;
	event_id: string;
	summary: string | null;
	participants_summary: string | null;
	documents: unknown[];
	tasks: unknown[];
	open_questions: unknown[];
	risks: unknown[];
	suggested_agenda: unknown[];
	suggested_actions: unknown[];
	generated_at: string;
	model: string | null;
	created_at: string;
	updated_at: string;
};

export type EventAgenda = {
	id: string;
	event_id: string;
	items: unknown[];
	source: string;
	created_by: string | null;
	created_at: string;
	updated_at: string;
};

export type EventChecklist = {
	id: string;
	event_id: string;
	items: unknown[];
	source: string;
	created_at: string;
	updated_at: string;
};

export type MeetingNote = {
	id: string;
	event_id: string;
	content: string;
	format: string;
	source: string;
	linked_note_id: string | null;
	created_at: string;
	updated_at: string;
};

export type MeetingNotesResponse = { items: MeetingNote[] };

export type MeetingOutcome = {
	id: string;
	event_id: string;
	outcome_type: string;
	title: string;
	description: string | null;
	owner_person_id: string | null;
	due_date: string | null;
	source: string;
	confidence: number;
	linked_entity_id: string | null;
	created_at: string;
	updated_at: string;
};

export type MeetingOutcomesResponse = { items: MeetingOutcome[] };

export type DeadlineEvent = {
	id: string;
	source_entity_type: string | null;
	source_entity_id: string | null;
	title: string;
	due_at: string;
	severity: string;
	status: string;
	linked_calendar_event_id: string | null;
	created_at: string;
	updated_at: string;
};

export type DeadlinesResponse = { items: DeadlineEvent[] };

export type FocusBlock = {
	id: string;
	title: string;
	start_at: string;
	end_at: string;
	purpose: string | null;
	linked_project_id: string | null;
	protection_level: string;
	status: string;
	created_at: string;
	updated_at: string;
};

export type FocusBlocksResponse = { items: FocusBlock[] };

export type CalendarRule = {
	rule_id: string;
	name: string;
	natural_language_description: string | null;
	compiled_dsl: Record<string, unknown>;
	enabled: boolean;
	approval_mode: string;
	last_run_at: string | null;
	created_at: string;
	updated_at: string;
};

export type CalendarRulesResponse = { items: CalendarRule[] };
