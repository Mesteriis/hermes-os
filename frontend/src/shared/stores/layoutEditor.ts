import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type WidgetGridDimension = 'columns' | 'rows'
export type WidgetPanelSurfaceSetting = 'panelOpacity' | 'panelBlur'
export type ScrollMode = 'default' | 'horizontal' | 'vertical' | 'none'

export type ViewLayoutOverride = {
  hiddenWidgetIds: string[]
  zoneOverrides: Record<string, unknown>
  orderOverrides: Record<string, unknown>
  gridOverrides: Record<string, { columns: number; rows: number }>
  panelSurfaceOverrides: Record<string, { panelOpacity: number; panelBlur: number }>
}

export type WidgetDefinition = {
  id: string
  title: string
  icon: string
  viewScope: string[]
  defaultColumns: number
  defaultRows: number
  minColumns: number
  minRows: number
  canAdd: boolean
  removable: boolean
}

export type ResolvedWidget = {
  id: string
  title: string
  icon: string
  columns: number
  rows: number
  minColumns: number
  minRows: number
  canAdd: boolean
  removable: boolean
  panelOpacity: number
  panelBlur: number
  zone: string
  order: number
}

export type ResolvedLayout = {
  viewId: string
  widgets: ResolvedWidget[]
  widgetById: Map<string, ResolvedWidget>
}

export type LayoutSettings = {
  schemaVersion: number
  views: Record<string, ViewLayoutOverride>
}

function defaultLayoutSettings(): LayoutSettings {
  return {
    schemaVersion: 2,
    views: {}
  }
}

const defaultWidgets: WidgetDefinition[] = [
  { id: 'home-welcome', title: 'Welcome', icon: 'tabler:home', viewScope: ['home'], defaultColumns: 6, defaultRows: 2, minColumns: 3, minRows: 1, canAdd: true, removable: true },
  { id: 'home-stats', title: 'Statistics', icon: 'tabler:chart-bar', viewScope: ['home'], defaultColumns: 6, defaultRows: 2, minColumns: 3, minRows: 1, canAdd: true, removable: true },
  { id: 'home-timeline', title: 'Recent Activity', icon: 'tabler:timeline-event', viewScope: ['home'], defaultColumns: 4, defaultRows: 3, minColumns: 2, minRows: 2, canAdd: true, removable: true },
  { id: 'persons-list', title: 'Personas', icon: 'tabler:user', viewScope: ['persons'], defaultColumns: 6, defaultRows: 4, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'persons-recent', title: 'Recent Persons', icon: 'tabler:user-plus', viewScope: ['persons'], defaultColumns: 6, defaultRows: 2, minColumns: 3, minRows: 1, canAdd: true, removable: true },
  { id: 'projects-list', title: 'Projects', icon: 'tabler:briefcase', viewScope: ['projects'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'tasks-list', title: 'Tasks', icon: 'tabler:checkbox', viewScope: ['tasks'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'calendar-month', title: 'Month View', icon: 'tabler:calendar', viewScope: ['calendar'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'calendar-upcoming', title: 'Upcoming', icon: 'tabler:calendar-event', viewScope: ['calendar'], defaultColumns: 4, defaultRows: 4, minColumns: 2, minRows: 2, canAdd: true, removable: true },
  { id: 'documents-list', title: 'Documents', icon: 'tabler:file-text', viewScope: ['documents'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'notes-list', title: 'Notes', icon: 'tabler:notes', viewScope: ['notes'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'knowledge-graph', title: 'Graph Explorer', icon: 'tabler:share', viewScope: ['knowledge'], defaultColumns: 12, defaultRows: 6, minColumns: 6, minRows: 3, canAdd: true, removable: true },
  { id: 'review-queue', title: 'Review Queue', icon: 'tabler:clipboard-check', viewScope: ['review'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'communications-unified', title: 'Unified Inbox', icon: 'tabler:messages', viewScope: ['communications'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true },
  { id: 'communications-mail', title: 'Mail', icon: 'tabler:mail', viewScope: ['communications'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'communications-telegram', title: 'Telegram', icon: 'tabler:brand-telegram', viewScope: ['communications', 'telegram'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'communications-whatsapp', title: 'WhatsApp', icon: 'tabler:brand-whatsapp', viewScope: ['communications', 'whatsapp'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'settings-general', title: 'General', icon: 'tabler:settings', viewScope: ['settings'], defaultColumns: 6, defaultRows: 3, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'settings-accounts', title: 'Accounts', icon: 'tabler:plug', viewScope: ['settings'], defaultColumns: 6, defaultRows: 3, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'settings-theme', title: 'Theme', icon: 'tabler:palette', viewScope: ['settings'], defaultColumns: 6, defaultRows: 3, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'agents-overview', title: 'AI Agents', icon: 'tabler:sparkles', viewScope: ['agents'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'organizations-list', title: 'Organizations', icon: 'tabler:building-community', viewScope: ['organizations'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'timeline-stream', title: 'Timeline', icon: 'tabler:timeline-event', viewScope: ['timeline'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true },
  { id: 'telegram-messages', title: 'Messages', icon: 'tabler:brand-telegram', viewScope: ['communications', 'telegram'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true },
  { id: 'whatsapp-messages', title: 'Messages', icon: 'tabler:brand-whatsapp', viewScope: ['communications', 'whatsapp'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true }
]

function getWidgetsForView(viewId: string, setting: LayoutSettings): ResolvedWidget[] {
  const override = setting.views[viewId]
  const hiddenIds = new Set(override?.hiddenWidgetIds ?? [])
  const gridOverrides = override?.gridOverrides ?? {}
  const panelOverrides = override?.panelSurfaceOverrides ?? {}

  const widgets: ResolvedWidget[] = []
  const viewWidgets = defaultWidgets.filter((w) => w.viewScope.includes(viewId))

  for (let i = 0; i < viewWidgets.length; i++) {
    const def = viewWidgets[i]
    if (hiddenIds.has(def.id)) continue

    const gridOverride = gridOverrides[def.id]

    widgets.push({
      id: def.id,
      title: def.title,
      icon: def.icon,
      columns: gridOverride?.columns ?? def.defaultColumns,
      rows: gridOverride?.rows ?? def.defaultRows,
      minColumns: def.minColumns,
      minRows: def.minRows,
      canAdd: def.canAdd,
      removable: def.removable,
      panelOpacity: panelOverrides[def.id]?.panelOpacity ?? 60,
      panelBlur: panelOverrides[def.id]?.panelBlur ?? 12,
      zone: 'main',
      order: i
    })
  }

  return widgets
}

export const useLayoutEditorStore = defineStore('layoutEditor', () => {
  const layoutSettings = ref<LayoutSettings>(defaultLayoutSettings())
  const layoutDraft = ref<LayoutSettings | null>(null)
  const isLayoutEditing = ref(false)
  const isWidgetDrawerOpen = ref(false)
  const selectedLayoutWidgetId = ref<string | null>(null)
  const pendingNotificationTarget = ref<string | null>(null)

  const effectiveLayoutSettings = computed<LayoutSettings>(() => {
    return layoutDraft.value ?? layoutSettings.value
  })

  const currentView = ref<string>('home')

  const activeWidgets = computed<ResolvedWidget[]>(() => {
    return getWidgetsForView(currentView.value, effectiveLayoutSettings.value)
  })

  const activeWidgetById = computed<Map<string, ResolvedWidget>>(() => {
    const map = new Map<string, ResolvedWidget>()
    for (const widget of activeWidgets.value) {
      map.set(widget.id, widget)
    }
    return map
  })

  const visibleWidgetIds = computed<Set<string>>(() => {
    return new Set(activeWidgets.value.map((w) => w.id))
  })

  const addableWidgetsForCurrentView = computed<WidgetDefinition[]>(() => {
    const visibleIds = visibleWidgetIds.value
    return defaultWidgets
      .filter((w) => w.viewScope.includes(currentView.value))
      .filter((w) => w.canAdd && !visibleIds.has(w.id))
  })

  function setLayoutSettings(settings: LayoutSettings): void {
    layoutSettings.value = settings
  }

  function startLayoutEditing(): void {
    layoutDraft.value = JSON.parse(JSON.stringify(layoutSettings.value))
    isLayoutEditing.value = true
  }

  function cancelLayoutEditing(): void {
    layoutDraft.value = null
    isLayoutEditing.value = false
    selectedLayoutWidgetId.value = null
  }

  function saveLayoutSettings(): void {
    if (layoutDraft.value) {
      layoutSettings.value = JSON.parse(JSON.stringify(layoutDraft.value))
      layoutDraft.value = null
    }
    isLayoutEditing.value = false
    selectedLayoutWidgetId.value = null
  }

  function openAddWidgetDrawer(): void {
    isWidgetDrawerOpen.value = true
  }

  function closeAddWidgetDrawer(): void {
    isWidgetDrawerOpen.value = false
  }

  function openWidgetSettingsDrawer(widgetId: string): void {
    selectedLayoutWidgetId.value = widgetId
  }

  function closeWidgetSettingsDrawer(): void {
    selectedLayoutWidgetId.value = null
  }

  function isWidgetVisible(widgetId: string): boolean {
    return visibleWidgetIds.value.has(widgetId)
  }

  function hideWidget(widgetId: string): void {
    updateCurrentViewOverride((override) => ({
      ...override,
      hiddenWidgetIds: [...override.hiddenWidgetIds, widgetId]
    }))
  }

  function showWidget(widgetId: string): void {
    updateCurrentViewOverride((override) => ({
      ...override,
      hiddenWidgetIds: override.hiddenWidgetIds.filter((id) => id !== widgetId)
    }))
  }

  function resetCurrentViewLayout(): void {
    updateCurrentViewOverride(() => ({
      hiddenWidgetIds: [],
      zoneOverrides: {},
      orderOverrides: {},
      gridOverrides: {},
      panelSurfaceOverrides: {}
    }))
  }

  function setWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, value: number): void {
    updateCurrentViewOverride((override) => ({
      ...override,
      gridOverrides: {
        ...override.gridOverrides,
        [widgetId]: {
          ...(override.gridOverrides[widgetId] ?? { columns: 6, rows: 4 }),
          [dimension]: value
        }
      }
    }))
  }

  function normalizeWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, value: number): number {
    const def = defaultWidgets.find((w) => w.id === widgetId)
    if (!def) return value
    const min = dimension === 'columns' ? def.minColumns : def.minRows
    const max = dimension === 'columns' ? 12 : 8
    return Math.max(min, Math.min(max, Math.round(value)))
  }

  function adjustWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, delta: number): void {
    const current = activeWidgetById.value.get(widgetId)
    if (!current) return
    const currentVal = dimension === 'columns' ? current.columns : current.rows
    const newVal = normalizeWidgetGridValue(widgetId, dimension, currentVal + delta)
    setWidgetGridValue(widgetId, dimension, newVal)
  }

  function handleWidgetGridInput(widgetId: string, dimension: WidgetGridDimension, event: Event): void {
    const target = event.target as HTMLInputElement
    const value = parseInt(target.value, 10)
    if (isNaN(value)) return
    const normalized = normalizeWidgetGridValue(widgetId, dimension, value)
    setWidgetGridValue(widgetId, dimension, normalized)
  }

  function setWidgetPanelSurfaceValue(widgetId: string, setting: WidgetPanelSurfaceSetting, value: number): void {
    updateCurrentViewOverride((override) => ({
      ...override,
      panelSurfaceOverrides: {
        ...override.panelSurfaceOverrides,
        [widgetId]: {
          panelOpacity: override.panelSurfaceOverrides[widgetId]?.panelOpacity ?? 60,
          panelBlur: override.panelSurfaceOverrides[widgetId]?.panelBlur ?? 12,
          [setting]: value
        }
      }
    }))
  }

  function handleWidgetPanelSurfaceInput(widgetId: string, setting: WidgetPanelSurfaceSetting, event: Event): void {
    const target = event.target as HTMLInputElement
    const value = parseInt(target.value, 10)
    if (isNaN(value)) return
    const stepped = Math.round(value / 10) * 10
    const clamped = Math.max(40, Math.min(100, stepped))
    setWidgetPanelSurfaceValue(widgetId, setting, clamped)
  }

  function resetWidgetPanelSurface(widgetId: string): void {
    updateCurrentViewOverride((override) => {
      const surfaces = { ...override.panelSurfaceOverrides }
      delete surfaces[widgetId]
      return { ...override, panelSurfaceOverrides: surfaces }
    })
  }

  function resetWidgetGrid(widgetId: string): void {
    updateCurrentViewOverride((override) => {
      const grids = { ...override.gridOverrides }
      delete grids[widgetId]
      return { ...override, gridOverrides: grids }
    })
  }

  function moveWidgetInZone(widgetId: string, direction: -1 | 1): void {
    updateCurrentViewOverride((override) => {
      const orderOverrides = { ...override.orderOverrides }
      const currentOrder = (orderOverrides[widgetId] as number) ?? 0
      orderOverrides[widgetId] = currentOrder + direction
      return { ...override, orderOverrides }
    })
  }

  const selectedLayoutWidget = computed<ResolvedWidget | null>(() => {
    if (!selectedLayoutWidgetId.value) return null
    return activeWidgetById.value.get(selectedLayoutWidgetId.value) ?? null
  })

  function widgetGridMin(widgetId: string, dimension: WidgetGridDimension): number {
    const def = defaultWidgets.find((w) => w.id === widgetId)
    if (!def) return 2
    return dimension === 'columns' ? def.minColumns : def.minRows
  }

  function widgetGridMax(widgetId: string, dimension: WidgetGridDimension): number {
    return dimension === 'columns' ? 12 : 8
  }

  function widgetPanelSurfaceValue(widgetId: string, setting: WidgetPanelSurfaceSetting): number {
    const widget = activeWidgetById.value.get(widgetId)
    if (!widget) return setting === 'panelOpacity' ? 60 : 12
    return setting === 'panelOpacity' ? widget.panelOpacity : widget.panelBlur
  }

  function updateCurrentViewOverride(update: (override: ViewLayoutOverride) => ViewLayoutOverride): void {
    const draft = layoutDraft.value
    if (!draft) return

    const viewId = currentView.value
    const currentOverride = draft.views[viewId] ?? {
      hiddenWidgetIds: [],
      zoneOverrides: {},
      orderOverrides: {},
      gridOverrides: {},
      panelSurfaceOverrides: {}
    }

    draft.views[viewId] = update(currentOverride)
  }

  function setCurrentView(viewId: string): void {
    currentView.value = viewId
  }

  return {
    layoutSettings,
    layoutDraft,
    effectiveLayoutSettings,
    isLayoutEditing,
    isWidgetDrawerOpen,
    selectedLayoutWidgetId,
    selectedLayoutWidget,
    activeWidgets,
    activeWidgetById,
    visibleWidgetIds,
    addableWidgetsForCurrentView,
    setLayoutSettings,
    startLayoutEditing,
    cancelLayoutEditing,
    saveLayoutSettings,
    openAddWidgetDrawer,
    closeAddWidgetDrawer,
    openWidgetSettingsDrawer,
    closeWidgetSettingsDrawer,
    isWidgetVisible,
    hideWidget,
    showWidget,
    resetCurrentViewLayout,
    setWidgetGridValue,
    normalizeWidgetGridValue,
    adjustWidgetGridValue,
    handleWidgetGridInput,
    setWidgetPanelSurfaceValue,
    handleWidgetPanelSurfaceInput,
    resetWidgetPanelSurface,
    resetWidgetGrid,
    moveWidgetInZone,
    widgetGridMin,
    widgetGridMax,
    widgetPanelSurfaceValue,
    setCurrentView
  }
})
