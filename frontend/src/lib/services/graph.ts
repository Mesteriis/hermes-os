import {
	fetchGraphSummary,
	fetchGraphNodes,
	searchGraphNodes,
	fetchGraphNeighborhood,
	type GraphSummary,
	type GraphNode,
	type GraphNeighborhood
} from '$lib/api';

export async function loadGraphSummary(
	requestSequence: number
): Promise<{ summary: GraphSummary | null; error: string; isLoading: boolean }> {
	try {
		const graphSummary = await fetchGraphSummary();
		return { summary: graphSummary, error: '', isLoading: false };
	} catch (error) {
		return {
			summary: null,
			error: error instanceof Error ? error.message : 'Unknown graph summary error',
			isLoading: false
		};
	}
}

export async function loadGraphNodeChoices(
	requestSequence: number,
	expectedSequence: number
): Promise<{ nodes: GraphNode[]; error: string; isLoading: boolean; sequence: number }> {
	try {
		const nodes = await fetchGraphNodes(20);
		if (requestSequence !== expectedSequence) {
			return { nodes: [], error: '', isLoading: true, sequence: expectedSequence };
		}
		return { nodes, error: '', isLoading: false, sequence: expectedSequence };
	} catch (error) {
		if (requestSequence !== expectedSequence) {
			return { nodes: [], error: '', isLoading: true, sequence: expectedSequence };
		}
		return {
			nodes: [],
			error: error instanceof Error ? error.message : 'Unknown graph node picker error',
			isLoading: false,
			sequence: expectedSequence
		};
	}
}

export async function runGraphSearch(
	query: string,
	requestSequence: number,
	expectedSequence: number
): Promise<{
	results: GraphNode[];
	error: string;
	isLoading: boolean;
	query: string;
}> {
	if (!query) {
		return { results: [], error: '', isLoading: false, query };
	}

	try {
		const results = await searchGraphNodes(query, 20);
		if (requestSequence !== expectedSequence) {
			return { results: [], error: '', isLoading: true, query };
		}
		return { results, error: '', isLoading: false, query };
	} catch (error) {
		if (requestSequence !== expectedSequence) {
			return { results: [], error: '', isLoading: true, query };
		}
		return {
			results: [],
			error: error instanceof Error ? error.message : 'Unknown graph search error',
			isLoading: false,
			query
		};
	}
}

export async function selectGraphNode(
	node: GraphNode,
	requestSequence: number,
	expectedSequence: number
): Promise<{
	neighborhood: GraphNeighborhood | null;
	error: string;
	isLoading: boolean;
	sequence: number;
}> {
	try {
		const neighborhood = await fetchGraphNeighborhood(
			node.node_id,
			1
		);
		if (requestSequence !== expectedSequence) {
			return { neighborhood: null, error: '', isLoading: true, sequence: expectedSequence };
		}
		return { neighborhood, error: '', isLoading: false, sequence: expectedSequence };
	} catch (error) {
		if (requestSequence !== expectedSequence) {
			return { neighborhood: null, error: '', isLoading: true, sequence: expectedSequence };
		}
		return {
			neighborhood: null,
			error: error instanceof Error ? error.message : 'Unknown graph neighborhood error',
			isLoading: false,
			sequence: expectedSequence
		};
	}
}
