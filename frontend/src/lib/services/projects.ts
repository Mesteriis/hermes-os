import {
	fetchProjects,
	fetchProjectDetail,
	type ProjectSummary,
	type ProjectDetail,
	type ProjectStats,
	type ProjectTimelineItem,
	type ProjectDocumentSummary,
	type ProjectMessageSummary
} from '$lib/api';
import { senderLabel } from './communications';
import { formatDateTime, formatProjectDate, formatProjectDateTime } from './formatting';

export function emptyProjectStats(): ProjectStats {
	return {
		message_count: 0,
		document_count: 0,
		people_count: 0,
		graph_connection_count: 0,
		latest_activity_at: null
	};
}

export async function loadProjects(
	selectedProjectId: string,
	requestSequence: number
): Promise<{
	summaries: ProjectSummary[];
	detail: ProjectDetail | null;
	error: string;
	isLoading: boolean;
	selectedProjectId: string;
}> {
	try {
		const response = await fetchProjects(25);
		const summaries = response.items;
		const nextProjectId = selectedProjectId || summaries[0]?.project.project_id || '';
		let detail: ProjectDetail | null = null;
		if (nextProjectId) {
			try {
				detail = await fetchProjectDetail(nextProjectId);
			} catch { /* detail optional */ }
		}
		return {
			summaries,
			detail: detail && detail.project.project_id === nextProjectId ? detail : null,
			error: '',
			isLoading: false,
			selectedProjectId: detail?.project.project_id ?? nextProjectId
		};
	} catch (error) {
		return {
			summaries: [],
			detail: null,
			error: error instanceof Error ? error.message : 'Unknown projects error',
			isLoading: false,
			selectedProjectId: ''
		};
	}
}

export async function loadProjectDetail(
	projectId: string
): Promise<{ detail: ProjectDetail | null; error: string }> {
	if (!projectId) {
		return { detail: null, error: '' };
	}
	try {
		const detail = await fetchProjectDetail(projectId);
		return { detail, error: '' };
	} catch (error) {
		return {
			detail: null,
			error: error instanceof Error ? error.message : 'Unknown project detail error'
		};
	}
}

export function selectProject(
	project: ProjectSummary,
	selectedProjectId: string,
	hasDetail: boolean
): { shouldLoad: boolean; projectId: string } {
	if (project.project.project_id === selectedProjectId && hasDetail) {
		return { shouldLoad: false, projectId: selectedProjectId };
	}
	return { shouldLoad: true, projectId: project.project.project_id };
}

export function projectStatusLabel(status: string) {
	return status
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function projectTimelineIcon(item: ProjectTimelineItem) {
	switch (item.item_kind) {
		case 'message':
			return 'tabler:mail';
		case 'document':
			return 'tabler:file-text';
		default:
			return 'tabler:circle-dot';
	}
}

export function projectDocumentIcon(document: ProjectDocumentSummary) {
	switch (document.document_kind) {
		case 'pdf':
			return 'tabler:file-type-pdf';
		case 'markdown':
			return 'tabler:file-text';
		default:
			return 'tabler:file';
	}
}

export function projectMessageSender(message: ProjectMessageSummary) {
	return senderLabel(message.sender);
}

export { formatProjectDate, formatProjectDateTime };
