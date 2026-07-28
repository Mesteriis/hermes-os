import { describe, expect, it } from 'vitest'

import { decodeCanonicalCommunicationContent } from './canonicalCommunicationContentModel'

describe('canonical Communications content model', () => {
	it('decodes exact UTF-8 without interpreting markup', () => {
		const content = new TextEncoder().encode('<script>alert(1)</script>\nPlain text')
		expect(decodeCanonicalCommunicationContent(content)).toBe(
			'<script>alert(1)</script>\nPlain text',
		)
	})

	it('rejects invalid UTF-8 instead of performing lossy replacement', () => {
		expect(() => decodeCanonicalCommunicationContent(
			new Uint8Array([0xc3, 0x28]),
		)).toThrow('not valid UTF-8')
	})
})
