import type { ApiError } from './types'

export class ApiClient {
	private baseUrl: string
	private secret: string

	constructor(baseUrl: string, secret: string) {
		this.baseUrl = baseUrl.replace(/\/+$/, '')
		this.secret = secret
	}

	private async request<T>(
		method: string,
		path: string,
		body?: unknown,
		fallbackMessage?: string
	): Promise<T> {
		const url = `${this.baseUrl}${path}`
		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			'X-Hermes-Secret': this.secret
		}

		const res = await fetch(url, {
			method,
			headers,
			body: body !== undefined ? JSON.stringify(body) : undefined
		})

		if (!res.ok) {
			let errorBody: string | undefined
			try {
				errorBody = await res.text()
			} catch {
				// ignore parse error
			}
			const err: ApiError = {
				message: errorBody ?? fallbackMessage ?? `${method} request failed`,
				status: res.status
			}
			throw err
		}

		// Handle 204 No Content
		if (res.status === 204) {
			return undefined as T
		}

		return res.json() as Promise<T>
	}

	async get<T>(path: string, fallbackMessage = 'GET request failed'): Promise<T> {
		return this.request<T>('GET', path, undefined, fallbackMessage)
	}

	async post<T>(path: string, body: unknown, fallbackMessage = 'POST request failed'): Promise<T> {
		return this.request<T>('POST', path, body, fallbackMessage)
	}

	async put<T>(path: string, body: unknown, fallbackMessage = 'PUT request failed'): Promise<T> {
		return this.request<T>('PUT', path, body, fallbackMessage)
	}

	async patch<T>(path: string, body: unknown, fallbackMessage = 'PATCH request failed'): Promise<T> {
		return this.request<T>('PATCH', path, body, fallbackMessage)
	}

	async delete<T>(path: string, fallbackMessage = 'DELETE request failed'): Promise<T> {
		return this.request<T>('DELETE', path, undefined, fallbackMessage)
	}

	async deleteWithBody<T>(path: string, body: unknown, fallbackMessage = 'DELETE request failed'): Promise<T> {
		return this.request<T>('DELETE', path, body, fallbackMessage)
	}

	private static _instance: ApiClient | null = null

	static get instance(): ApiClient {
		if (!ApiClient._instance) {
			throw new Error('ApiClient not initialized. Call ApiClient.init() first.')
		}
		return ApiClient._instance
	}

	static init(baseUrl: string, secret: string): ApiClient {
		if (secret.trim().length === 0) {
			throw new Error('X-Hermes-Secret cannot be empty')
		}

		ApiClient._instance = new ApiClient(baseUrl, secret)
		return ApiClient._instance
	}

	static resetForTests(): void {
		ApiClient._instance = null
	}
}
