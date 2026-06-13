import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	fetchAiAgents: vi.fn(),
	fetchAiRuns: vi.fn(),
	fetchAiStatus: vi.fn(),
	fetchOwnerPersona: vi.fn(),
	refreshAiTaskCandidates: vi.fn(),
	requestAiAnswer: vi.fn(),
	requestAiMeetingPrep: vi.fn()
}));

import { fetchAiAgents, fetchAiRuns, fetchAiStatus, fetchOwnerPersona } from '$lib/api';
import type { AiAgent, AiRun, AiStatus, OwnerPersona } from '$lib/api';
import { agentCardView, loadAiWorkspace } from './ai';

const ownerPersona = {
	person_id: 'person:v1:email:owner@example.com',
	display_name: 'owner@example.com',
	email_address: 'owner@example.com',
	persona_type: 'human',
	is_self: true,
	created_at: '2026-06-13T09:00:00Z',
	updated_at: '2026-06-13T09:00:00Z'
} satisfies OwnerPersona;

const aiStatus = {
	runtime: 'local',
	status: 'ok',
	version: null,
	chat_model: 'llama',
	chat_model_available: true,
	embedding_model: 'nomic',
	embedding_model_available: true,
	embedding_dimension: 768
} satisfies AiStatus;

describe('AI workspace owner context', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads Owner Persona context alongside agents and run history', async () => {
		vi.mocked(fetchAiAgents).mockResolvedValue({ items: [] });
		vi.mocked(fetchAiRuns).mockResolvedValue({ items: [] });
		vi.mocked(fetchAiStatus).mockResolvedValue(aiStatus);
		vi.mocked(fetchOwnerPersona).mockResolvedValue({ owner_persona: ownerPersona });

		const result = await loadAiWorkspace();

		expect(fetchOwnerPersona).toHaveBeenCalledTimes(1);
		expect(result.ownerPersona).toEqual(ownerPersona);
		expect(result.error).toBe('');
	});
});

describe('AI agent Persona identity', () => {
	it('uses the agent Persona email as the visible card name', () => {
		const agent = {
			agent_id: 'HESTIA',
			display_name: 'Hestia',
			role: 'meeting prep',
			default_model: 'llama',
			status: 'available',
			persona_id: 'persona:v1:ai_agent:HESTIA',
			persona_type: 'ai_agent',
			persona_email: 'hestia@sh-inc.ru'
		} satisfies AiAgent;

		const card = agentCardView(agent, [] satisfies AiRun[]);

		expect(card.name).toBe('hestia@sh-inc.ru');
	});

	it('derives the visible Persona email from the agent id when the API has not materialized one', () => {
		const agent = {
			agent_id: 'HESTIA',
			display_name: 'Hestia',
			role: 'meeting prep',
			default_model: 'llama',
			status: 'available'
		} satisfies AiAgent;

		const card = agentCardView(agent, [] satisfies AiRun[]);

		expect(card.name).toBe('hestia@sh-inc.ru');
	});
});
