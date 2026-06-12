import { get, type Readable, type Unsubscriber } from 'svelte/store';
import {
	FRONTEND_UI_STATE_SETTING_KEY,
	saveFrontendUiStateSetting,
	type ApplicationSetting
} from '$lib/api';
import {
	UI_STATE_LOCAL_STORAGE_KEY,
	buildUiStateSnapshot,
	loadUiStateFromLocalStorage,
	localStorageUiStateDriver,
	parseUiStateSnapshot,
	saveUiStateToLocalStorage,
	type UiStateSnapshotV1,
	type UiStateStorageDriver
} from '$lib/services/uiStatePersistence';
import {
	activeCommunicationSection,
	activeSidebarRailGroupId,
	currentView,
	expandedSidebarGroupIds,
	isSidebarRail
} from './navigation';
import {
	activeMessageContextTab,
	autoSaveOpenComposeDraft,
	communicationsInspectorMode,
	communicationsNavigatorMode,
	composeForm,
	expandedCommunicationContactKey,
	isComposeOpen,
	mailLocalStateFilter,
	mailStateFilter,
	messageSearchQuery,
	restoreComposeDraftById,
	selectedCommunication,
	selectedCommunicationMessageId,
	selectedMailAccountId
} from './communications';
import {
	dismissedNotificationIds,
	expandedNotificationIds,
	isNotificationsDrawerOpen
} from './notifications';
import {
	isLayoutEditing,
	isWidgetDrawerOpen,
	selectedLayoutWidgetId
} from './layoutEditor';
import {
	applicationSettings,
	selectedSettingsSection
} from './settings';

const SAVE_DEBOUNCE_MS = 450;
const COMPOSE_AUTOSAVE_DEBOUNCE_MS = 900;

let initialized = false;
let isApplyingSnapshot = false;
let isSubscribing = false;
let localDriver: UiStateStorageDriver | null = null;
let saveTimer: ReturnType<typeof setTimeout> | null = null;
let composeSaveTimer: ReturnType<typeof setTimeout> | null = null;
let lastAppliedSavedAtMs = 0;
let lastPersistedJson = '';
let pendingComposeDraftId: string | null = null;
const unsubscribers: Unsubscriber[] = [];

export function initializeUiStatePersistence(): void {
	if (initialized) {
		return;
	}
	initialized = true;
	localDriver = localStorageUiStateDriver();
	const localSnapshot = loadUiStateFromLocalStorage(localDriver);
	if (localSnapshot) {
		applyUiStateSnapshot(localSnapshot);
	}
	startPersistenceSubscriptions();
}

export function restoreUiStateFromBackendSettings(): void {
	const setting = uiStateSetting();
	const backendSnapshot = parseUiStateSnapshot(setting?.value ?? null);
	if (!backendSnapshot) {
		return;
	}
	applyUiStateSnapshot(backendSnapshot);
	scheduleUiStatePersist();
}

export async function restoreComposeFromPersistedUiState(): Promise<void> {
	const draftId = pendingComposeDraftId;
	pendingComposeDraftId = null;
	if (!draftId) {
		return;
	}
	await restoreComposeDraftById(draftId);
}

export function currentUiStateSnapshot(): UiStateSnapshotV1 {
	return buildUiStateSnapshot({
		shell: {
			currentView: get(currentView),
			activeCommunicationSection: get(activeCommunicationSection),
			isSidebarRail: get(isSidebarRail),
			expandedSidebarGroupIds: get(expandedSidebarGroupIds),
			activeSidebarRailGroupId: get(activeSidebarRailGroupId)
		},
		workspace: {
			selectedSettingsSection: get(selectedSettingsSection),
			isNotificationsDrawerOpen: get(isNotificationsDrawerOpen),
			dismissedNotificationIds: get(dismissedNotificationIds),
			expandedNotificationIds: get(expandedNotificationIds),
			isLayoutEditing: get(isLayoutEditing),
			isWidgetDrawerOpen: get(isWidgetDrawerOpen),
			selectedLayoutWidgetId: get(selectedLayoutWidgetId)
		},
		communications: {
			selectedMailAccountId: get(selectedMailAccountId),
			mailStateFilter: get(mailStateFilter),
			mailLocalStateFilter: get(mailLocalStateFilter),
			messageSearchQuery: get(messageSearchQuery),
			selectedMessageId:
				get(selectedCommunicationMessageId) ??
				get(selectedCommunication)?.message_id ??
				null,
			navigatorMode: get(communicationsNavigatorMode),
			expandedContactKey: get(expandedCommunicationContactKey),
			inspectorMode: get(communicationsInspectorMode),
			activeTab: get(activeMessageContextTab),
			compose: composeStateForSnapshot()
		}
	});
}

function applyUiStateSnapshot(snapshot: UiStateSnapshotV1): boolean {
	const savedAtMs = Date.parse(snapshot.savedAt);
	if (!Number.isFinite(savedAtMs) || savedAtMs <= lastAppliedSavedAtMs) {
		return false;
	}
	lastAppliedSavedAtMs = savedAtMs;
	isApplyingSnapshot = true;
	try {
		if (snapshot.shell?.currentView) currentView.set(snapshot.shell.currentView as never);
		if (snapshot.shell?.activeCommunicationSection) {
			activeCommunicationSection.set(snapshot.shell.activeCommunicationSection as never);
		}
		if (typeof snapshot.shell?.isSidebarRail === 'boolean') {
			isSidebarRail.set(snapshot.shell.isSidebarRail);
		}
		if (snapshot.shell?.expandedSidebarGroupIds) {
			expandedSidebarGroupIds.set(snapshot.shell.expandedSidebarGroupIds);
		}
		activeSidebarRailGroupId.set(snapshot.shell?.activeSidebarRailGroupId ?? null);

		if (snapshot.workspace?.selectedSettingsSection) {
			selectedSettingsSection.set(snapshot.workspace.selectedSettingsSection as never);
		}
		isNotificationsDrawerOpen.set(Boolean(snapshot.workspace?.isNotificationsDrawerOpen));
		dismissedNotificationIds.set(snapshot.workspace?.dismissedNotificationIds ?? []);
		expandedNotificationIds.set(snapshot.workspace?.expandedNotificationIds ?? []);
		isLayoutEditing.set(Boolean(snapshot.workspace?.isLayoutEditing));
		isWidgetDrawerOpen.set(Boolean(snapshot.workspace?.isWidgetDrawerOpen));
		selectedLayoutWidgetId.set(snapshot.workspace?.selectedLayoutWidgetId ?? null);

		if (snapshot.communications?.selectedMailAccountId !== undefined) {
			selectedMailAccountId.set(snapshot.communications.selectedMailAccountId);
		}
		if (snapshot.communications?.mailStateFilter !== undefined) {
			mailStateFilter.set(snapshot.communications.mailStateFilter as never);
		}
		if (snapshot.communications?.mailLocalStateFilter) {
			mailLocalStateFilter.set(snapshot.communications.mailLocalStateFilter as never);
		}
		if (snapshot.communications?.messageSearchQuery !== undefined) {
			messageSearchQuery.set(snapshot.communications.messageSearchQuery);
		}
		selectedCommunicationMessageId.set(snapshot.communications?.selectedMessageId ?? null);
		if (snapshot.communications?.navigatorMode) {
			communicationsNavigatorMode.set(snapshot.communications.navigatorMode as never);
		}
		expandedCommunicationContactKey.set(snapshot.communications?.expandedContactKey ?? null);
		communicationsInspectorMode.set((snapshot.communications?.inspectorMode ?? null) as never);
		if (snapshot.communications?.activeTab) {
			activeMessageContextTab.set(snapshot.communications.activeTab as never);
		}
		pendingComposeDraftId = snapshot.communications?.compose?.isOpen
			? snapshot.communications.compose.draftId ?? null
			: null;
		return true;
	} finally {
		isApplyingSnapshot = false;
	}
}

function startPersistenceSubscriptions(): void {
	isSubscribing = true;
	for (const store of persistedStores()) {
		unsubscribers.push(store.subscribe(() => {
			if (!isSubscribing && !isApplyingSnapshot) {
				scheduleUiStatePersist();
			}
		}));
	}
	for (const store of [isComposeOpen, composeForm]) {
		unsubscribers.push(store.subscribe(() => {
			if (!isSubscribing && !isApplyingSnapshot) {
				scheduleComposeAutoSave();
				scheduleUiStatePersist();
			}
		}));
	}
	isSubscribing = false;
}

function persistedStores(): Readable<unknown>[] {
	return [
		currentView,
		activeCommunicationSection,
		activeSidebarRailGroupId,
		isSidebarRail,
		expandedSidebarGroupIds,
		selectedSettingsSection,
		isNotificationsDrawerOpen,
		dismissedNotificationIds,
		expandedNotificationIds,
		isLayoutEditing,
		isWidgetDrawerOpen,
		selectedLayoutWidgetId,
		selectedMailAccountId,
		mailStateFilter,
		mailLocalStateFilter,
		messageSearchQuery,
		selectedCommunicationMessageId,
		communicationsNavigatorMode,
		expandedCommunicationContactKey,
		communicationsInspectorMode,
		activeMessageContextTab
	];
}

function scheduleUiStatePersist(): void {
	if (saveTimer) {
		clearTimeout(saveTimer);
	}
	saveTimer = setTimeout(() => {
		saveTimer = null;
		void persistUiStateSnapshot();
	}, SAVE_DEBOUNCE_MS);
}

async function persistUiStateSnapshot(): Promise<void> {
	const snapshot = currentUiStateSnapshot();
	const json = JSON.stringify(snapshot);
	if (json === lastPersistedJson) {
		return;
	}
	lastPersistedJson = json;
	saveUiStateToLocalStorage(localDriver, snapshot);
	const setting = uiStateSetting();
	if (!setting) {
		return;
	}
	try {
		const updated = await saveFrontendUiStateSetting(snapshot as unknown as Record<string, unknown>);
		applicationSettings.update((items) =>
			items.map((item) => item.setting_key === FRONTEND_UI_STATE_SETTING_KEY ? updated : item)
		);
	} catch {
		// Backend persistence is intentionally non-blocking; localStorage keeps reload continuity.
	}
}

function scheduleComposeAutoSave(): void {
	if (composeSaveTimer) {
		clearTimeout(composeSaveTimer);
	}
	composeSaveTimer = setTimeout(() => {
		composeSaveTimer = null;
		void autoSaveOpenComposeDraft();
	}, COMPOSE_AUTOSAVE_DEBOUNCE_MS);
}

function composeStateForSnapshot(): NonNullable<UiStateSnapshotV1['communications']>['compose'] {
	const form = get(composeForm);
	return {
		isOpen: get(isComposeOpen),
		mode: form.mode,
		draftId: form.draft_id,
		accountId: form.account_id,
		sourceMessageId: get(selectedCommunicationMessageId)
	};
}

function uiStateSetting(): ApplicationSetting | null {
	return get(applicationSettings).find((setting) => setting.setting_key === FRONTEND_UI_STATE_SETTING_KEY) ?? null;
}

export function resetUiStatePersistenceForTests(): void {
	for (const unsubscribe of unsubscribers.splice(0)) {
		unsubscribe();
	}
	if (saveTimer) clearTimeout(saveTimer);
	if (composeSaveTimer) clearTimeout(composeSaveTimer);
	initialized = false;
	isApplyingSnapshot = false;
	isSubscribing = false;
	localDriver = null;
	saveTimer = null;
	composeSaveTimer = null;
	lastAppliedSavedAtMs = 0;
	lastPersistedJson = '';
	pendingComposeDraftId = null;
	if (typeof window !== 'undefined') {
		window.localStorage?.removeItem(UI_STATE_LOCAL_STORAGE_KEY);
	}
}
