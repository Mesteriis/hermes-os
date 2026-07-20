import {
	ClientSurfaceAvailabilityStateV1,
	ClientSurfaceIdV1,
} from '../../gen/hermes/gateway/v1/client_bootstrap_pb'

export const CLIENT_SURFACE_CONTRACT_MAJOR = 1

export type ClientSurfaceRouteId =
	| 'dashboard'
	| 'communications-mail'
	| 'communications-telegram'
	| 'communications-whatsapp'
	| 'review'
	| 'personas'
	| 'knowledge'
	| 'tasks'
	| 'calendar'
	| 'documents'
	| 'settings'

export type ClientSurfaceMetadata = {
	routeId: ClientSurfaceRouteId
	surfaceId: ClientSurfaceIdV1
	label: string
	icon: string
	iconTone: ClientSurfaceIconTone
	adapterKind: 'owner-contract' | 'system-control'
	parentRouteId?: 'communications'
}

export type ClientSurfaceIconTone =
	| 'calendar'
	| 'communication'
	| 'dashboard'
	| 'documents'
	| 'knowledge'
	| 'mail'
	| 'review'
	| 'settings'
	| 'tasks'
	| 'telegram'
	| 'whatsapp'

export type ClientSurfaceAvailability = {
	state: ClientSurfaceAvailabilityStateV1
	reasonCode: string
	available: boolean
}

export const clientSurfaceCatalog: readonly ClientSurfaceMetadata[] = [
	{ routeId: 'dashboard', surfaceId: ClientSurfaceIdV1.DASHBOARD, label: 'Dashboard', icon: 'tabler:layout-dashboard', iconTone: 'dashboard', adapterKind: 'owner-contract' },
	{ routeId: 'communications-mail', surfaceId: ClientSurfaceIdV1.COMMUNICATIONS_MAIL, label: 'Mail', icon: 'tabler:mail', iconTone: 'mail', adapterKind: 'owner-contract', parentRouteId: 'communications' },
	{ routeId: 'communications-telegram', surfaceId: ClientSurfaceIdV1.COMMUNICATIONS_TELEGRAM, label: 'Telegram', icon: 'tabler:brand-telegram', iconTone: 'telegram', adapterKind: 'owner-contract', parentRouteId: 'communications' },
	{ routeId: 'communications-whatsapp', surfaceId: ClientSurfaceIdV1.COMMUNICATIONS_WHATSAPP, label: 'WhatsApp', icon: 'tabler:brand-whatsapp', iconTone: 'whatsapp', adapterKind: 'owner-contract', parentRouteId: 'communications' },
	{ routeId: 'review', surfaceId: ClientSurfaceIdV1.REVIEW, label: 'Review', icon: 'tabler:clipboard-check', iconTone: 'review', adapterKind: 'owner-contract' },
	{ routeId: 'personas', surfaceId: ClientSurfaceIdV1.PERSONAS, label: 'Personas', icon: 'tabler:user-circle', iconTone: 'knowledge', adapterKind: 'owner-contract' },
	{ routeId: 'knowledge', surfaceId: ClientSurfaceIdV1.KNOWLEDGE, label: 'Knowledge', icon: 'tabler:share', iconTone: 'knowledge', adapterKind: 'owner-contract' },
	{ routeId: 'tasks', surfaceId: ClientSurfaceIdV1.TASKS, label: 'Tasks', icon: 'tabler:checkbox', iconTone: 'tasks', adapterKind: 'owner-contract' },
	{ routeId: 'calendar', surfaceId: ClientSurfaceIdV1.CALENDAR, label: 'Calendar', icon: 'tabler:calendar', iconTone: 'calendar', adapterKind: 'owner-contract' },
	{ routeId: 'documents', surfaceId: ClientSurfaceIdV1.DOCUMENTS, label: 'Documents', icon: 'tabler:file-text', iconTone: 'documents', adapterKind: 'owner-contract' },
	{ routeId: 'settings', surfaceId: ClientSurfaceIdV1.SETTINGS, label: 'Settings', icon: 'tabler:settings', iconTone: 'settings', adapterKind: 'system-control' },
]

const surfaceMetadataByWireId = new Map(
	clientSurfaceCatalog.map((surface) => [surface.surfaceId, surface]),
)

export function clientSurfaceByWireId(surfaceId: ClientSurfaceIdV1): ClientSurfaceMetadata | undefined {
	return surfaceMetadataByWireId.get(surfaceId)
}

export function hasCompiledClientSurfaceAdapter(surface: ClientSurfaceMetadata): boolean {
	return surface.adapterKind === 'system-control'
}

export function unavailableClientSurface(reasonCode = 'bootstrap_unavailable'): ClientSurfaceAvailability {
	return {
		state: ClientSurfaceAvailabilityStateV1.UNAVAILABLE,
		reasonCode,
		available: false,
	}
}
