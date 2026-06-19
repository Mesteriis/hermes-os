import { ApiClient } from '../../../platform/api/ApiClient'
import type {
	ReviewItem,
	ReviewItemsResponse,
	ReviewItemPromotionRequest
} from '../types/review'

export async function fetchReviewItems(params: { status?: string; limit?: number } = {}): Promise<ReviewItemsResponse> {
	const search = new URLSearchParams()
	if (params.status) {
		search.set('status', params.status)
	}
	if (params.limit) {
		search.set('limit', String(Math.trunc(params.limit)))
	}
	return ApiClient.instance.get<ReviewItemsResponse>(
		`/api/v1/review/items?${search.toString()}`,
		'Review items request failed'
	)
}

export async function approveReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/approve`,
		{},
		'Review item approve request failed'
	)
}

export async function dismissReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/dismiss`,
		{},
		'Review item dismiss request failed'
	)
}

export async function takeReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/take`,
		{},
		'Review item take request failed'
	)
}

export async function archiveReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/archive`,
		{},
		'Review item archive request failed'
	)
}

export async function promoteReviewItem(
	reviewItemId: string,
	payload: ReviewItemPromotionRequest
): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/promote`,
		payload,
		'Review item promote request failed'
	)
}
