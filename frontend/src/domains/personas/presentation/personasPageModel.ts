import type {
	EnrichedPersona,
	PersonaDirectoryFilter,
	PersonaIdentity,
	PersonaIdentityCandidate,
	PersonaIdentityReviewState,
	PersonaPanelProfile,
	PersonaWorkspaceSection,
	Relationship,
} from '../types/persona'

/**
 * The compiled Personas page consumes this complete, owner-scoped view model.
 * It deliberately contains no transport, cache, or legacy store details.
 */
export type PersonasPageModel = {
	ownerPersona: PersonaPanelProfile | null
	selectedPersona: PersonaPanelProfile | null
	filteredPersonas: readonly EnrichedPersona[]
	directoryCount: number
	pendingReviewCount: number
	directoryFilter: PersonaDirectoryFilter
	activeSection: PersonaWorkspaceSection
	searchQuery: string
	suggestedIdentityCandidates: readonly PersonaIdentityCandidate[]
	identityTraces: readonly PersonaIdentity[]
	selectedPersonaRelationships: readonly Relationship[]
	isLoading: boolean
	isRefreshing: boolean
	actionError: string
	settingOwnerPersonaId: string | null
	reviewingCandidateId: string | null
	assigningTraceId: string | null
}

export type PersonasPageActions = {
	refresh(): void | Promise<void>
	selectPersona(index: number): void
	setOwner(persona: EnrichedPersona): void | Promise<void>
	reviewCandidate(candidate: PersonaIdentityCandidate, state: PersonaIdentityReviewState): void | Promise<void>
	assignTraceToOwner(trace: PersonaIdentity): void | Promise<void>
	assignTraceToSelectedPersona(trace: PersonaIdentity): void | Promise<void>
	toggleAddressBook(persona: PersonaPanelProfile, value: boolean): void | Promise<void>
	setDirectoryFilter(value: PersonaDirectoryFilter): void
	setActiveSection(value: PersonaWorkspaceSection): void
	setSearchQuery(value: string): void
}
