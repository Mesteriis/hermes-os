export type CanonicalCommunicationContentStatus =
	| 'idle'
	| 'loading'
	| 'ready'
	| 'unavailable'

export type CanonicalCommunicationContentModel = {
	status: CanonicalCommunicationContentStatus
	statusMessage: string
	bodyText: string
}

export function decodeCanonicalCommunicationContent(content: Uint8Array): string {
	try {
		return new TextDecoder('utf-8', { fatal: true }).decode(content)
	} catch {
		throw new Error('Canonical communication content is not valid UTF-8')
	}
}
