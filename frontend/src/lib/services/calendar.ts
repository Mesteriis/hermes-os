import {
	fetchCalendarAccounts,
	fetchCalendarSources,
	fetchCalendarEvents,
	fetchCalendarWatchtower,
	fetchWeeklyBrief,
	searchCalendarEvents,
	fetchEventBrief,
	fetchEventContextPack,
	fetchEventAgenda,
	fetchMeetingNotes,
	createCalendarEvent,
	type CalendarAccount,
	type CalendarSource,
	type CalendarEvent
} from '$lib/api';

export async function loadCalendar(): Promise<{
	accounts: CalendarAccount[];
	events: CalendarEvent[];
	sources: CalendarSource[];
	watchtower: Record<string, unknown>;
	error: string;
}> {
	try {
		const [accts, events] = await Promise.all([
			fetchCalendarAccounts(),
			fetchCalendarEvents({ limit: 200 })
		]);
		const sources: CalendarSource[] = [];
		for (const acct of accts.items) {
			try {
				const srcs = await fetchCalendarSources(acct.account_id);
				sources.push(...srcs.items);
			} catch (_) { /* sources optional */ }
		}
		fetchCalendarWatchtower().then((r) => { /* caller will handle via state */ }).catch(() => {});
		return {
			accounts: accts.items,
			events: events.items,
			sources,
			watchtower: {},
			error: ''
		};
	} catch (error) {
		return {
			accounts: [],
			events: [],
			sources: [],
			watchtower: {},
			error: error instanceof Error ? error.message : 'Calendar load failed'
		};
	}
}

export async function loadCalendarWatchtower(): Promise<{ watchtower: Record<string, unknown> }> {
	try {
		const r = await fetchCalendarWatchtower();
		return { watchtower: r };
	} catch {
		return { watchtower: {} };
	}
}

export function getEventTimeRange(calendarViewMode: string): { from: string; to: string } {
	const now = new Date();
	const from = new Date(now);
	if (calendarViewMode === 'day') { from.setHours(0, 0, 0, 0); }
	else if (calendarViewMode === 'week') { from.setDate(now.getDate() - now.getDay() + 1); from.setHours(0, 0, 0, 0); }
	else { from.setDate(1); from.setHours(0, 0, 0, 0); }
	const to = new Date(from);
	if (calendarViewMode === 'day') to.setDate(to.getDate() + 1);
	else if (calendarViewMode === 'week') to.setDate(to.getDate() + 7);
	else to.setMonth(to.getMonth() + 1);
	return { from: from.toISOString(), to: to.toISOString() };
}

export async function prepareEvent(evt: CalendarEvent): Promise<{
	context: Record<string, unknown> | null;
	brief: Record<string, unknown> | null;
	agenda: Record<string, unknown> | null;
}> {
	try {
		const [ctx, brief, agenda] = await Promise.all([
			fetchEventContextPack(evt.event_id),
			fetchEventBrief(evt.event_id),
			fetchEventAgenda(evt.event_id)
		]);
		return { context: ctx, brief, agenda };
	} catch (_) {
		return { context: null, brief: null, agenda: null };
	}
}

export async function completeEvent(evt: CalendarEvent): Promise<{
	context: Record<string, unknown> | null;
}> {
	try {
		const notes = await fetchMeetingNotes(evt.event_id);
		return { context: { notes: notes.items } };
	} catch (_) {
		return { context: null };
	}
}

export async function searchCalendar(query: string): Promise<{ results: CalendarEvent[] }> {
	if (!query.trim()) {
		return { results: [] };
	}
	try {
		const result = await searchCalendarEvents(query);
		return { results: (result.results as CalendarEvent[]) || [] };
	} catch (_) {
		return { results: [] };
	}
}

export async function loadWeeklyBrief(): Promise<{ brief: Record<string, unknown> | null }> {
	try {
		const brief = await fetchWeeklyBrief();
		return { brief };
	} catch (_) {
		return { brief: null };
	}
}

export async function handleCreateEvent(
	title: string,
	startAt: string,
	endAt: string,
	eventType: string
): Promise<{ error: string }> {
	if (!title || !startAt || !endAt) return { error: '' };
	try {
		await createCalendarEvent({
			title,
			start_at: new Date(startAt).toISOString(),
			end_at: new Date(endAt).toISOString(),
			event_type: eventType
		});
		return { error: '' };
	} catch (e) {
		return { error: e instanceof Error ? e.message : 'Create failed' };
	}
}
