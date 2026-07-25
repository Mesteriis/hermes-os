import { readFileSync, readdirSync } from 'node:fs'
import { relative } from 'node:path'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

const SOURCE_ROOT = fileURLToPath(new URL('../../../', import.meta.url))
const LEGACY_ROUTE_PREFIX = ['/api/v1', 'communications'].join('/')
const EXACT_PENDING_FILES = [
	'domains/communications/api/attachmentImportApi.ts',
	'integrations/telegram/api/telegramBusiness.ts',
	'integrations/whatsapp/api/whatsappBusiness.ts',
] as const

describe('legacy Communications REST inventory', () => {
	it('keeps the secondary frontend cutover exact and bounded', () => {
		const pendingFiles = collectSourceFiles(SOURCE_ROOT)
			.filter((path) => !path.endsWith('.test.ts'))
			.filter((path) => readFileSync(path, 'utf8').includes(LEGACY_ROUTE_PREFIX))
			.map((path) => relative(SOURCE_ROOT, path).replaceAll('\\', '/'))
			.sort()

		expect(pendingFiles).toEqual(EXACT_PENDING_FILES)
	})
})

function collectSourceFiles(directory: string): string[] {
	return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
		const path = `${directory}/${entry.name}`
		if (entry.isDirectory()) return collectSourceFiles(path)
		if (!entry.isFile() || !/\.(?:ts|vue)$/.test(entry.name)) return []
		return [path]
	})
}
