// Default grid geometry shared by the widget registry (drives min sizes) and the
// layout presets (drives each instance's starting columns/rows). Kept in one place
// so the two consumers cannot drift; this table was previously duplicated verbatim.

const defaultGridByZone: Record<string, { columns: number; rows: number }> = {
	header: { columns: 12, rows: 3 },
	hero: { columns: 12, rows: 3 },
	metrics: { columns: 12, rows: 3 },
	filters: { columns: 12, rows: 1 },
	toolbar: { columns: 12, rows: 1 },
	metadata: { columns: 12, rows: 2 },
	tabs: { columns: 12, rows: 1 },
	list: { columns: 3, rows: 12 },
	detail: { columns: 6, rows: 12 },
	main: { columns: 4, rows: 12 },
	canvas: { columns: 9, rows: 12 },
	rail: { columns: 3, rows: 6 },
	inspector: { columns: 3, rows: 6 },
	bottom: { columns: 12, rows: 4 }
};

const defaultGridByWidget: Record<string, { columns: number; rows: number }> = {
	'documents-list': { columns: 6, rows: 12 },
	'documents-related-context': { columns: 3, rows: 12 },
	'telegram-account-status': { columns: 12, rows: 3 },
	'telegram-sync-controls': { columns: 3, rows: 12 },
	'telegram-selected-chat-metadata': { columns: 3, rows: 12 },
	'whatsapp-sync-controls': { columns: 3, rows: 12 }
};

export function defaultWidgetGrid(
	widgetId: string,
	zoneId: string
): { columns: number; rows: number } {
	return defaultGridByWidget[widgetId] ?? defaultGridByZone[zoneId] ?? { columns: 4, rows: 4 };
}
