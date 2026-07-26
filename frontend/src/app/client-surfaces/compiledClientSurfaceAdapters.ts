import type {
	ClientSurfaceAdapterId,
	ClientSurfaceMetadata,
} from '../../platform/client-runtime/clientSurfaces'

export const compiledClientSurfaceAdapterIds: readonly ClientSurfaceAdapterId[] = [
	'communications-owner',
	'mail-integration',
	'telegram-integration',
	'whatsapp-integration',
	'zulip-integration',
	'system-control',
]

export function hasCompiledClientSurfaceAdapter(surface: ClientSurfaceMetadata): boolean {
	return compiledClientSurfaceAdapterIds.includes(surface.adapterId)
}
