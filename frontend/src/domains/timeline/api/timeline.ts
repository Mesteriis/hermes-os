import { ApiClient } from '../../../platform/api/ApiClient'
import type { TimelineMessage } from '../types/timeline'

export async function fetchCommunicationMessages(limit = 500): Promise<TimelineMessage[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
	const response = await ApiClient.instance.get<{ items: TimelineMessage[] }>(
		`/api/v1/communications/messages?${params.toString()}`,
		'Communication messages request failed'
	)
	return response.items ?? []
}
