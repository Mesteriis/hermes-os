import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('SystemControlPage bootstrap boundary', () => {
	it('renders typed module composition without claiming unavailable platform health', () => {
		const source = readFileSync(new URL('./SystemControlPage.vue', import.meta.url), 'utf8')

		expect(source).toContain('systemControlModuleRows')
		expect(source).toContain('Module Control Plane')
		expect(source).toContain('module.applyState')
		expect(source).toContain('module.reasonCode')
		expect(source).toContain('Developer mode')
		expect(source).toContain('Settings registry')
		expect(source).toContain('Public module settings')
		expect(source).toContain('publicModuleSettingRows')
		expect(source).toContain('Interface language')
		expect(source).toContain("selectedSection === 'scheduler'")
		expect(source).toContain('Scheduler runtime status')
		expect(source).toContain("selectedSection === 'events'")
		expect(source).toContain('Events runtime status')
		expect(source).toContain('systemControlComponentRows')
		expect(source).toContain("emit('languageChange', value)")
		expect(source).not.toContain('settings-status-strip')
		expect(source).not.toContain('settings-note-panel')
		expect(source).not.toContain('Only signed, local client surfaces')
		expect(source).not.toContain('Recovery shell is independent')
		expect(source).not.toMatch(/fetch\(|useQuery|ApiClient|ClientSystemStatusService/)
	})
})
