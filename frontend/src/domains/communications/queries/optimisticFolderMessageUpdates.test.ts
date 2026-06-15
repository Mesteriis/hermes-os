import { describe, expect, it } from 'vitest'
import type { InfiniteData } from '@tanstack/vue-query'
import type { FolderMessage, FolderMessageListResponse } from '../types/folders'
import {
	findCachedFolderMessage,
	optimisticFolderMessageForTarget,
	removeFolderMessageFromFolderList,
	upsertFolderMessageInFolderList
} from './optimisticFolderMessageUpdates'

function folderMessage(overrides: Partial<FolderMessage> = {}): FolderMessage {
	return {
		folder_id: 'folder-1',
		message_id: 'message-1',
		account_id: 'account-1',
		subject: 'Status',
		sender: 'sender@example.com',
		occurred_at: '2026-06-15T10:00:00Z',
		projected_at: '2026-06-15T10:01:00Z',
		workflow_state: 'new',
		local_state: 'active',
		added_at: '2026-06-15T10:02:00Z',
		attachment_count: 0,
		...overrides
	}
}

function folderMessageList(items: FolderMessage[]): InfiniteData<FolderMessageListResponse> {
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

describe('optimistic folder message updates', () => {
	it('upserts folder messages into matching folder caches in newest-first order', () => {
		const older = folderMessage({
			folder_id: 'folder-2',
			message_id: 'older',
			added_at: '2026-06-15T10:00:00Z'
		})
		const newer = folderMessage({
			folder_id: 'folder-2',
			message_id: 'newer',
			added_at: '2026-06-15T11:00:00Z'
		})

		const updated = upsertFolderMessageInFolderList(
			folderMessageList([older]),
			['communications-folder-messages', 'folder-2'],
			newer
		)

		expect(updated?.pages[0]?.items.map((item) => item.message_id)).toEqual([
			'newer',
			'older'
		])
	})

	it('does not upsert folder messages into unrelated folder caches', () => {
		const data = folderMessageList([folderMessage({ folder_id: 'folder-1' })])

		const updated = upsertFolderMessageInFolderList(
			data,
			['communications-folder-messages', 'folder-1'],
			folderMessage({ folder_id: 'folder-2' })
		)

		expect(updated).toBe(data)
	})

	it('removes moved messages from cached source folder lists', () => {
		const first = folderMessage({ message_id: 'message-1' })
		const second = folderMessage({ message_id: 'message-2' })
		const data = folderMessageList([first, second])

		const removed = removeFolderMessageFromFolderList(data, 'message-1')
		const unchanged = removeFolderMessageFromFolderList(data, 'missing')

		expect(removed?.pages[0]?.items).toEqual([second])
		expect(unchanged).toBe(data)
	})

	it('finds cached folder messages and builds target-folder optimistic rows', () => {
		const source = folderMessage({ folder_id: 'folder-1', message_id: 'message-1' })
		const lists: Array<[readonly unknown[], InfiniteData<FolderMessageListResponse> | undefined]> = [
			[['communications-folder-messages', 'folder-1'], folderMessageList([source])]
		]

		const found = findCachedFolderMessage(lists, 'message-1')
		const target = optimisticFolderMessageForTarget(source, 'folder-2', '2026-06-15T12:00:00Z')

		expect(found).toEqual(source)
		expect(target).toMatchObject({
			folder_id: 'folder-2',
			message_id: 'message-1',
			added_at: '2026-06-15T12:00:00Z'
		})
	})
})
