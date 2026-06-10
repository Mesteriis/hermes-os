import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import {
	activeCommunicationSection,
	activeView,
	activeWorkspaceView,
	currentView,
	navigateToCommunicationSection,
	shellViewClass
} from './navigation';

describe('navigation store', () => {
	beforeEach(() => {
		currentView.set('home');
		activeCommunicationSection.set('unified');
	});

	it('uses communication sub-sections as the active workspace view for Telegram and WhatsApp', () => {
		navigateToCommunicationSection('telegram');

		expect(get(currentView)).toBe('communications');
		expect(get(activeCommunicationSection)).toBe('telegram');
		expect(get(activeWorkspaceView)).toBe('telegram');
		expect(get(activeView).title).toBe('Telegram Client');
		expect(get(shellViewClass)).toBe('view-telegram');

		navigateToCommunicationSection('whatsapp');

		expect(get(activeWorkspaceView)).toBe('whatsapp');
		expect(get(activeView).title).toBe('WhatsApp Web');
		expect(get(shellViewClass)).toBe('view-whatsapp');
	});
});
