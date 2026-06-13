import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClient } from '../client';
import {
	assignIdentityTrace,
	createIdentityTrace,
	fetchPersonDossier,
	fetchIdentityTraces,
	fetchOwnerPersona,
	fetchPersonaReadModel,
	fetchPersonaReadModels,
	updatePersonaReadModel
} from './persons';

const persona = {
	persona_id: 'person:v1:email:alex@example.com',
	persona_type: 'human',
	is_self: true,
	identity: {
		display_name: 'alex@example.com',
		email_address: 'alex@example.com'
	},
	communication: {
		primary_email: 'alex@example.com'
	},
	compatibility: {
		legacy_person_id: 'person:v1:email:alex@example.com',
		legacy_route: '/api/v1/persons'
	},
	created_at: '2026-06-13T09:00:00Z',
	updated_at: '2026-06-13T09:00:00Z'
};

const identityTrace = {
	id: 'identity:v1:message-participant:alex',
	person_id: null,
	identity_type: 'message_participant',
	identity_value: 'message:v1:inbox:alex',
	source: 'communication_projection',
	confidence: 0.74,
	last_verified_at: null,
	status: 'active',
	metadata: {},
	created_at: '2026-06-13T09:00:00Z',
	updated_at: '2026-06-13T09:00:00Z'
};

const ownerPersona = {
	person_id: 'person:v1:email:owner@example.com',
	display_name: 'owner@example.com',
	email_address: 'owner@example.com',
	persona_type: 'human',
	is_self: true,
	created_at: '2026-06-13T09:00:00Z',
	updated_at: '2026-06-13T09:00:00Z'
};

const personDossier = {
	person: {
		person_id: 'person:v1:email:alex@example.com',
		display_name: 'alex@example.com',
		email_address: 'alex@example.com'
	},
	summary: 'Tone: concise | Topics: local-first systems',
	interests: [{ label: 'interest', value: 'local-first systems', source_refs: ['message:1'], confidence: 0.9 }],
	projects: [],
	organizations: [],
	skills: [{ label: 'systems', value: 'Rust backend design', source_refs: ['document:1'], confidence: 0.8 }],
	communication_patterns: [],
	ai_observations: [],
	source_refs: ['message:1', 'document:1'],
	generated_at: '2026-06-13T09:00:00Z'
};

describe('persons API identity trace endpoints', () => {
	beforeEach(() => {
		ApiClient.init('http://127.0.0.1:8080', 'local-secret');
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string, init?: RequestInit) => {
				if (url.includes('/api/v1/persons/owner')) {
					return new Response(JSON.stringify({ owner_persona: ownerPersona }), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				if (url.includes('/api/v1/persons/') && url.includes('/dossier')) {
					return new Response(JSON.stringify(personDossier), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				if (url.includes('/api/v1/personas/') && init?.method === 'PUT') {
					return new Response(
						JSON.stringify({
							...persona,
							identity: {
								...persona.identity,
								display_name: 'Owner Persona'
							},
							is_self: true
						}),
						{
							status: 200,
							headers: { 'Content-Type': 'application/json' }
						}
					);
				}

				if (url.includes('/api/v1/personas/')) {
					return new Response(JSON.stringify(persona), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				if (url.includes('/api/v1/personas?')) {
					return new Response(JSON.stringify({ items: [persona] }), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				if (init?.method === 'POST') {
					return new Response(JSON.stringify(identityTrace), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				if (init?.method === 'PUT') {
					return new Response(JSON.stringify({ ...identityTrace, person_id: 'person:v1:alex' }), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ items: [identityTrace] }), {
					status: 200,
					headers: { 'Content-Type': 'application/json' }
				});
			})
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('requests the configured Owner Persona through the compatibility route', async () => {
		const response = await fetchOwnerPersona();

		expect(response.owner_persona).toEqual(ownerPersona);
		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe('http://127.0.0.1:8080/api/v1/persons/owner');
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('requests a source-backed Persona dossier by legacy person id', async () => {
		const response = await fetchPersonDossier('person:v1:email:alex@example.com');

		expect(response).toEqual(personDossier);
		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/persons/person%3Av1%3Aemail%3Aalex%40example.com/dossier'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('requests Persona-native list data with the configured local API secret', async () => {
		const response = await fetchPersonaReadModels(25);

		expect(response.items).toEqual([persona]);
		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe('http://127.0.0.1:8080/api/v1/personas?limit=25');
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('requests Persona-native detail by persona id', async () => {
		const response = await fetchPersonaReadModel('person:v1:email:alex@example.com');

		expect(response).toEqual(persona);
		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/personas/person%3Av1%3Aemail%3Aalex%40example.com'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('updates Persona-native identity through the compatibility write bridge', async () => {
		const response = await updatePersonaReadModel('person:v1:email:alex@example.com', {
			identity: {
				display_name: 'Owner Persona'
			},
			is_self: true
		});

		expect(response.identity.display_name).toBe('Owner Persona');
		expect(response.is_self).toBe(true);
		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/personas/person%3Av1%3Aemail%3Aalex%40example.com'
		);
		expect(init?.method).toBe('PUT');
		expect(init?.headers).toEqual({
			'Content-Type': 'application/json',
			'X-Hermes-Secret': 'local-secret'
		});
		expect(JSON.parse(String(init?.body))).toEqual({
			identity: {
				display_name: 'Owner Persona'
			},
			is_self: true
		});
	});

	it('requests unattached identity traces with the configured local API secret', async () => {
		const response = await fetchIdentityTraces({ status: 'unattached', limit: 25 });

		expect(response.items).toEqual([identityTrace]);
		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe('http://127.0.0.1:8080/api/v1/identity-traces?status=unattached&limit=25');
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('creates an unattached identity trace for communication-derived evidence', async () => {
		await createIdentityTrace({
			identity_type: 'message_participant',
			identity_value: 'message:v1:inbox:alex',
			source: 'communication_projection'
		});

		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe('http://127.0.0.1:8080/api/v1/identity-traces');
		expect(init?.method).toBe('POST');
		expect(init?.headers).toEqual({
			'Content-Type': 'application/json',
			'X-Hermes-Secret': 'local-secret'
		});
		expect(JSON.parse(String(init?.body))).toEqual({
			identity_type: 'message_participant',
			identity_value: 'message:v1:inbox:alex',
			source: 'communication_projection'
		});
	});

	it('assigns an identity trace to a selected Persona', async () => {
		await assignIdentityTrace('identity trace with spaces', 'person:v1:alex');

		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/identity-traces/identity%20trace%20with%20spaces/assignment'
		);
		expect(init?.method).toBe('PUT');
		expect(init?.headers).toEqual({
			'Content-Type': 'application/json',
			'X-Hermes-Secret': 'local-secret'
		});
		expect(JSON.parse(String(init?.body))).toEqual({ person_id: 'person:v1:alex' });
	});
});
