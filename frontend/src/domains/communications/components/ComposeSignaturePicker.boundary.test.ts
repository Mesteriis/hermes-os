import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentPath = resolve(dirname(fileURLToPath(import.meta.url)), 'ComposeSignaturePicker.vue')

describe('ComposeSignaturePicker boundaries', () => {
	it('uses the personas query hook and emits selected signatures', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).not.toContain('../api/communications')
		expect(source).toContain('usePersonasQuery')
		expect(source).toContain("emit('apply'")
		expect(source).toContain('persona.signature')
	})
})
