import { beforeEach, describe, expect, it } from 'vitest'
import { ApiClient } from '../api/ApiClient'
import { initializeApiClient } from './api'

describe('initializeApiClient', () => {
	beforeEach(() => {
		ApiClient.resetForTests()
	})

	it('initializes the singleton from config', () => {
		initializeApiClient({
			apiBaseUrl: 'http://127.0.0.1:8080',
			apiSecret: 'dev-secret',
			sseUrl: 'http://127.0.0.1:8080/api/events/stream'
		})

		expect(ApiClient.instance).toBeInstanceOf(ApiClient)
	})
})
