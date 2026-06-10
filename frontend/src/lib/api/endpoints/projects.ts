import { ApiClient } from '../client';
import type { ProjectListResponse, ProjectDetail } from '../types';

export async function fetchProjects(limit = 25): Promise<ProjectListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<ProjectListResponse>(
		`/api/v1/projects?${params.toString()}`,
		'Projects request failed'
	);
}

export async function fetchProjectDetail(projectId: string): Promise<ProjectDetail> {
	return ApiClient.instance.get<ProjectDetail>(
		`/api/v1/projects/${encodeURIComponent(projectId)}`,
		'Project detail request failed'
	);
}
