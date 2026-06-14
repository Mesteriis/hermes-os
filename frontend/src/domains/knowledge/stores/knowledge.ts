import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
	GraphSummary,
	GraphNode,
	GraphNeighborhood,
	GraphEdge,
	GraphEvidenceSummary,
	ContradictionObservation,
	ContradictionSeverity,
	GraphNodeKind
} from '../types/knowledge'
import {
	fetchGraphNodes,
	searchGraphNodes,
	fetchGraphNeighborhood,
	reviewContradiction
} from '../api/knowledge'

export interface GraphCanvasNode {
	node_id: string
	node_kind: GraphNodeKind
	label: string
	x: number
	y: number
	isSelected: boolean
	layoutClass: string
}

export interface GraphCanvasEdge {
	x1: number
	y1: number
	x2: number
	y2: number
	label: string
	review_state: string
}

export type GraphFilterChip = {
	kind: string
	label: string
	icon: string
	count: number
}

const RADIUS = 38

function buildRadialLayout(
	center: GraphNode,
	neighbors: GraphNode[],
	radius: number
): GraphCanvasNode[] {
	const maxNeighbors = 14
	const nodes: GraphCanvasNode[] = [
		{
			node_id: center.node_id,
			node_kind: center.node_kind,
			label: center.label,
			x: 50,
			y: 50,
			isSelected: true,
			layoutClass: 'center'
		}
	]

	const limited = neighbors.slice(0, maxNeighbors)
	const count = limited.length
	for (let i = 0; i < count; i++) {
		const angle = (2 * Math.PI * i) / count - Math.PI / 2
		nodes.push({
			node_id: limited[i].node_id,
			node_kind: limited[i].node_kind,
			label: limited[i].label,
			x: 50 + radius * Math.cos(angle),
			y: 50 + radius * Math.sin(angle),
			isSelected: false,
			layoutClass: `neighbor-${i}`
		})
	}
	return nodes
}

function buildEdges(
	centerId: string,
	edges: GraphEdge[],
	canvasNodes: GraphCanvasNode[]
): GraphCanvasEdge[] {
	const nodeMap = new Map(canvasNodes.map((n) => [n.node_id, n]))
	return edges.map((edge) => {
		const source = nodeMap.get(edge.source_node_id)
		const target = nodeMap.get(edge.target_node_id)
		return {
			x1: source?.x ?? 50,
			y1: source?.y ?? 50,
			x2: target?.x ?? 50,
			y2: target?.y ?? 50,
			label: edge.relationship_type.replace(/_/g, ' '),
			review_state: edge.review_state
		}
	})
}

export function graphNodeKindIcon(kind: string): string {
	switch (kind) {
		case 'person':
			return 'tabler:user'
		case 'email_address':
			return 'tabler:mail'
		case 'message':
			return 'tabler:message'
		case 'document':
			return 'tabler:file'
		case 'project':
			return 'tabler:folder'
		case 'organization':
			return 'tabler:building'
		case 'task':
			return 'tabler:checkbox'
		case 'event':
			return 'tabler:calendar'
		case 'decision':
			return 'tabler:scale'
		case 'obligation':
			return 'tabler:gavel'
		case 'knowledge':
			return 'tabler:brain'
		default:
			return 'tabler:circle'
	}
}

export function graphNodeKindLabel(kind: string): string {
	return kind
		.split('_')
		.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
		.join(' ')
}

export function contradictionSeverityTone(severity: ContradictionSeverity): ContradictionSeverity {
	return severity
}

export function formatContradictionClaim(observation: ContradictionObservation): string {
	return `${observation.old_claim} -> ${observation.new_claim}`
}

export function formatContradictionTime(value: string): string {
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) {
		return 'Unknown date'
	}
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(date)
}

export function formatContradictionSource(kind: string, sourceId: string): string {
	const label = kind
		.split('_')
		.map((p) => p.charAt(0).toUpperCase() + p.slice(1))
		.join(' ')
	return `${label} · ${sourceId}`
}

export const useKnowledgeStore = defineStore('knowledge', () => {
	const graphSummary = ref<GraphSummary | null>(null)
	const graphError = ref('')
	const graphSearchQuery = ref('')
	const graphSearchResults = ref<GraphNode[]>([])
	const graphNeighborhood = ref<GraphNeighborhood | null>(null)
	const selectedGraphNode = ref<GraphNode | null>(null)
	const contradictionObservations = ref<ContradictionObservation[]>([])
	const reviewingContradictionObservationId = ref<string | null>(null)

	const graphCanvasNodes = computed<GraphCanvasNode[]>(() => {
		const neighborhood = graphNeighborhood.value
		if (!neighborhood) return []
		return buildRadialLayout(neighborhood.selected_node, neighborhood.nodes, RADIUS)
	})

	const graphCanvasEdges = computed<GraphCanvasEdge[]>(() => {
		const neighborhood = graphNeighborhood.value
		if (!neighborhood) return []
		return buildEdges(neighborhood.selected_node.node_id, neighborhood.edges, graphCanvasNodes.value)
	})

	const selectedGraphProperties = computed(() => {
		const node = selectedGraphNode.value
		if (!node) return []
		return Object.entries(node.properties)
			.slice(0, 8)
			.sort(([a], [b]) => a.localeCompare(b))
			.map(([key, value]) => ({ key, value }))
	})

	const graphNeighborCounts = computed(() => {
		const neighborhood = graphNeighborhood.value
		if (!neighborhood) return []
		const counts = new Map<string, number>()
		for (const node of neighborhood.nodes) {
			counts.set(node.node_kind, (counts.get(node.node_kind) ?? 0) + 1)
		}
		return Array.from(counts.entries())
			.sort(([, a], [, b]) => b - a)
			.map(([kind, count]) => ({ kind, count }))
	})

	const graphFilterChips = computed<GraphFilterChip[]>(() => {
		const summary = graphSummary.value
		if (!summary) return []
		return summary.node_counts.map((c) => ({
			kind: c.key,
			label: graphNodeKindLabel(c.key),
			icon: graphNodeKindIcon(c.key),
			count: c.count
		}))
	})

	function setGraphSummary(summary: GraphSummary | null, error: string) {
		graphSummary.value = summary
		graphError.value = error
	}

	function setGraphSearchResults(results: GraphNode[], query: string) {
		graphSearchResults.value = results
		graphSearchQuery.value = query
	}

	async function selectGraphNode(node: GraphNode) {
		selectedGraphNode.value = node
		graphNeighborhood.value = null
		try {
			const neighborhood = await fetchGraphNeighborhood(node.node_id, 1)
			graphNeighborhood.value = neighborhood
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown graph neighborhood error'
		}
	}

	async function runGraphSearch(query: string) {
		if (!query.trim()) {
			graphSearchResults.value = []
			graphSearchQuery.value = ''
			return
		}
		try {
			const results = await searchGraphNodes(query, 20)
			graphSearchResults.value = results
			graphSearchQuery.value = query
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown graph search error'
		}
	}

	async function loadGraphNodeChoices() {
		try {
			const nodes = await fetchGraphNodes(20)
			return nodes
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown graph node picker error'
			return []
		}
	}

	function setContradictionObservations(observations: ContradictionObservation[]) {
		contradictionObservations.value = observations
	}

	async function reviewContradictionObservation(
		observation: ContradictionObservation,
		reviewState: Exclude<ContradictionObservation['review_state'], 'suggested'>,
		resolution?: string
	) {
		reviewingContradictionObservationId.value = observation.observation_id
		try {
			await reviewContradiction(observation.observation_id, {
				review_state: reviewState,
				resolution: resolution?.trim() || undefined
			})
			const idx = contradictionObservations.value.findIndex(
				(o) => o.observation_id === observation.observation_id
			)
			if (idx !== -1) {
				contradictionObservations.value[idx] = {
					...contradictionObservations.value[idx],
					review_state: reviewState
				}
			}
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown contradiction review action error'
		} finally {
			reviewingContradictionObservationId.value = null
		}
	}

	return {
		graphSummary,
		graphError,
		graphSearchQuery,
		graphSearchResults,
		graphNeighborhood,
		selectedGraphNode,
		contradictionObservations,
		reviewingContradictionObservationId,
		graphCanvasNodes,
		graphCanvasEdges,
		selectedGraphProperties,
		graphNeighborCounts,
		graphFilterChips,
		setGraphSummary,
		setGraphSearchResults,
		selectGraphNode,
		runGraphSearch,
		loadGraphNodeChoices,
		setContradictionObservations,
		reviewContradictionObservation
	}
})
