# Инвентаризация UI States и Component Size

> Создано: 2026-06-14 в рамках Phase 1 (Foundation & Safety Net)
> Цель: Задокументировать какие компоненты имеют Loading/Empty/Error/Skeleton states и превышают лимиты по размеру

## 1. Component Size Inventory

Правило: компоненты >500 строк подлежат рефакторингу.

### Превышают лимит (требуют рефакторинга в Phase 5)

| Компонент | Строк | Файл | Статус |
|-----------|-------|------|--------|
| `CommunicationsPage.vue` | 891 | [`frontend/src/domains/communications/views/CommunicationsPage.vue`](../../frontend/src/domains/communications/views/CommunicationsPage.vue) | **GOD COMPONENT** |
| `CommunicationsConversationList.vue` | ~250+ (предположительно) | [`frontend/src/domains/communications/components/CommunicationsConversationList.vue`](../../frontend/src/domains/communications/components/CommunicationsConversationList.vue) | Проверить |
| `CommunicationsContextInspector.vue` | ~200+ | [`frontend/src/domains/communications/components/CommunicationsContextInspector.vue`](../../frontend/src/domains/communications/components/CommunicationsContextInspector.vue) | Проверить |

### CommunicationsPage.vue — разбивка по ответственности (891 lines)

| Раздел | Строки | % | Описание |
|--------|--------|---|----------|
| Imports (script setup) | 1-86 | 9.7% | 85 импортов из 10+ модулей |
| State declarations | 90-94 | 0.6% | refs для UI состояния |
| TanStack Query hooks | 96-150 | 6.2% | 7 useQuery + 3 useMutation |
| Computed properties | 152-163 | 1.3% | 8 computed |
| Watchers | 166-189 | 2.7% | 7 watch для синхронизации Query→Store |
| Message interaction handlers | 191-450 | 29.2% | 20+ handler functions |
| `onMounted` | ~450-470 | 2.2% | Инициализация |
| Template | ~470-891 | 47.3% | HTML template |
| `<style>` | (встроенный) | - | Scoped styles |

**Вывод:** CommunicationsPage.vue — классический God Component. Содержит 7 TanStack Query hooks, 3 mutation hooks, 20+ обработчиков, watchers для синхронизации Query→Store (вместо прямого использования TanStack Query), и ~420 строк шаблона. Требует разбивки на:
- Отдельные composables для Query-логики (уже есть в `queries/`)
- Page layout component
- Отдельные компоненты для ActionBar, MailListSection, ViewerPanel
- Прямое использование TanStack Query в дочерних компонентах вместо watch→store

## 2. UI States Inventory

Легенда:
- ✅ = реализован
- ❌ = отсутствует
- ⚠️ = частично реализован
- N/A = не применимо

### 2.1 Communications Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| `CommunicationsPage.vue` | ❌ (isMailListLoading не используется в template) | ✅ (CommunicationsEmptyPage) | ❌ | ❌ | ❌ |
| `CommunicationsConversationList.vue` | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| `MailList.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `MailViewer.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `ComposeDrawer.vue` | ⚠️ | N/A | ⚠️ | ❌ | ❌ |
| `DraftStrip.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `HealthStrip.vue` | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| `CommunicationsContextInspector.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `CommunicationsContextRail.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.2 Personas Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Personas page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Identity section | ❌ | ❌ | ❌ | ❌ | ❌ |
| Intelligence section | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.3 Calendar Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Calendar page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Event list | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.4 Tasks Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Tasks page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Task detail | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.5 Knowledge Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Knowledge page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Graph view | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.6 Settings Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Settings page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Account setup | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.7 Telegram Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Telegram page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Chat list | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.8 WhatsApp Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| WhatsApp page | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.9 Shared UI Components (базовые)

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| `Skeleton.vue` | ✅ | N/A | N/A | ✅ | N/A |
| `Toast.vue` | N/A | N/A | ✅ | N/A | ✅ |
| `Button.vue` | ⚠️ (disabled state) | N/A | N/A | N/A | ✅ |

## 3. Итог по UI States

**Критический вывод:** Ни один domain-level компонент не имеет полного набора Loading/Empty/Error/Skeleton состояний. Только базовые UI компоненты (`Skeleton.vue`, `Toast.vue`) реализуют отдельные состояния.

**Приоритет для Phase 5 (God Component Refactoring):**
1. CommunicationsPage.vue — разбить на компоненты, добавить Skeleton/Error/Empty/Loading
2. Добавить Skeleton.vue во все списки (MailList, ConversationList, TaskList, EventList)
3. Добавить отображение ошибок (через Toast.vue + inline error banners)
4. Empty states для всех списков
5. Анимации переходов (FadeTransition, SlideTransition уже есть в shared)

## 4. Missing Stores Inventory

| Store | Файл | Статус |
|-------|------|--------|
| Personas | `frontend/src/domains/personas/stores/` | ❌ Отсутствует |
| WhatsApp | `frontend/src/domains/whatsapp/stores/` | ❌ Отсутствует |
| Organizations | `frontend/src/domains/organizations/stores/` | ❌ Отсутствует |
| Documents | `frontend/src/domains/documents/stores/` | ❌ Отсутствует |
| Notes | `frontend/src/domains/notes/stores/` | ❌ Отсутствует |
| Communications | `frontend/src/domains/communications/stores/communications.ts` | ✅ Существует |
| Telegram | `frontend/src/domains/telegram/stores/telegram.ts` | ✅ Существует |
| Knowledge | `frontend/src/domains/knowledge/stores/knowledge.ts` | ✅ Существует |
| Review | `frontend/src/domains/review/stores/review.ts` | ✅ Существует |
| Tasks | `frontend/src/domains/tasks/stores/tasks.ts` | ✅ Существует |
| Calendar | `frontend/src/domains/calendar/stores/calendar.ts` | ✅ Существует |

## 5. Cross-Domain Import Dependencies

| Source | Target | Файл |
|--------|--------|------|
| `personas/api/personas.ts` | Organizations API | [`frontend/src/domains/personas/api/personas.ts`](../../frontend/src/domains/personas/api/personas.ts) |
| `review/stores/review.ts` | `personas/api/personas` | [`frontend/src/domains/review/stores/review.ts`](../../frontend/src/domains/review/stores/review.ts) |
| `review/stores/review.ts` | `tasks/api/tasks` | same file |
| `review/stores/review.ts` | `knowledge/api/knowledge` | same file |
| `organizations/queries/` | `personas/api/personas` | [`frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`](../../frontend/src/domains/organizations/queries/useOrganizationsQuery.ts) |
