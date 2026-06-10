import { ApiClient } from '../client';
import type { OrganizationListResponse, Organization } from '../types';

export async function fetchOrganizations(limit = 50): Promise<OrganizationListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<OrganizationListResponse>(
		`/api/v1/organizations?${params.toString()}`,
		'Organizations request failed'
	);
}

export async function fetchOrganization(orgId: string): Promise<Organization> {
	return ApiClient.instance.get<Organization>(
		`/api/v1/organizations/${encodeURIComponent(orgId)}`,
		'Organization request failed'
	);
}
