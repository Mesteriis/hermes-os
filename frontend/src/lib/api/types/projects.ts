export type ProjectRecord = {
	project_id: string;
	name: string;
	kind: string;
	status: 'planning' | 'active' | 'on_hold' | 'completed' | 'archived';
	description: string;
	owner_display_name: string;
	progress_percent: number;
	start_date: string | null;
	target_date: string | null;
	created_at: string;
	updated_at: string;
};

export type ProjectStats = {
	message_count: number;
	document_count: number;
	people_count: number;
	graph_connection_count: number;
	latest_activity_at: string | null;
};

export type ProjectSummary = {
	project: ProjectRecord;
	stats: ProjectStats;
	graph_node_id: string;
};

export type ProjectTimelineItem = {
	item_kind: 'message' | 'document' | string;
	item_id: string;
	title: string;
	subtitle: string;
	occurred_at: string;
};

export type ProjectPersonSummary = {
	display_name: string;
	email_address: string;
	interaction_count: number;
	last_interaction_at: string | null;
};

export type ProjectMessageSummary = {
	message_id: string;
	subject: string;
	sender: string;
	occurred_at: string;
};

export type ProjectDocumentSummary = {
	document_id: string;
	document_kind: string;
	title: string;
	imported_at: string;
};

export type ProjectDetail = {
	project: ProjectRecord;
	stats: ProjectStats;
	graph_node_id: string;
	timeline: ProjectTimelineItem[];
	key_people: ProjectPersonSummary[];
	recent_messages: ProjectMessageSummary[];
	documents: ProjectDocumentSummary[];
};

export type ProjectListResponse = {
	items: ProjectSummary[];
};
