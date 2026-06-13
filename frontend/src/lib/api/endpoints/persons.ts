import { ApiClient } from '../client';
import type {
	IdentityTraceAssignmentRequest,
	IdentityTraceListStatus,
	NewIdentityTraceRequest,
	OwnerPersonaResponse,
	PersonDossier,
	PersonIdentity,
	PersonIdentityCandidateListResponse,
	PersonIdentityReviewState,
	PersonIdentityTraceListResponse,
	PersonaListResponse,
	PersonaReadModel,
	PersonaUpdateRequest,
	PersonListResponse,
	EnrichedPerson
} from '../types';

export type IdentityTraceListParams = {
	status?: IdentityTraceListStatus;
	limit?: number;
};

export async function fetchIdentityCandidates(limit = 50): Promise<PersonIdentityCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<PersonIdentityCandidateListResponse>(
		`/api/v1/identity-candidates?${params.toString()}`,
		'Identity candidate request failed'
	);
}

export async function reviewIdentityCandidate(
	identityCandidateId: string,
	reviewState: PersonIdentityReviewState,
	commandId = `person-identity-review-${crypto.randomUUID()}`
) {
	return ApiClient.instance.put(
		`/api/v1/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
		{
			command_id: commandId,
			review_state: reviewState
		}
	);
}

export async function fetchOwnerPersona(): Promise<OwnerPersonaResponse> {
	return ApiClient.instance.get<OwnerPersonaResponse>(
		'/api/v1/persons/owner',
		'Owner Persona request failed'
	);
}

export async function fetchPersonaReadModels(limit = 50): Promise<PersonaListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<PersonaListResponse>(
		`/api/v1/personas?${params.toString()}`,
		'Personas request failed'
	);
}

export async function fetchPersonaReadModel(personaId: string): Promise<PersonaReadModel> {
	return ApiClient.instance.get<PersonaReadModel>(
		`/api/v1/personas/${encodeURIComponent(personaId)}`,
		'Persona request failed'
	);
}

export async function updatePersonaReadModel(
	personaId: string,
	request: PersonaUpdateRequest
): Promise<PersonaReadModel> {
	return ApiClient.instance.put<PersonaReadModel>(
		`/api/v1/personas/${encodeURIComponent(personaId)}`,
		request,
		'Persona update request failed'
	);
}

export async function fetchIdentityTraces(
	params: IdentityTraceListParams = {}
): Promise<PersonIdentityTraceListResponse> {
	const query = new URLSearchParams({
		status: params.status ?? 'unattached',
		limit: String(Math.trunc(params.limit ?? 50))
	});
	return ApiClient.instance.get<PersonIdentityTraceListResponse>(
		`/api/v1/identity-traces?${query.toString()}`,
		'Identity trace request failed'
	);
}

export async function createIdentityTrace(
	request: NewIdentityTraceRequest
): Promise<PersonIdentity> {
	return ApiClient.instance.post<PersonIdentity>(
		'/api/v1/identity-traces',
		request,
		'Identity trace create request failed'
	);
}

export async function assignIdentityTrace(
	identityTraceId: string,
	personId: string
): Promise<PersonIdentity> {
	const request: IdentityTraceAssignmentRequest = { person_id: personId };
	return ApiClient.instance.put<PersonIdentity>(
		`/api/v1/identity-traces/${encodeURIComponent(identityTraceId)}/assignment`,
		request,
		'Identity trace assignment request failed'
	);
}

export async function fetchPersons(limit = 50, favoritesOnly = false): Promise<PersonListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (favoritesOnly) params.set('favorites_only', 'true');
	return ApiClient.instance.get<PersonListResponse>(
		`/api/v1/persons?${params.toString()}`,
		'Persons request failed'
	);
}

export async function fetchPerson(personId: string): Promise<EnrichedPerson> {
	return ApiClient.instance.get<EnrichedPerson>(
		`/api/v1/persons/${encodeURIComponent(personId)}`,
		'Person request failed'
	);
}

export async function fetchPersonDossier(personId: string): Promise<PersonDossier> {
	return ApiClient.instance.get<PersonDossier>(
		`/api/v1/persons/${encodeURIComponent(personId)}/dossier`,
		'Person dossier request failed'
	);
}
