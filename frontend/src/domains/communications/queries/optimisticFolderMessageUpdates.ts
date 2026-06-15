import type { InfiniteData } from '@tanstack/vue-query'
import type { FolderMessage, FolderMessageListResponse } from '../types/folders'

export type FolderMessageListCache = Array<[
	readonly unknown[],
	InfiniteData<FolderMessageListResponse> | undefined
]>

export function upsertFolderMessageInFolderList(
	data: InfiniteData<FolderMessageListResponse> | undefined,
	queryKey: readonly unknown[],
	folderMessage: FolderMessage
): InfiniteData<FolderMessageListResponse> | undefined {
	if (!data || !folderMessageMatchesQuery(queryKey, folderMessage)) return data

	let changed = false
	const pages = data.pages.map((page, pageIndex) => {
		const existingIndex = page.items.findIndex((item) => item.message_id === folderMessage.message_id)

		if (existingIndex >= 0) {
			const items = page.items.slice()
			items[existingIndex] = folderMessage
			changed = true
			return {
				...page,
				items: sortFolderMessages(items)
			}
		}

		if (pageIndex === 0) {
			changed = true
			return {
				...page,
				items: sortFolderMessages([folderMessage, ...page.items])
			}
		}

		return page
	})

	return changed ? { ...data, pages } : data
}

export function removeFolderMessageFromFolderList(
	data: InfiniteData<FolderMessageListResponse> | undefined,
	messageId: string
): InfiniteData<FolderMessageListResponse> | undefined {
	if (!data) return data

	let changed = false
	const pages = data.pages.map((page) => {
		const items = page.items.filter((item) => item.message_id !== messageId)
		if (items.length === page.items.length) return page
		changed = true
		return { ...page, items }
	})

	return changed ? { ...data, pages } : data
}

export function findCachedFolderMessage(
	folderMessageLists: FolderMessageListCache,
	messageId: string
): FolderMessage | undefined {
	for (const [, data] of folderMessageLists) {
		for (const page of data?.pages ?? []) {
			const folderMessage = page.items.find((item) => item.message_id === messageId)
			if (folderMessage) return folderMessage
		}
	}
	return undefined
}

export function optimisticFolderMessageForTarget(
	source: FolderMessage,
	folderId: string,
	addedAt: string
): FolderMessage {
	return {
		...source,
		folder_id: folderId,
		added_at: addedAt
	}
}

export function folderMessageMatchesQuery(
	queryKey: readonly unknown[],
	folderMessage: FolderMessage
): boolean {
	const folderId = queryKey[1]
	return typeof folderId !== 'string' || folderId === folderMessage.folder_id
}

function sortFolderMessages(items: FolderMessage[]): FolderMessage[] {
	return items
		.slice()
		.sort((left, right) => {
			const addedAt = Date.parse(right.added_at) - Date.parse(left.added_at)
			if (Number.isFinite(addedAt) && addedAt !== 0) return addedAt
			return left.message_id.localeCompare(right.message_id)
		})
}
