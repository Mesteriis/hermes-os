import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { CommunicationFolder, CommunicationFolderInput } from '../types/folders'

const HEX_COLOR_PATTERN = /^#[0-9a-fA-F]{6}$/

export type CommunicationFolderFormValues = z.infer<typeof mailFolderFormSchema>
export type CommunicationFolderDeleteDialogCopy = {
	title: string
	message: string
	confirmLabel: string
}
export type CommunicationFolderNameParts = {
	parentPath: string
	leafName: string
}

export const mailFolderFormSchema = z.object({
	name: z.string().trim().min(1, 'Name is required').max(120, 'Name is too long'),
	description: z.string().trim().max(500, 'Description is too long'),
	color: z
		.string()
		.trim()
		.refine((value) => value === '' || HEX_COLOR_PATTERN.test(value), {
			message: 'Color must be a hex color'
		}),
	sort_order: z.coerce
		.number()
		.min(0, 'Sort order cannot be negative')
		.transform((value) => Math.trunc(value))
})

export const mailFolderVeeValidationSchema = toTypedSchema(mailFolderFormSchema)

export function mailFolderFormDefaults(folder?: CommunicationFolder | null): CommunicationFolderFormValues {
	return {
		name: folder?.name ?? '',
		description: folder?.description ?? '',
		color: folder?.color ?? '',
		sort_order: folder?.sort_order ?? 0
	}
}

export function splitCommunicationFolderName(name: string): CommunicationFolderNameParts {
	const parts = normalizeCommunicationFolderPathParts(name)
	if (parts.length === 0) {
		return {
			parentPath: '',
			leafName: ''
		}
	}

	return {
		parentPath: parts.slice(0, -1).join(' / '),
		leafName: parts[parts.length - 1] ?? ''
	}
}

export function composeCommunicationFolderName(parentPath: string, leafName: string): string {
	const parts = [
		...normalizeCommunicationFolderPathParts(parentPath),
		...normalizeCommunicationFolderPathParts(leafName)
	]
	return parts.join(' / ')
}

export function mailFolderParentPathOptions(
	folders: ReadonlyArray<Pick<CommunicationFolder, 'folder_id' | 'name'>>,
	editingFolder?: Pick<CommunicationFolder, 'folder_id' | 'name'> | null
): string[] {
	const currentPath = normalizeCommunicationFolderPath(editingFolder?.name ?? '')
	const unique = new Set<string>()
	const options: string[] = []

	for (const folder of folders) {
		const normalizedPath = normalizeCommunicationFolderPath(folder.name)
		if (!normalizedPath) continue
		if (currentPath && isSameOrDescendantPath(normalizedPath, currentPath)) continue
		if (unique.has(normalizedPath)) continue
		unique.add(normalizedPath)
		options.push(normalizedPath)
	}

	return options
}

export function validateCommunicationFolderParentPath(
	parentPath: string,
	editingFolder?: Pick<CommunicationFolder, 'name'> | null
): string {
	const normalizedParentPath = normalizeCommunicationFolderPath(parentPath)
	const currentPath = normalizeCommunicationFolderPath(editingFolder?.name ?? '')
	if (!normalizedParentPath || !currentPath) return ''
	if (normalizedParentPath === currentPath) return 'Folder cannot be its own parent'
	if (isSameOrDescendantPath(normalizedParentPath, currentPath)) {
		return 'Folder cannot move inside one of its child paths'
	}
	return ''
}

export function mailFolderFormToInput(
	values: CommunicationFolderFormValues,
	accountId: string | null
): CommunicationFolderInput {
	const parsed = mailFolderFormSchema.parse(values)
	return {
		account_id: accountId?.trim() || null,
		name: parsed.name,
		description: parsed.description || null,
		color: parsed.color || null,
		sort_order: parsed.sort_order
	}
}

export function mailFolderDeleteDialogCopy(
	folder: Pick<CommunicationFolder, 'name'>
): CommunicationFolderDeleteDialogCopy {
	return {
		title: 'Delete folder',
		message: `Delete the folder "${folder.name}"? This does not delete messages.`,
		confirmLabel: 'Delete'
	}
}

export function mailFolderMessageCountLabel(
	folder: Pick<CommunicationFolder, 'message_count'>
): string {
	return String(Math.max(0, Math.trunc(folder.message_count)))
}

function normalizeCommunicationFolderPath(value: string): string {
	return normalizeCommunicationFolderPathParts(value).join(' / ')
}

function normalizeCommunicationFolderPathParts(value: string): string[] {
	return value
		.split('/')
		.map((part) => part.trim())
		.filter(Boolean)
}

function isSameOrDescendantPath(candidatePath: string, currentPath: string): boolean {
	if (!candidatePath || !currentPath) return false
	if (candidatePath === currentPath) return true
	return candidatePath.startsWith(`${currentPath} / `)
}
