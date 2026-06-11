export const UI_STATE_SCHEMA_VERSION = 1;
export const UI_STATE_TTL_MS = 7 * 24 * 60 * 60 * 1000;
export const UI_STATE_LOCAL_STORAGE_KEY = 'hermes-hub.ui-state.v1';

const APP_VIEWS = new Set([
	'home',
	'communications',
	'persons',
	'projects',
	'tasks',
	'calendar',
	'documents',
	'notes',
	'knowledge',
	'agents',
	'organizations',
	'settings',
	'timeline'
]);
const COMMUNICATION_SECTIONS = new Set([
	'unified',
	'inbox',
	'waiting',
	'needs_reply',
	'mail',
	'telegram',
	'whatsapp',
	'calls',
	'meetings',
	'timeline',
	'mentions'
]);
const WORKFLOW_STATES = new Set(['', 'new', 'needs_action', 'waiting', 'done', 'archived']);
const LOCAL_MESSAGE_STATES = new Set(['active', 'trash']);
const NAVIGATOR_MODES = new Set(['threads', 'contacts']);
const INSPECTOR_MODES = new Set(['context', 'contact', 'organization']);
const MESSAGE_TABS = new Set(['message', 'attachments', 'headers', 'related', 'timeline']);
const COMPOSE_MODES = new Set(['compose', 'reply', 'forward']);
const SETTINGS_SECTIONS = new Set(['appearance', 'application', 'sidebar', 'integrations', 'language']);

export type UiStateStorageDriver = {
	read: () => string | null;
	write: (value: string) => void;
	remove: () => void;
};

export type UiStateSnapshotV1 = {
	schemaVersion: 1;
	savedAt: string;
	expiresAt: string;
	shell?: {
		currentView?: string;
		activeCommunicationSection?: string;
		isSidebarRail?: boolean;
		expandedSidebarGroupIds?: string[];
		activeSidebarRailGroupId?: string | null;
	};
	workspace?: {
		selectedSettingsSection?: string;
		isNotificationsDrawerOpen?: boolean;
		dismissedNotificationIds?: string[];
		expandedNotificationIds?: string[];
		isLayoutEditing?: boolean;
		isWidgetDrawerOpen?: boolean;
		selectedLayoutWidgetId?: string | null;
	};
	communications?: {
		selectedMailAccountId?: string;
		mailStateFilter?: string;
		mailLocalStateFilter?: string;
		messageSearchQuery?: string;
		selectedMessageId?: string | null;
		navigatorMode?: string;
		expandedContactKey?: string | null;
		inspectorMode?: string | null;
		activeTab?: string;
		compose?: {
			isOpen?: boolean;
			mode?: string;
			draftId?: string;
			accountId?: string;
			sourceMessageId?: string | null;
		};
	};
};

export type UiStateSnapshotInput = Omit<UiStateSnapshotV1, 'schemaVersion' | 'savedAt' | 'expiresAt'>;
type UiStateCommunicationsInput = NonNullable<UiStateSnapshotInput['communications']>;

export function buildUiStateSnapshot(
	input: UiStateSnapshotInput,
	nowMs = Date.now()
): UiStateSnapshotV1 {
	const snapshot = parseKnownUiStateFields(input);
	return stripEmptyObjects({
		schemaVersion: UI_STATE_SCHEMA_VERSION,
		savedAt: new Date(nowMs).toISOString(),
		expiresAt: new Date(nowMs + UI_STATE_TTL_MS).toISOString(),
		...snapshot
	});
}

export function parseUiStateSnapshot(value: unknown, nowMs = Date.now()): UiStateSnapshotV1 | null {
	const source = objectValue(value);
	if (!source || source.schemaVersion !== UI_STATE_SCHEMA_VERSION) {
		return null;
	}

	const savedAt = parseIsoDate(source.savedAt);
	const expiresAt = parseIsoDate(source.expiresAt);
	if (!savedAt || !expiresAt || expiresAt.getTime() <= nowMs) {
		return null;
	}

	const snapshot = parseKnownUiStateFields(source);
	return stripEmptyObjects({
		schemaVersion: UI_STATE_SCHEMA_VERSION,
		savedAt: savedAt.toISOString(),
		expiresAt: expiresAt.toISOString(),
		...snapshot
	});
}

export function localStorageUiStateDriver(): UiStateStorageDriver | null {
	if (typeof window === 'undefined' || !window.localStorage) {
		return null;
	}
	return {
		read: () => window.localStorage.getItem(UI_STATE_LOCAL_STORAGE_KEY),
		write: (value: string) => window.localStorage.setItem(UI_STATE_LOCAL_STORAGE_KEY, value),
		remove: () => window.localStorage.removeItem(UI_STATE_LOCAL_STORAGE_KEY)
	};
}

export function loadUiStateFromLocalStorage(
	driver: UiStateStorageDriver | null,
	nowMs = Date.now()
): UiStateSnapshotV1 | null {
	if (!driver) {
		return null;
	}
	try {
		const raw = driver.read();
		if (!raw) {
			return null;
		}
		return parseUiStateSnapshot(JSON.parse(raw), nowMs);
	} catch {
		return null;
	}
}

export function saveUiStateToLocalStorage(
	driver: UiStateStorageDriver | null,
	snapshot: UiStateSnapshotV1
): void {
	if (!driver) {
		return;
	}
	try {
		driver.write(JSON.stringify(snapshot));
	} catch {
		// Persistence is best-effort; backend settings remain the durable mirror.
	}
}

export function removeUiStateFromLocalStorage(driver: UiStateStorageDriver | null): void {
	if (!driver) {
		return;
	}
	try {
		driver.remove();
	} catch {
		// Nothing actionable for disabled localStorage.
	}
}

function parseKnownUiStateFields(source: unknown): UiStateSnapshotInput {
	const value = objectValue(source) ?? {};
	const shell = sanitizeShell(value.shell);
	const workspace = sanitizeWorkspace(value.workspace);
	const communications = sanitizeCommunications(value.communications);
	return stripEmptyObjects({ shell, workspace, communications });
}

function sanitizeShell(value: unknown): UiStateSnapshotInput['shell'] {
	const source = objectValue(value);
	if (!source) return undefined;
	return stripEmptyObjects({
		currentView: enumString(source.currentView, APP_VIEWS),
		activeCommunicationSection: enumString(source.activeCommunicationSection, COMMUNICATION_SECTIONS),
		isSidebarRail: booleanValue(source.isSidebarRail),
		expandedSidebarGroupIds: stringArray(source.expandedSidebarGroupIds, 24),
		activeSidebarRailGroupId: nullableString(source.activeSidebarRailGroupId)
	});
}

function sanitizeWorkspace(value: unknown): UiStateSnapshotInput['workspace'] {
	const source = objectValue(value);
	if (!source) return undefined;
	return stripEmptyObjects({
		selectedSettingsSection: enumString(source.selectedSettingsSection, SETTINGS_SECTIONS),
		isNotificationsDrawerOpen: booleanValue(source.isNotificationsDrawerOpen),
		dismissedNotificationIds: stringArray(source.dismissedNotificationIds, 100),
		expandedNotificationIds: stringArray(source.expandedNotificationIds, 100),
		isLayoutEditing: booleanValue(source.isLayoutEditing),
		isWidgetDrawerOpen: booleanValue(source.isWidgetDrawerOpen),
		selectedLayoutWidgetId: nullableString(source.selectedLayoutWidgetId)
	});
}

function sanitizeCommunications(value: unknown): UiStateSnapshotInput['communications'] {
	const source = objectValue(value);
	if (!source) return undefined;
	return stripEmptyObjects({
		selectedMailAccountId: stringValue(source.selectedMailAccountId),
		mailStateFilter: enumString(source.mailStateFilter, WORKFLOW_STATES),
		mailLocalStateFilter: enumString(source.mailLocalStateFilter, LOCAL_MESSAGE_STATES),
		messageSearchQuery: stringValue(source.messageSearchQuery),
		selectedMessageId: nullableString(source.selectedMessageId),
		navigatorMode: enumString(source.navigatorMode, NAVIGATOR_MODES),
		expandedContactKey: nullableString(source.expandedContactKey),
		inspectorMode: nullableEnumString(source.inspectorMode, INSPECTOR_MODES),
		activeTab: enumString(source.activeTab, MESSAGE_TABS),
		compose: sanitizeCompose(source.compose)
	});
}

function sanitizeCompose(value: unknown): UiStateCommunicationsInput['compose'] {
	const source = objectValue(value);
	if (!source) return undefined;
	return stripEmptyObjects({
		isOpen: booleanValue(source.isOpen),
		mode: enumString(source.mode, COMPOSE_MODES),
		draftId: stringValue(source.draftId),
		accountId: stringValue(source.accountId),
		sourceMessageId: nullableString(source.sourceMessageId)
	});
}

function stripEmptyObjects<T>(value: T): T {
	if (!value || typeof value !== 'object' || Array.isArray(value)) {
		return value;
	}
	const result: Record<string, unknown> = {};
	for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
		if (child === undefined) continue;
		const nextChild = stripEmptyObjects(child);
		if (
			nextChild &&
			typeof nextChild === 'object' &&
			!Array.isArray(nextChild) &&
			Object.keys(nextChild as Record<string, unknown>).length === 0
		) {
			continue;
		}
		result[key] = nextChild;
	}
	return result as T;
}

function objectValue(value: unknown): Record<string, unknown> | null {
	return value !== null && typeof value === 'object' && !Array.isArray(value)
		? (value as Record<string, unknown>)
		: null;
}

function parseIsoDate(value: unknown): Date | null {
	if (typeof value !== 'string') return null;
	const date = new Date(value);
	return Number.isFinite(date.getTime()) ? date : null;
}

function stringValue(value: unknown): string | undefined {
	return typeof value === 'string' ? value.slice(0, 512) : undefined;
}

function nullableString(value: unknown): string | null | undefined {
	if (value === null) return null;
	return stringValue(value);
}

function enumString(value: unknown, allowed: Set<string>): string | undefined {
	return typeof value === 'string' && allowed.has(value) ? value : undefined;
}

function nullableEnumString(value: unknown, allowed: Set<string>): string | null | undefined {
	if (value === null) return null;
	return enumString(value, allowed);
}

function booleanValue(value: unknown): boolean | undefined {
	return typeof value === 'boolean' ? value : undefined;
}

function stringArray(value: unknown, limit: number): string[] | undefined {
	if (!Array.isArray(value)) return undefined;
	const values = value.filter((item): item is string => typeof item === 'string').slice(0, limit);
	return values.length ? values : undefined;
}
