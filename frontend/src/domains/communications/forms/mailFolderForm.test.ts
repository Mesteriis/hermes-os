import { describe, expect, it } from 'vitest'
import {
	composeCommunicationFolderName,
	mailFolderDeleteDialogCopy,
	mailFolderFormDefaults,
	mailFolderParentPathOptions,
	mailFolderFormSchema,
	mailFolderFormToInput,
	mailFolderMessageCountLabel,
	splitCommunicationFolderName,
	validateCommunicationFolderParentPath
} from './mailFolderForm'
import type { CommunicationFolder } from '../types/folders'

describe('mail folder form', () => {
	it('normalizes form values into a custom folder input payload', () => {
		const values = mailFolderFormSchema.parse({
			name: '  Client Projects  ',
			description: '  Active client mail  ',
			color: '  #3b82f6  ',
			sort_order: 12.8
		})

		expect(mailFolderFormToInput(values, 'account-1')).toEqual({
			account_id: 'account-1',
			name: 'Client Projects',
			description: 'Active client mail',
			color: '#3b82f6',
			sort_order: 12
		})
	})

	it('allows global folders and clears empty optional fields', () => {
		const values = mailFolderFormSchema.parse({
			name: 'Receipts',
			description: ' ',
			color: '',
			sort_order: 0
		})

		expect(mailFolderFormToInput(values, null)).toEqual({
			account_id: null,
			name: 'Receipts',
			description: null,
			color: null,
			sort_order: 0
		})
	})

	it('rejects empty names, long descriptions and invalid colors', () => {
		const result = mailFolderFormSchema.safeParse({
			name: ' ',
			description: 'x'.repeat(501),
			color: 'blue',
			sort_order: -1
		})

		expect(result.success).toBe(false)
		if (!result.success) {
			expect(result.error.issues.map((issue) => issue.path.join('.'))).toEqual([
				'name',
				'description',
				'color',
				'sort_order'
			])
		}
	})

	it('uses existing folder values as edit defaults', () => {
		expect(mailFolderFormDefaults(folder())).toEqual({
			name: 'Clients',
			description: 'Relationship mail',
			color: '#10b981',
			sort_order: 7
		})
	})

	it('splits and composes hierarchy-aware folder names', () => {
		expect(splitCommunicationFolderName('Projects / Client A / Q1')).toEqual({
			parentPath: 'Projects / Client A',
			leafName: 'Q1'
		})
		expect(composeCommunicationFolderName(' Projects / Client A ', ' Q1 ')).toBe('Projects / Client A / Q1')
		expect(composeCommunicationFolderName('', 'Inbox')).toBe('Inbox')
	})

	it('builds parent path suggestions and rejects self-descendant parents', () => {
		const folders = [
			folder(),
			{ ...folder(), folder_id: 'mail_folder:2', name: 'Clients / Acme' },
			{ ...folder(), folder_id: 'mail_folder:3', name: 'Finance' }
		]

		expect(mailFolderParentPathOptions(folders, folders[0])).toEqual(['Finance'])
		expect(validateCommunicationFolderParentPath('Clients', folders[0])).toBe('Folder cannot be its own parent')
		expect(validateCommunicationFolderParentPath('Clients / Acme', folders[0])).toBe(
			'Folder cannot move inside one of its child paths'
		)
		expect(validateCommunicationFolderParentPath('Finance', folders[0])).toBe('')
	})

	it('builds delete confirmation copy and compact message counts', () => {
		expect(mailFolderDeleteDialogCopy(folder())).toEqual({
			title: 'Delete folder',
			message: 'Delete the folder "Clients"? This does not delete messages.',
			confirmLabel: 'Delete'
		})
		expect(mailFolderMessageCountLabel({ message_count: -10 })).toBe('0')
		expect(mailFolderMessageCountLabel({ message_count: 42 })).toBe('42')
	})
})

function folder(): CommunicationFolder {
	return {
		folder_id: 'mail_folder:1',
		account_id: 'account-1',
		name: 'Clients',
		description: 'Relationship mail',
		color: '#10b981',
		sort_order: 7,
		message_count: 2,
		created_at: '2026-06-15T00:00:00Z',
		updated_at: '2026-06-15T00:00:00Z'
	}
}
