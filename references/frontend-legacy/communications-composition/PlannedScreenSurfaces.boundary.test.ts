// Historical pre-clean-room planned-screen inventory. Not part of the active test suite.
import { existsSync, readdirSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

const plannedSurfaces = [
	'useAgentsViewSurface',
	'useCalendarViewSurface',
	'useCommunicationsViewSurface',
	'useDocumentsViewSurface',
	'useEventTracingViewSurface',
	'useHomeViewSurface',
	'useKnowledgeViewSurface',
	'useNotesViewSurface',
	'useOrganizationsViewSurface',
	'useProjectsViewSurface',
	'useReviewViewSurface',
	'useSettingsViewSurface',
	'useTasksViewSurface',
	'useTimelineViewSurface'
] as const

describe('planned app screen surfaces', () => {
	it('keeps planned screen logic in app-level surfaces without legacy route pages', () => {
		const viewsDirectory = new URL('../views', import.meta.url)
		const legacyRoutePages = existsSync(viewsDirectory)
			? readdirSync(viewsDirectory).filter((entry) => entry.endsWith('.vue'))
			: []

		expect(legacyRoutePages).toEqual([])

		for (const surfaceName of plannedSurfaces) {
			const surfaceFile = `./${surfaceName}.ts`
			const surfaceSource = readFileSync(new URL(surfaceFile, import.meta.url), 'utf8')

			expect(surfaceSource).toContain(`export function ${surfaceName}()`)
			expect(surfaceSource).toContain('createPlannedScreenSurface')
			expect(surfaceSource).toContain('screenId:')
			expect(surfaceSource).toContain('status:')
		}
	})

	it('keeps Personas routed to the rebuilt personas domain surface', () => {
		const surfaceSource = readFileSync(
			new URL('./usePersonasViewSurface.ts', import.meta.url),
			'utf8'
		)

		expect(surfaceSource).toContain('usePersonasSurface')
		expect(surfaceSource).not.toContain('createPlannedScreenSurface')
	})
})
