import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('NotesPage boundary', () => {
	it('preserves notes query and fallback note state after removing the legacy NotesPage Vue layer', () => {
		const appSurfaceSource = readFileSync(new URL('../../../app/queries/useNotesViewSurface.ts', import.meta.url), 'utf8')
		const surfaceSource = readFileSync(new URL('../queries/useNotesPageSurface.ts', import.meta.url), 'utf8')
		const storeSource = readFileSync(new URL('../stores/notes.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./NotesPage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/NotesSourceFilters.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/NotesList.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/NotesInsights.vue', import.meta.url))).toBe(false)

		expect(appSurfaceSource).toContain('Notes UI removed after logic extraction. Rebuild pending new design language.')
		expect(appSurfaceSource).toContain('Notes logic is preserved')

    expect(surfaceSource).toContain('fallbackNotes')
    expect(surfaceSource).toContain('useNotesQuery')
    expect(surfaceSource).toContain('notesQuery.data.value?.items ?? fallbackNotes')
    expect(storeSource).toContain('toggleSource')
    expect(storeSource).toContain('toggleTag')
    expect(storeSource).toContain('setSearchQuery')
  })
})
