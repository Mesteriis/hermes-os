import { ApiClient } from '../client';
import type { GraphSummary, GraphNode, GraphNeighborhood } from '../types';

export async function fetchGraphSummary(): Promise<GraphSummary> {
	return ApiClient.instance.get<GraphSummary>('/api/v1/graph/summary', 'Graph summary request failed');
}

export async function fetchGraphNodes(limit = 20): Promise<GraphNode[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<GraphNode[]>(`/api/v1/graph/nodes?${params.toString()}`, 'Graph node picker request failed');
}

export async function searchGraphNodes(query: string, limit = 20): Promise<GraphNode[]> {
	const normalizedQuery = query.trim();
	if (!normalizedQuery) {
		return [];
	}

	const params = new URLSearchParams({
		q: normalizedQuery,
		limit: String(Math.trunc(limit))
	});

	return ApiClient.instance.get<GraphNode[]>(`/api/v1/graph/search?${params.toString()}`, 'Graph search request failed');
}

export async function fetchGraphNeighborhood(nodeId: string, depth = 1): Promise<GraphNeighborhood> {
	const params = new URLSearchParams({
		node_id: nodeId,
		depth: String(depth)
	});

	return ApiClient.instance.get<GraphNeighborhood>(`/api/v1/graph/neighborhood?${params.toString()}`, 'Graph neighborhood request failed');
}
