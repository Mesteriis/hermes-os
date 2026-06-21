import { describe, expect, it } from 'vitest'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const root = new URL('../..', import.meta.url)

function filesUnder(path: string): string[] {
	const dir = new URL(path, root)
	const output: string[] = []
	for (const entry of readdirSync(dir)) {
		const fullPath = join(dir.pathname, entry)
		const stat = statSync(fullPath)
		if (stat.isDirectory()) {
			output.push(...filesUnder(relative(root.pathname, fullPath)))
		} else if (/\.(ts|vue)$/.test(entry)) {
			output.push(fullPath)
		}
	}
	return output
}

function readAll(paths: string[]): string {
	return paths.map((path) => readFileSync(path, 'utf8')).join('\n')
}

describe('business communication hooks ownership boundary', () => {
	it('keeps shared communication modules DTO-only', () => {
		const source = readAll(filesUnder('shared/communications'))

		expect(source).not.toMatch(/\/api\/v1\/communications/)
		expect(source).not.toMatch(/\buseQuery\b/)
		expect(source).not.toMatch(/\bqueryKey\b/)
		expect(source).not.toMatch(/\[\s*['"]communications['"]/)
		expect(source).not.toMatch(/\bfetch\(/)
	})

	it('keeps integration modules out of Communications business read models', () => {
		const source = readAll(filesUnder('integrations'))

		expect(source).not.toMatch(/shared\/communications\/.*Business/)
		expect(source).not.toMatch(/\[\s*['"]communications['"]/)
		expect(source).not.toMatch(/\/api\/v1\/communications\/(conversations|messages|search|topics)/)
		expect(source).not.toMatch(/MessageThread|ChatList|MediaGallery|RawEvidence|ReplyChain|ForwardChain|Reactions|Topics/)
	})

	it('keeps Communications domain out of provider-control endpoints', () => {
		const source = readAll(filesUnder('domains/communications'))

		expect(source).not.toMatch(/\/api\/v1\/integrations\/(telegram|whatsapp|mail)/)
	})
})
