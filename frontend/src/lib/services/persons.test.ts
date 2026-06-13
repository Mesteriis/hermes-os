import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	assignIdentityTrace: vi.fn(),
	fetchIdentityTraces: vi.fn(),
	fetchIdentityCandidates: vi.fn(),
	fetchPersonDossier: vi.fn(),
	fetchOrganizations: vi.fn(),
	fetchPersons: vi.fn(),
	reviewIdentityCandidate: vi.fn()
}));

import { assignIdentityTrace, fetchIdentityTraces, fetchPersonDossier } from '$lib/api';
import type { PersonDossier, PersonIdentity } from '$lib/api';
import {
	assignIdentityTraceToPersona,
	dossierSectionPreview,
	formatIdentityTraceKind,
	formatIdentityTraceValue,
	loadIdentityTraces,
	loadPersonDossier
} from './persons';

const identityTrace: PersonIdentity = {
	id: 'identity:v1:message-participant:alex',
	person_id: null,
	identity_type: 'message_participant',
	identity_value: 'message:v1:inbox:alex@example.com',
	source: 'communication_projection',
	confidence: 0.74,
	last_verified_at: null,
	status: 'active',
	metadata: {},
	created_at: '2026-06-13T09:00:00Z',
	updated_at: '2026-06-13T09:00:00Z'
};

const personDossier: PersonDossier = {
	person: {
		person_id: 'person:v1:email:alex@example.com',
		display_name: 'alex@example.com',
		email_address: 'alex@example.com'
	},
	summary: 'Tone: concise | Topics: local-first systems',
	interests: [
		{ label: 'interest', value: 'local-first systems', source_refs: ['message:1'], confidence: 0.9 }
	],
	projects: [],
	organizations: [],
	skills: [
		{ label: 'systems', value: 'Rust backend design', source_refs: ['document:1'], confidence: 0.8 }
	],
	communication_patterns: [],
	ai_observations: [],
	source_refs: ['message:1', 'document:1'],
	generated_at: '2026-06-13T09:00:00Z'
};

describe('Persona identity trace service', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads unattached identity traces for review', async () => {
		vi.mocked(fetchIdentityTraces).mockResolvedValue({ items: [identityTrace] });

		const result = await loadIdentityTraces();

		expect(fetchIdentityTraces).toHaveBeenCalledWith({ status: 'unattached', limit: 50 });
		expect(result.error).toBe('');
		expect(result.identityTraces).toEqual([identityTrace]);
		expect(result.pendingCount).toBe(1);
	});

	it('loads a source-backed Persona dossier for the selected Persona', async () => {
		vi.mocked(fetchPersonDossier).mockResolvedValue(personDossier);

		const result = await loadPersonDossier('person:v1:email:alex@example.com');

		expect(fetchPersonDossier).toHaveBeenCalledWith('person:v1:email:alex@example.com');
		expect(result.error).toBe('');
		expect(result.dossier).toEqual(personDossier);
	});

	it('assigns a trace to a Persona through the backend API', async () => {
		vi.mocked(assignIdentityTrace).mockResolvedValue({
			...identityTrace,
			person_id: 'person:v1:alex'
		});

		const result = await assignIdentityTraceToPersona(identityTrace, 'person:v1:alex');

		expect(assignIdentityTrace).toHaveBeenCalledWith(identityTrace.id, 'person:v1:alex');
		expect(result.error).toBe('');
	});

	it('returns a validation error when assignment has no target Persona', async () => {
		const result = await assignIdentityTraceToPersona(identityTrace, '');

		expect(assignIdentityTrace).not.toHaveBeenCalled();
		expect(result.error).toBe('Select a Persona before assigning this identity trace.');
	});

	it('formats trace kind and compact value labels for review rows', () => {
		expect(formatIdentityTraceKind('message_participant')).toBe('Message Participant');
		expect(formatIdentityTraceValue(identityTrace)).toBe('message:v1:inbox:alex@example.com');
	});

	it('builds compact dossier section previews from generated sections', () => {
		expect(dossierSectionPreview(personDossier)).toEqual([
			'local-first systems',
			'Rust backend design'
		]);
	});
});
