import { ApiClient } from '../../../../platform/api/ApiClient'

export async function postCommunicationsConnectJson<T>(
  method: string,
  body: Record<string, unknown>
): Promise<T> {
  const apiClient = ApiClient.instance
  const response = await fetch(
    `${apiClient.getBaseUrl()}/hermes.communications.v1.CommunicationsService/${method}`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Hermes-Secret': apiClient.getSecret()
      },
      body: JSON.stringify(body)
    }
  )
  if (!response.ok) {
    throw new Error(`CommunicationsService/${method} failed with status ${response.status}`)
  }
  return (await response.json()) as T
}
