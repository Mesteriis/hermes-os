import { describe, expect, it } from 'vitest'
import type { InfiniteData } from '@tanstack/vue-query'
import type { CommunicationFolder, CommunicationFolderListResponse } from '../types/folders'
import {
	optimisticFolderFromUpdate,
	removeFolderFromFolderList,
	upsertFolderInFolderList
} from './optimisticFolderUpdates'

function folder(overrides: Partial<CommunicationFolder> = {}): CommunicationFolder {
	return {
		folder_id: 'folder-1',
		account_id: 'account-1',
		name: 'Clients',
		description: null,
		color: null,
		sort_order: 100,
		message_count: 0,
		created_at: '2026-06-15T10:00:00Z',
		updated_at: '2026-06-15T10:00:00Z',
		...overrides
	}
}

function folderList(items: CommunicationFolder[]): InfiniteData<CommunicationFolderListResponse> {
	return {
		pages: [
			{
				items,
				next_cursor: null,
				has_more: false
			}
		],
		pageParams: [null]
	}
}

describe('optimistic folder updates', () => {
	it('upserts folders into matching folder list caches in display order', () => {
		const bravo = folder({ folder_id: 'bravo', name: 'Bravo', sort_order: 200 })
		const charlie = folder({ folder_id: 'charlie', name: 'Charlie', sort_order: 300 })
		const alpha = folder({ folder_id: 'alpha', name: 'Alpha', sort_order: 100 })

		const updated = upsertFolderInFolderList(
			folderList([bravo, charlie]),
			['communications-folders', 'account-1'],
			alpha
		)

		expect(updated?.pages[0]?.items.map((item) => item.folder_id)).toEqual([
			'alpha',
			'bravo',
			'charlie'
		])
	})

	it('does not patch account-scoped folder caches for another account', () => {
		const existing = folder({ folder_id: 'existing', account_id: 'account-2' })
		const data = folderList([existing])

		const updated = upsertFolderInFolderList(
			data,
			['communications-folders', 'account-2'],
			folder({ folder_id: 'foreign', account_id: 'account-1' })
		)

		expect(updated).toBe(data)
	})

	it('removes a cached folder when an update no longer matches the account-scoped query', () => {
		const existing = folder({ folder_id: 'folder-1', account_id: 'account-1' })
		const data = folderList([existing])

		const updated = upsertFolderInFolderList(
			data,
			['communications-folders', 'account-1'],
			{ ...existing, account_id: 'account-2' }
		)

		expect(updated?.pages[0]?.items).toEqual([])
	})

	it('removes folders from cached folder lists without touching unchanged caches', () => {
		const first = folder({ folder_id: 'folder-1' })
		const second = folder({ folder_id: 'folder-2' })
		const data = folderList([first, second])

		const removed = removeFolderFromFolderList(data, 'folder-1')
		const unchanged = removeFolderFromFolderList(data, 'missing')

		expect(removed?.pages[0]?.items).toEqual([second])
		expect(unchanged).toBe(data)
	})

	it('builds optimistic folder updates from cached rows and partial updates', () => {
		const existing = folder({ folder_id: 'folder-1', name: 'Old', color: 'blue' })

		expect(optimisticFolderFromUpdate(existing, {
			account_id: null,
			name: 'New',
			color: null
		}, '2026-06-15T11:00:00Z')).toMatchObject({
			folder_id: 'folder-1',
			account_id: null,
			name: 'New',
			color: null,
			updated_at: '2026-06-15T11:00:00Z'
		})
	})
})
