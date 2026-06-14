export type ApiError = {
	message: string
	status?: number
	code?: string
}

export type PaginatedResponse<T> = {
	data: T[]
	total: number
	offset: number
	limit: number
}
