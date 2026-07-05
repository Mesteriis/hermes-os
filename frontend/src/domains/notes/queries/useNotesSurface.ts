import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/notes/queries/useNotesPageSurface.ts'

export function useNotesSurface() {
  return createDomainSurface({
    surfaceId: 'notes',
    labelKey: 'Notes',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'notes-list',
        labelKey: 'Notes list',
        descriptionKey: 'Owner notes, filters and local note selection state.',
        icon: 'tabler:notes',
        status: 'active',
        kind: 'query',
        contract: 'useNotesPageSurface.notes'
      },
      {
        id: 'notes-editor',
        labelKey: 'Note editor',
        descriptionKey: 'Drafting surface for notes that remain owner-authored memory.',
        icon: 'tabler:edit',
        status: 'active',
        kind: 'workspace',
        contract: 'useNotesPageSurface.selectedNote'
      },
      {
        id: 'notes-context-links',
        labelKey: 'Context links',
        descriptionKey: 'Links between notes, communications, documents and memory context.',
        icon: 'tabler:link',
        status: 'facade',
        kind: 'graph'
      }
    ],
    childSurfaces: [
      {
        id: 'notes-list',
        labelKey: 'List',
        status: 'facade',
        surfacePath,
        capabilityIds: ['notes-list']
      },
      {
        id: 'notes-editor',
        labelKey: 'Editor',
        status: 'facade',
        surfacePath,
        capabilityIds: ['notes-editor']
      },
      {
        id: 'notes-context',
        labelKey: 'Context',
        status: 'facade',
        capabilityIds: ['notes-context-links']
      }
    ]
  })
}

