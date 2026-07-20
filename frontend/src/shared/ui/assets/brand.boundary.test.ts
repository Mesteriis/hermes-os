import { describe, expect, it } from 'vitest'
import { hermesBrandAssets, hermesShellBackgroundAssetPaths } from './brand'

describe('Hermes UI local asset inventory', () => {
	it('keeps shell assets compiled and fixed to local public paths', () => {
		expect(hermesBrandAssets.logoMarkDark).toBe('/assets/hermes-logo-mark-dark.png')
		expect(hermesBrandAssets.logoMarkLight).toBe('/assets/hermes-logo-mark-light.png')
		expect(hermesShellBackgroundAssetPaths).toHaveLength(10)
		for (const assetPath of hermesShellBackgroundAssetPaths) {
			expect(assetPath).toMatch(/^\/assets\/shell-backgrounds\/[a-z-]+\.webp$/)
		}
	})
})
