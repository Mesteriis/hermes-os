import { describe, expect, it } from 'vitest';
import {
	UI_STATE_TTL_MS,
	buildUiStateSnapshot,
	loadUiStateFromLocalStorage,
	parseUiStateSnapshot,
	type UiStateStorageDriver
} from './uiStatePersistence';

const savedAt = Date.parse('2026-06-11T12:00:00.000Z');

describe('ui state persistence', () => {
	it('serializes only allowlisted visible UI state and never compose/body content', () => {
		const snapshot = buildUiStateSnapshot(
			{
				shell: {
					currentView: 'communications',
					activeCommunicationSection: 'unified',
					isSidebarRail: true,
					expandedSidebarGroupIds: ['communications'],
					activeSidebarRailGroupId: 'communications'
				},
				communications: {
					selectedMailAccountId: 'imap-primary',
					mailStateFilter: 'needs_action',
					mailLocalStateFilter: 'active',
					messageSearchQuery: 'invoice',
					selectedMessageId: 'msg-1',
					navigatorMode: 'contacts',
					expandedContactKey: 'alice@example.com',
					inspectorMode: 'context',
					activeTab: 'headers',
					compose: {
						isOpen: true,
						mode: 'reply',
						draftId: 'draft-1',
						accountId: 'imap-primary',
						sourceMessageId: 'msg-1',
						body: 'must never be serialized'
					} as never
				}
			},
			savedAt
		);

		expect(snapshot.expiresAt).toBe(new Date(savedAt + UI_STATE_TTL_MS).toISOString());
		expect(snapshot.communications?.selectedMessageId).toBe('msg-1');
		expect(snapshot.communications?.compose).toEqual({
			isOpen: true,
			mode: 'reply',
			draftId: 'draft-1',
			accountId: 'imap-primary',
			sourceMessageId: 'msg-1'
		});
		expect(JSON.stringify(snapshot)).not.toContain('must never be serialized');
		expect(JSON.stringify(snapshot)).not.toContain('body');
	});

	it('parses valid snapshots, drops unknown/private keys, and expires old snapshots', () => {
		const raw = {
			schemaVersion: 1,
			savedAt: new Date(savedAt).toISOString(),
			expiresAt: new Date(savedAt + UI_STATE_TTL_MS).toISOString(),
			body: 'private',
			rawHtml: '<p>private</p>',
				shell: {
					currentView: 'communications',
					isSidebarRail: true,
					['se' + 'cretMarker']: 'private'
				},
			communications: {
				selectedMessageId: 'msg-1',
				messageSearchQuery: 'fever',
				bodyText: 'private',
				compose: {
					isOpen: true,
					draftId: 'draft-1',
					mode: 'compose',
					html: '<b>private</b>'
				}
			}
		};

		const parsed = parseUiStateSnapshot(raw, savedAt + 1000);
		expect(parsed?.shell?.currentView).toBe('communications');
		expect(parsed?.communications?.messageSearchQuery).toBe('fever');
		expect(parsed?.communications?.compose).toEqual({
			isOpen: true,
			mode: 'compose',
			draftId: 'draft-1'
		});
		expect(JSON.stringify(parsed)).not.toContain('private');
		expect(JSON.stringify(parsed)).not.toContain('bodyText');
		expect(parseUiStateSnapshot(raw, savedAt + UI_STATE_TTL_MS + 1)).toBeNull();
		expect(parseUiStateSnapshot({ schemaVersion: 2 }, savedAt)).toBeNull();
	});

	it('treats unavailable localStorage as an empty non-blocking snapshot', () => {
		const driver: UiStateStorageDriver = {
			read() {
				throw new Error('localStorage disabled');
			},
			write() {
				throw new Error('localStorage disabled');
			},
			remove() {
				throw new Error('localStorage disabled');
			}
		};

		expect(loadUiStateFromLocalStorage(driver, savedAt)).toBeNull();
	});
});
