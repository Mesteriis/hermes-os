export type ApiError = {
	message?: string;
	status?: number;
};

export class ApiClient {
	readonly baseUrl: string;

	constructor(baseUrl: string, private readonly secret: string) {
		this.baseUrl = baseUrl.replace(/\/+$/, '');
	}

	async get<T>(path: string, fallbackMessage = 'GET request failed'): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			headers: { 'X-Hermes-Secret': this.secret }
		});

		if (!response.ok) {
			const error = (await response.json().catch(() => null)) as ApiError | null;
			throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
		}

		return response.json() as T;
	}

	async post<T>(path: string, body: unknown, fallbackMessage = 'POST request failed'): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				'X-Hermes-Secret': this.secret
			},
			body: JSON.stringify(body)
		});

		if (!response.ok) {
			const error = (await response.json().catch(() => null)) as ApiError | null;
			throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
		}

		return response.json() as T;
	}

	async put<T>(path: string, body: unknown, fallbackMessage = 'PUT request failed'): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json',
				'X-Hermes-Secret': this.secret
			},
			body: JSON.stringify(body)
		});

		if (!response.ok) {
			const error = (await response.json().catch(() => null)) as ApiError | null;
			throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
		}

		return response.json() as T;
	}

	async delete<T>(path: string, fallbackMessage = 'DELETE request failed'): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: 'DELETE',
			headers: { 'X-Hermes-Secret': this.secret }
		});

		if (!response.ok) {
			const error = (await response.json().catch(() => null)) as ApiError | null;
			throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
		}

		return response.json() as T;
	}

	private static _instance: ApiClient | null = null;

	static get instance(): ApiClient {
		if (!ApiClient._instance) {
			throw new Error(
				'ApiClient not initialized. Call ApiClient.init(baseUrl, secret) first.'
			);
		}
		return ApiClient._instance;
	}

	static init(baseUrl: string, secret: string): ApiClient {
		ApiClient._instance = new ApiClient(baseUrl, secret);
		return ApiClient._instance;
	}
}
