import { ApiClient } from '../api/ApiClient'
import type { FrontendConfig } from '../config/env'

export function initializeApiClient(config: FrontendConfig): ApiClient {
	return ApiClient.init(config.apiBaseUrl, config.apiSecret)
}
