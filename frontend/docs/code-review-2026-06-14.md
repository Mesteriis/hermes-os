# Полное ревью фронтенда Hermes Hub

**Дата:** 2026-06-14  
**Объём:** 80+ файлов прочитано (конфигурация, API, типы, эндпоинты, сервисы, стор-файлы, страницы, тесты, i18n, стили)  
**Вердикт:** Фундамент качественный, но есть критические проблемы, которые нужно исправить перед расширением.

---

## Дополнение: ревью текущего AI Control Center diff

**Объём:** backend AI Control Center readiness guard, provider split, Testcontainers testkit retry, frontend AI Settings selector.  
**ADR:** ADR-0082, ADR-0076.

### Major: API key можно сохранить для не-API provider

**Статус:** Исправлено в текущем diff. `api_key` теперь отклоняется для не-API providers на create/update boundary, а `bind_api_key_secret` дополнительно проверяет `provider_kind = 'api'` перед записью binding.

**Файлы:** `backend/src/ai/api/control_center.rs`, `backend/src/ai/control_center/providers/secrets.rs`, `backend/src/ai/control_center/vault.rs`

`post_ai_provider` и `patch_ai_provider` принимают `api_key` независимо от `provider_kind` и после create/update вызывают `store_api_key_in_host_vault(...)`. `bind_api_key_secret` проверяет только то, что `secret_ref` указывает на `host_vault` + `api_token`, но не проверяет, что целевой provider существует как `provider_kind = 'api'`.

**Impact:** `built_in` или `cli` provider может получить host-vault API-key secret и запись в `ai_provider_secret_refs`. Это нарушает ADR-0082 provider model, загрязняет host-vault metadata и создает misleading setup/status state.

**Fix:** reject `api_key` для `provider_kind != 'api'` в request/API boundary и продублировать invariant в `bind_api_key_secret` через загрузку provider перед записью binding.

### Warning: create-with-api-key оставляет частично созданный provider при сбое host vault

**Статус:** Исправлено в текущем diff. Create/patch с `api_key` теперь выполняют preflight host-vault unlock check до provider mutation; regression test проверяет, что locked/uninitialized vault возвращает `host_vault_error` и не оставляет provider row.

**Файлы:** `backend/src/ai/api/control_center.rs`, `backend/src/ai/control_center/vault.rs`

Provider создается до записи секрета в host vault. Затем `store_api_key_in_host_vault` сначала upsert-ит `secret_references`, потом пишет payload в host vault, потом bind-ит `ai_provider_secret_refs`. Если host vault write или binding падает, API request возвращает ошибку, но provider row уже остается; при повторном create с тем же `provider_kind/provider_key` возможен duplicate/unique conflict вместо идемпотентного восстановления setup flow.

**Impact:** UI может считать создание неуспешным, но backend уже содержит `needs_setup` provider и, в части failure modes, orphan `secret_references` metadata. Это операционный edge case вокруг credential setup.

**Fix:** сделать credential setup явным idempotent second step либо добавить compensating cleanup для provider/secret metadata при failure до binding. DB-части стоит держать в транзакции, но сам host vault write останется внешней side effect boundary.

### Minor: consent endpoint меняет consent_state у provider, которым consent не нужен

**Статус:** Исправлено в текущем diff. `record_consent` теперь загружает provider и разрешает mutation только для `provider_kind = 'api'`; non-API providers сохраняют `consent_state = not_required`.

**Файл:** `backend/src/ai/control_center/providers/consent.rs`

`record_consent` напрямую обновляет `consent_state` для любого provider. Для `built_in` и `cli` provider корректное состояние по текущей модели - `not_required`, но endpoint может записать `granted` или `revoked`.

**Impact:** readiness guard для local/cli provider это сейчас не ломает, но durable state начинает противоречить provider kind semantics и может сбивать будущие UI/reporting checks.

**Fix:** загрузить provider перед update и разрешать consent mutation только для `provider_kind = 'api'`; для остальных возвращать domain error или no-op с текущим provider.

### Review note

SRP split `backend/src/ai/control_center/providers.rs` сам по себе выглядит механическим и bounded: facade сокращен до module declarations, focused provider modules остаются меньше 120 строк, а route/prompt/runtime readiness checks используют общий availability guard. Blocking regressions в этом фрагменте review не найдено.

---

## 🔴 CRITICAL

### 1. `en.json` полностью пуст

**Файл:** [`frontend/src/lib/i18n/en.json`](frontend/src/lib/i18n/en.json:1)

`en.json` содержит `{}`. Все английские `t()`-вызовы возвращают сырой ключ перевода.

**Риск:** При первом запуске с `navigator.language = 'en'` или при сбое определения локали пользователь видит технические идентификаторы (`sidebar.home`, `settings.appearance`) вместо человекочитаемого текста.

**Рекомендация:** Скопировать `ru.json` как baseline для `en.json`, затем перевести на английский. Либо — сделать английский primary-словарём (ключи = английские строки), а русский — наложением.

---

### 2. Module-level side-effect в `config.ts`

**Файл:** [`frontend/src/lib/config.ts`](frontend/src/lib/config.ts)

```ts
export const apiBaseUrl: string = (import.meta.env.VITE_API_BASE_URL as string) || 'http://localhost:8081';
export const apiSecret: string = (import.meta.env.VITE_HERMES_LOCAL_API_SECRET as string) || 'change-me-local-api-secret';
ApiClient.init(apiBaseUrl, apiSecret);
```

`ApiClient.init()` вызывается на уровне модуля, а не при старте приложения. Любой `import '$lib/config'` (напр. через `+layout.ts`) сразу инициализирует синглтон с переменными окружения или fallback-значениями.

**Риски:**
- В production при забытой `VITE_HERMES_LOCAL_API_SECRET` используется fallback `change-me-local-api-secret` — **дыра в безопасности**.
- При тестировании каждый импорт config.ts переинициализирует клиент. Нет `reset()` или `destroy()`.
- Невозможно иметь разные инстансы ApiClient для разных окружений.

**Рекомендация:** Заменить на явный вызов `init()` в `+layout.ts` или в корневом `+page.svelte`, добавить проверку, что секрет — не fallback:

```ts
export function initializeApp(): void {
  const baseUrl = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8081';
  const secret = import.meta.env.VITE_HERMES_LOCAL_API_SECRET;
  if (!secret || secret === 'change-me-local-api-secret') {
    console.error('CRITICAL: HERMES_LOCAL_API_SECRET is not set or is fallback value');
  }
  ApiClient.init(baseUrl, secret);
}
```

---

### 3. Пароли учётных записей в URL-encoded виде в типе ProviderAccount

**Файл:** [`frontend/src/lib/api/types/accounts.ts`](frontend/src/lib/api/types/accounts.ts)

`ProviderAccount.client_app_password` опционально содержит URL-encoded mailbox password.

**Риск:** Если этот тип используется в стор-файлах (communications state), пароль может попасть в localStorage через UI state persistence или в отладчик.

**Рекомендация:** Убедиться, что `client_app_password` исключён из UI state snapshot (через sanitizer в `uiStatePersistence.ts`). Рассмотреть удаление поля из фронтенд-типа полностью — пароль не нужен на клиенте.

---

## 🟡 WARNING

### 4. Огромный тип-файл `mail.ts` (619 строк)

**Файл:** [`frontend/src/lib/api/types/mail.ts`](frontend/src/lib/api/types/mail.ts:1)

50+ типов/интерфейсов в одном файле: CommunicationMessageSummaryV2, MailSyncSettings, EmailThread, EmailRule, EmailTemplate, InvoiceRecord, LegalDocument, CertificateRecord, RichTemplate, MailResourceSnapshot и т.д.

**Рекомендация:** Разделить на:
- `mail-message.ts` — CommunicationMessageSummaryV2, MailMessageDetailItemV2, MailMessageDetailResponse
- `mail-sync.ts` — MailSyncSettings, MailSyncStatus, MailSyncRunResponse
- `mail-thread.ts` — EmailThread, ThreadMessage
- `mail-actions.ts` — SendEmailRequest/Response, MessageExportResponse, WorkflowActionRequest/Response
- `mail-ai.ts` — MessageAnalyzeResponse, AiReplyResponse, TranslationResponse
- `mail-subscriptions.ts` — SubscriptionSource, DuplicateAttachmentGroup
- `mail-finance.ts` — InvoiceRecord, LegalDocument, CertificateRecord
- `mail-template.ts` — EmailTemplate, RichTemplate, RenderTemplateResponse
- `mail-health.ts` — MailboxHealth, SenderStats, MailArchitectureBlocker

---

### 5. Огромный endpoint-файл `communications.ts` (~50 функций)

**Файл:** [`frontend/src/lib/api/endpoints/communications.ts`](frontend/src/lib/api/endpoints/communications.ts:1)

~50 экспортируемых async-функций в одном файле: от `fetchCommunicationMessages` до `fetchMailBlockers`.

**Рекомендация:** Разделить по той же логике, что и типы: messages, sync, drafts, send, threads, health, subscriptions, invoices, legal, templates.

---

### 6. SPA-роутинг через `{#if}` / `{:else}` цепочку

**Файл:** [`frontend/src/routes/+page.svelte`](frontend/src/routes/+page.svelte)

Переключение между 17 вьюхами происходит через:

```svelte
{#if $activeWorkspaceView === 'home'}
  <HomePage ... />
{:else if $activeWorkspaceView === 'communications'}
  <CommunicationsPage ... />
{:else if ...}
```

**Риски:**
- Все страницы существуют в памяти одновременно? Если Svelte не разрушает блоки при переключении — утечка.
- Нет code-splitting — весь фронтенд загружается одним бандлом.
- Нет нативной SvelteKit route-based code splitting.
- Не используются `{#key}` блоки для разрушения/создания компонентов при переключении.

**Рекомендация:**
- Добавить `{#key $activeWorkspaceView}` вокруг цепочки, чтобы Svelte разрушал неактивные компоненты.
- Рассмотреть динамические импорты для code-splitting: `{@await import('./pages/...')}`.
- При переходе на SvelteKit routing — использовать динамические роуты.

---

### 7. 40+ writable-сторов для communications state

**Файл:** [`frontend/src/lib/stores/communications/state.ts`](frontend/src/lib/stores/communications/state.ts:1)

```ts
export const communicationMessages = writable<CommunicationMessageSummaryV2[]>([]);
export const communicationDetail = writable<CommunicationMessageDetail | null>(null);
export const communicationLoading = writable(false);
export const communicationError = writable('');
export const selectedCommunicationMessageId = writable<string | null>(null);
// ... ещё 40 сторов
```

**Риски:**
- 40+ независимых `writable` — нет атомарности обновлений.
- На каждое обновление срабатывают подписки на каждый стор по отдельности.
- При инициализации воркспейса нужно последовательно обновить 10+ сторов — возможны промежуточные состояния.

**Рекомендация:**
- Сгруппировать в единый `communicationsState` стор с типизированным объектом.
- Использовать `immer`-подобный подход или Svelte 5 `$state` с глубоким реактивным доступом.
- Обновлять пачкой: `communicationsState.set({ messages, detail, loading, error })`.

---

### 8. Только 8 тестовых файлов

**Найдено:**
| Файл | Строк |
|------|-------|
| `api/bootstrap.test.ts` | 9 |
| `services/communications.test.ts` | 443 |
| `services/ai.test.ts` | 46 |
| `services/aiSettings.test.ts` | 159 |
| `services/contradictions.test.ts` | 87 |
| `services/obligations.test.ts` | 89 |
| `services/relationships.test.ts` | 95 |
| `services/review.test.ts` | 144 |

**Не покрыто тестами:** все сервисы (accounts, documents, graph, persons, tasks, telegram, uiStatePersistence, vault, whatsapp), все стори (navigation, vault, layoutEditor, sidebar, theme, uiState, aiSettings, settings, notifications, accountWizard), все страницы и компоненты.

**Рекомендация:** Приоритет тестов по риску:
1. `layoutEditor.ts` (555 строк, сложная логика сетки)
2. `uiStatePersistence.ts` (301 строка, sanitization, schema versioning)
3. `graph.ts` (sequence detection, stale response handling)
4. `persons.ts` (identity merge/split, dossier assembly)
5. `vault.ts` (entropy collection, crypto boundary)
6. `sidebar.ts` (сложная логика групп/перемещения)

---

### 9. ApiClient — нет таймаута, retry, AbortController

**Файл:** [`frontend/src/lib/api/client.ts`](frontend/src/lib/api/client.ts)

```ts
const response = await fetch(url, { method, headers, body });
```

- Нет `AbortController` — запросы не отменяются при уходе со страницы.
- Нет таймаута — зависший бэкенд вешает UI.
- Нет ретрая — временные сетевые ошибки приводят к показу ошибки пользователю.

**Рекомендация:** Добавить middleware-слой:

```ts
type ApiMiddleware = (req: RequestInit & { url: string }) => Promise<Response>;

class ApiClient {
  private middlewares: ApiMiddleware[] = [];
  use(mw: ApiMiddleware): void { this.middlewares.push(mw); }
}
```

И встроенные middlewares: `timeout(10_000)`, `retry(3)`, `abortOnNavigate()`.

---

### 10. `as T` в каждом методе API

**Файл:** [`frontend/src/lib/api/client.ts`](frontend/src/lib/api/client.ts)

```ts
return (await response.json()) as T;
```

Нет runtime-валидации ответа от бэкенда. Если бэкенд меняет схему — фронтенд молча получает `undefined` и разбирается с последствиями в рантайме.

**Рекомендация:**
- Добавить Zod-схемы для критических ответов.
- Или хотя бы базовый type guard: `if (!response.ok) throw new ApiError(response.status, body)`.
- Использовать discriminated union для ответов: `{ data: T } | { error: string }`.

---

### 11. `$derived.by` для вычислимых полей на массивах без мемоизации

**Файл:** [`frontend/src/lib/pages/home/HomePage.svelte`](frontend/src/lib/pages/home/HomePage.svelte:1)

```ts
let stats = $derived.by(() => {
  const msgs = $communicationMessages;
  return {
    total: msgs.length,
    unread: msgs.filter(m => !m.isRead).length,
    // ...
  };
});
```

Вызов `filter` на каждой итерации цикла обновления. При 1000+ сообщениях — заметно.

**Рекомендация:** Использовать `$derived` с вычислителями или выделить `unreadMessages = $derived(msgs.filter(m => !m.isRead))`.

---

### 12. Жёстко закодированные русские sample-данные

**Файл:** [`frontend/src/routes/+page.svelte`](frontend/src/routes/+page.svelte)

Массив `notes` содержит русские строки-примеры. Это выглядит как демо-данные, которые могут попасть в production.

**Рекомендация:** Вынести в константы или удалить перед production.

---

## 🟢 POSITIVE

### 13. Качественная архитектура i18n

**Файл:** [`frontend/src/lib/i18n/ru.json`](frontend/src/lib/i18n/ru.json:1) — 877 строк, все строки интерфейса переведены на русский. Используется простой `dictionary[key] ?? key` подход с writable store + derived store для реактивности. Минимально, предсказуемо, легко тестировать.

### 14. Чистое разделение stores/services

Stores (`stores/`) владеют реактивным состоянием. Services (`services/`) владеют бизнес-логикой и HTTP-вызовами. Чёткое разделение: стор вызывает сервис, сервис не знает о сторах. Barrel-экспорты (`index.ts`) скрывают внутреннюю структуру.

### 15. Надёжная HTML-санитизация писем

**Файл:** [`frontend/src/lib/services/communications/rendering.ts`](frontend/src/lib/services/communications/rendering.ts)

- Стриппинг `<style>`, `<script>`, `onclick`, `onload` и др.
- Прокси для удалённых изображений (`remoteMailImageProxyUrl`)
- Удаление HTML-спейсеров и пустых image-ссылок
- Декодинг HTML-entities
- Сохранение rich link labels вместо трекинг-ссылок

### 16. UI State Persistence с dual storage

**Файл:** [`frontend/src/lib/services/uiStatePersistence.ts`](frontend/src/lib/services/uiStatePersistence.ts:1) — 300 строк грамотного кода:
- Dual localStorage + backend persist
- Schema versioning (V1)
- TTL enforcement (24h для compose)
- Null-safe sanitizers для каждого поля
- Debounced запись (450ms UI, 900ms compose)
- Тестовый teardown (`resetUiStatePersistenceForTests`)

### 17. Design tokens через CSS custom properties

**Файл:** [`frontend/src/lib/styles/app.css`](frontend/src/lib/styles/app.css)

Все `--hh-*` переменные: `--hh-color-bg`, `--hh-color-text`, `--hh-radius-sm`, `--hh-space-xs`, и т.д. Через `app.css` + `theme.ts` с `shellThemeClass`.

### 18. Custom linting with inline style detection

**Файл:** [`frontend/scripts/check-no-inline-styles.mjs`](frontend/scripts/check-no-inline-styles.mjs)

Регулярка ищет `style="` и `<style>` — и пайплайн это режет. Это архитектурное решение (а не баг): все стили через классы и `--hh-*` переменные.

### 19. Widget grid layout system

**Файл:** [`frontend/src/lib/stores/layoutEditor.ts`](frontend/src/lib/stores/layoutEditor.ts:1) — 555 строк продуманной логики:
- Resolved/override/active layout separation
- Content clipping detection через `requestAnimationFrame`
- Panel surface controls (opacity/blur/brightness)
- Widget grid reset/move/hide/show
- Add widget drawer

### 20. Vault onboarding wizard

**Файл:** [`frontend/src/lib/services/vault.ts`](frontend/src/lib/services/vault.ts:1)

Чёткая последовательность: entropy collection (velocity/acceleration с mouse/touch) → flush (batch 100) → create → biometric → recovery export. Состояние из 4 шагов.

### 21. Комплексное тестирование communications сервиса

**Файл:** [`frontend/src/lib/services/communications.test.ts`](frontend/src/lib/services/communications.test.ts:1) — 443 строки, 21 тест-кейс. Покрывает: фильтрацию, send capability, compose, HTML sanitization, image proxy, workflow actions, draft CRUD, resource summary.

### 22. Review workspace aggregation с partial failure

**Файл:** [`frontend/src/lib/services/review.test.ts`](frontend/src/lib/services/review.test.ts:1) — корректная обработка частичных ошибок при агрегации данных из 4 источников (relationships, decisions, obligations, contradictions).

---

## 📊 Статистика

| Метрика | Значение |
|---------|----------|
| SvelteKit версия | 5.55.2 |
| TypeScript | 6.0.2 |
| Vite | 8.0.7 |
| Vitest | 4.1.8 |
| Tauri | 2.11.2 |
| Runtime зависимостей | 1 (`@iconify/svelte`) |
| Тестовых файлов | 8 |
| Всего строк тестов | ~1072 |
| Строк в i18n/ru.json | 877 |
| Вьюх (AppViewId) | 17 |
| Страниц (page directories) | 19 |
| Сервисных модулей | ~25 |
| Стор-файлов | ~12 |

---

## 🎯 Приоритетные действия

1. **Немедленно:** Заполнить `en.json` или сделать английские строки ключами
2. **Немедленно:** Убрать module-level side-effect из `config.ts`, добавить проверку секрета
3. **Высокий:** Проверить, что пароли не попадают в localStorage через UI persistence
4. **Высокий:** Разделить `mail.ts` (619 строк) и `communications.ts` (~50 функций)
5. **Высокий:** Добавить `{#key $activeWorkspaceView}` в `+page.svelte`
6. **Средний:** Добавить timeout/retry/AbortController в ApiClient
7. **Средний:** Написать тесты для layoutEditor, uiStatePersistence, graph, persons, vault
8. **Средний:** Сократить количество writable stores в communications state
9. **Низкий:** Убрать демо-данные из `+page.svelte`
10. **Низкий:** Добавить Zod-схемы для критических API-ответов
