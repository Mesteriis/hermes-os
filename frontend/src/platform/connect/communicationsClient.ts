import { createClient } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'
import type { Client } from '@connectrpc/connect'
import { ApiClient } from '../api/ApiClient'
import { CommunicationsService } from '../../gen/hermes/communications/v1/communications_pb'

let communicationsClient: Client<typeof CommunicationsService> | null = null

function createCommunicationsConnectClient(): Client<typeof CommunicationsService> {
	const apiClient = ApiClient.instance
	const secret = apiClient.getSecret()
	const transport = createConnectTransport({
		baseUrl: apiClient.getBaseUrl(),
		useBinaryFormat: false,
		fetch: (input, init) => {
			const headers = new Headers(init?.headers)
			headers.set('X-Hermes-Secret', secret)
			return fetch(input, {
				...init,
				headers
			})
		}
	})

	return createClient(CommunicationsService, transport)
}

export function getCommunicationsConnectClient(): Client<typeof CommunicationsService> {
	if (!communicationsClient) {
		communicationsClient = createCommunicationsConnectClient()
	}

	return communicationsClient
}

export function resetCommunicationsConnectClientForTests(): void {
	communicationsClient = null
}
