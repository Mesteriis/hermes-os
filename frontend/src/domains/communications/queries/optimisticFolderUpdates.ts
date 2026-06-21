import type { InfiniteData } from '@tanstack/vue-query'
import type { CommunicationFolder, CommunicationFolderListResponse, CommunicationFolderUpdate } from '../types/folders'

export function upsertFolderInFolderList(
	data: InfiniteData<CommunicationFolderListResponse> | undefined,
	queryKey: readonly unknown[],
	folder: CommunicationFolder
): InfiniteData<CommunicationFolderListResponse> | undefined {
	if (!data) return data
	if (!folderMatchesQuery(queryKey, folder)) {
		return removeFolderFromFolderList(data, folder.folder_id)
	}

	let changed = false
	const pages = data.pages.map((page, pageIndex) => {
		const existingIndex = page.items.findIndex((item) => item.folder_id === folder.folder_id)

		if (existingIndex >= 0) {
			const items = page.items.slice()
			items[existingIndex] = folder
			changed = true
			return {
				...page,
				items: sortFolders(items)
			}
		}

		if (pageIndex === 0) {
			changed = true
			return {
				...page,
				items: sortFolders([folder, ...page.items])
			}
		}

		return page
	})

	return changed ? { ...data, pages } : data
}

export function removeFolderFromFolderList(
	data: InfiniteData<CommunicationFolderListResponse> | undefined,
	folderId: string
): InfiniteData<CommunicationFolderListResponse> | undefined {
	if (!data) return data

	let changed = false
	const pages = data.pages.map((page) => {
		const items = page.items.filter((item) => item.folder_id !== folderId)
		if (items.length === page.items.length) return page
		changed = true
		return { ...page, items }
	})

	return changed ? { ...data, pages } : data
}

export function optimisticFolderFromUpdate(
	existing: CommunicationFolder,
	update: CommunicationFolderUpdate,
	updatedAt: string
): CommunicationFolder {
	return {
		...existing,
		account_id: typeof update.account_id === 'undefined' ? existing.account_id : update.account_id,
		name: update.name ?? existing.name,
		description: typeof update.description === 'undefined' ? existing.description : update.description,
		color: typeof update.color === 'undefined' ? existing.color : update.color,
		sort_order: update.sort_order ?? existing.sort_order,
		updated_at: updatedAt
	}
}

export function folderMatchesQuery(queryKey: readonly unknown[], folder: CommunicationFolder): boolean {
	const accountId = queryKey[1]
	if (typeof accountId !== 'string' || !accountId.trim()) return true
	return folder.account_id === accountId
}

function sortFolders(folders: CommunicationFolder[]): CommunicationFolder[] {
	return folders
		.slice()
		.sort((left, right) => left.sort_order - right.sort_order || left.name.localeCompare(right.name))
}
