import type { CommunicationSearchHitV1 } from '../../../gen/hermes/communications/query/v1/query_pb'
import { getCommunicationsQueryConnectClient } from '../../../platform/connect/communicationsQueryClient'

const MAX_SEARCH_LIMIT = 100

export async function searchCanonicalCommunications(
	query: string,
	limit = 20,
): Promise<readonly CommunicationSearchHitV1[]> {
	const normalizedQuery = query.trim()
	if (!normalizedQuery) {
		throw new RangeError('Communications search query must not be empty')
	}
	if (!Number.isInteger(limit) || limit < 1 || limit > MAX_SEARCH_LIMIT) {
		throw new RangeError(`Communications search limit must be between 1 and ${MAX_SEARCH_LIMIT}`)
	}

	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'searchCommunications',
			value: { query: normalizedQuery, limit },
		},
	})
	if (response.errorCode || response.result.case !== 'searchCommunications') {
		throw new Error('Communications canonical search is unavailable')
	}

	return response.result.value.hits
}
