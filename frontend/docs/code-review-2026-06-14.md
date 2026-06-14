# Полное ревью фронтенда Hermes Hub

**Дата:** 2026-06-14  
**Объём:** 80+ файлов прочитано (конфигурация, API, типы, эндпоинты, сервисы, стор-файлы, страницы, тесты, i18n, стили)  
**Вердикт:** Фундамент качественный, но есть критические проблемы, которые нужно исправить перед расширением.

---

## Дополнение: code review текущей Vue 3 миграции

**Дата проверки:** 2026-06-14 16:47 CEST
**Объём:** новый Vue 3 bootstrap, platform API/SSE/i18n, communications HTML rendering, Calendar/Tasks view data flow, frontend validation gate.
**ADR:** ADR-0093, ADR-0031, ADR-0056, ADR-0077.

### Critical: `ApiClient` не инициализируется перед использованием

**Статус:** Открыто.

**Файлы:** `frontend/src/main.ts`, `frontend/src/platform/api/ApiClient.ts`, `frontend/src/domains/*/api/*.ts`

Новый entrypoint создает Vue app, Pinia, Vue Query and router, но не вызывает `ApiClient.init(...)`. При этом `ApiClient.instance` бросает `ApiClient not initialized. Call ApiClient.init() first.`, а все domain API modules используют именно `ApiClient.instance`.

**Evidence:** `frontend/src/main.ts:8-14`, `frontend/src/platform/api/ApiClient.ts:72-83`, `rg "ApiClient\\.init" frontend/src` находит только сам метод.

**Impact:** любой экран, который загружает backend data через domain API, падает на runtime до network request. `vue-tsc` и `vite build` это не ловят, потому что контракт bootstrap singleton-а не покрыт тестом.

**Fix:** инициализировать `ApiClient` в bootstrap до mount/router-driven queries, используя validated config. Добавить regression test на bootstrap/config path: без secret fail-fast, с secret `ApiClient.instance` доступен.

### Critical: backend URL и secret env names расходятся с Makefile/backend contract

**Статус:** Открыто.

**Файлы:** `frontend/src/config/index.ts`, `Makefile`, `frontend/src-tauri/src/lib.rs`

`frontend-dev` передает `VITE_HERMES_API_BASE_URL` и `VITE_HERMES_LOCAL_API_SECRET`, backend sidecar стартует на `127.0.0.1:8080`, но новый config читает `VITE_API_BASE_URL` / `VITE_HERMES_API_SECRET` и default-ит API base URL в `http://localhost:3000`.

**Evidence:** `frontend/src/config/index.ts:4-13`, `Makefile:277-280`, `frontend/src-tauri/src/lib.rs:53`.

**Impact:** даже после исправления `ApiClient.init` frontend по умолчанию будет ходить не в Hermes backend и отправлять пустой `X-Hermes-Secret`. Protected endpoints по ADR-0056 будут недоступны, а dev loop через `make frontend-dev` не сможет поднять рабочий UI без ручного дублирования env vars.

**Fix:** вернуть единый public env contract: `VITE_HERMES_API_BASE_URL` with default `http://127.0.0.1:8080` and required `VITE_HERMES_LOCAL_API_SECRET`; пустой secret должен быть bootstrap error, а не silently accepted config.

### Critical: renderer писем снова вставляет непроверенный HTML через `innerHTML`

**Статус:** Открыто.

**Файл:** `frontend/src/domains/communications/components/MessageBodyTab.vue`

`sanitizeEmailHtml` удаляет несколько unsafe block tags, но не удаляет event-handler attributes (`onerror`, `onclick`), `javascript:` URLs, dangerous `src`/`href` schemes or inline CSS. Затем результат вставляется через `body.innerHTML`. Plain-text branch внутри shadow renderer также делает `bodyText.value.replace(/\n/g, '<br>')` and assigns it to `innerHTML` without escaping.

**Evidence:** `frontend/src/domains/communications/components/MessageBodyTab.vue:51-70`, `frontend/src/domains/communications/components/MessageBodyTab.vue:91-95`.

**Impact:** импортированное письмо является untrusted input. HTML вроде `<img src=x onerror=...>` или текстовое тело с HTML-like payload может выполнить script/event handler в основном renderer context, а не только в sandbox iframe. Это регрессия относительно mail rendering boundary, где message bodies должны быть sanitized/proxied before display.

**Fix:** не поддерживать sanitizer regex-цепочкой. Перенести проверенную allowlist sanitation из старого renderer или использовать DOM-based sanitizer boundary, удалить unsafe attributes/schemes, escape plain text before `innerHTML`, and cover with regression tests for event handlers, `javascript:` links and malformed HTML.

### Major: default rendered view для HTML-писем пустой

**Статус:** Открыто.

**Файл:** `frontend/src/domains/communications/components/MessageBodyTab.vue`

Для HTML email default path показывает `shadow-frame-wrapper`, но `renderShadowContent()` вызывается только в `onMounted`, когда `!isHtmlEmail`. Для HTML bodies renderer не вызывается ни при mount, ни при смене message, ни при переключении обратно с iframe view.

**Evidence:** `frontend/src/domains/communications/components/MessageBodyTab.vue:98-102`, `frontend/src/domains/communications/components/MessageBodyTab.vue:113-130`.

**Impact:** пользователь выбирает HTML-письмо и видит пустой rendered body до ручного переключения в `Original HTML`; кнопка `Rendered` не восстанавливает rendered content. Это ломает основную Communications surface.

**Fix:** render HTML shadow content on mount and whenever `message/bodyHtml/showOriginalFrame` changes, or remove dead shadow branch and use a single safe renderer path.

### Major: frontend validation gate заменен на сборку и один placeholder test

**Статус:** Открыто.

**Файлы:** `Makefile`, `frontend/package.json`, `frontend/src/__tests__/placeholder.test.ts`, удаленные `frontend/src/lib/**/*.test.ts`

`frontend-check` теперь запускает только `pnpm build`; `package.json` не содержит `check`, `lint:styles` or test-inclusive validation script. Старые API/service/store/layout tests удалены, а новая test suite состоит из одного placeholder test.

**Evidence:** `Makefile:282-292`, `frontend/package.json:7-14`, `frontend/src/__tests__/placeholder.test.ts:1-7`, `git diff --name-status -- frontend/src/lib` показывает удаление старых `.test.ts` files.

**Impact:** migration может пройти `make validate`/`frontend-check` без покрытия bootstrap API initialization, auth config, query invalidation, mail sanitizer, layout state or settings persistence. Это уже проявилось: `pnpm build` проходит при неинициализированном `ApiClient`.

**Fix:** восстановить frontend check как `lint + unit tests + build`; заменить placeholder на regression tests для bootstrap config, `ApiClient`, communications sanitizer/rendering, settings/sidebar persistence and critical query/mutation flows.

### Major: views обходят TanStack Query boundary из ADR-0093

**Статус:** Открыто.

**Файлы:** `frontend/src/domains/calendar/views/CalendarPage.vue`, `frontend/src/domains/tasks/views/TasksPage.vue`, `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md`

ADR-0093 требует, чтобы server-derived state шел через TanStack Query, а direct API calls inside components были запрещены. Сейчас `CalendarPage.vue` напрямую вызывает `fetchCalendarSources`, `fetchWeeklyBrief`, `searchCalendarEvents`, `fetchEventContextPack`, `fetchEventBrief`, `fetchEventAgenda`, `createCalendarEvent`; `TasksPage.vue` напрямую вызывает decisions/obligations/task review APIs from the view.

**Evidence:** `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md:136-143`, `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md:350-353`, `frontend/src/domains/calendar/views/CalendarPage.vue:13-21`, `frontend/src/domains/calendar/views/CalendarPage.vue:56-127`, `frontend/src/domains/tasks/views/TasksPage.vue:6-7`, `frontend/src/domains/tasks/views/TasksPage.vue:27-95`.

**Impact:** server state is split between Query cache, local refs and Pinia. Invalidation becomes manual and easy to miss; loading/error state differs per screen; background refresh/stale-while-revalidate semantics promised by ADR-0093 do not hold.

**Fix:** move these calls into domain query/mutation composables with stable query keys and mutation invalidation. Views should consume composables and keep only transient UI state locally/Pinia.

### Warning: i18n contract drift from ADR-0077

**Статус:** Открыто.

**Файлы:** `frontend/src/platform/i18n/index.ts`, `frontend/src/platform/i18n/en.json`, `docs/adr/ADR-0077-i18n-russian-english.md`

ADR-0077 defines English strings as keys, `en.json` as empty identity fallback, and default locale `en`; the migrated implementation defaults to `ru` and ships a populated `en.json` identity dictionary.

**Evidence:** `docs/adr/ADR-0077-i18n-russian-english.md:16-23`, `frontend/src/platform/i18n/index.ts:10-18`, `wc -l frontend/src/platform/i18n/en.json frontend/src/platform/i18n/ru.json` reports 863 and 877 lines.

**Impact:** not a runtime blocker, but it changes the translation maintenance model without superseding ADR-0077. Future string additions now require two files instead of one English key plus Russian dictionary, and default locale behavior no longer matches the accepted ADR.

**Fix:** either restore ADR-0077 semantics in the Vue implementation or write a superseding ADR for the new default locale and dictionary model.

### Validation notes

- Ran: `pnpm lint:ts` from `frontend/`; result: passed (`vue-tsc --noEmit`).
- Ran: `pnpm test:unit` from `frontend/`; result: passed, but only 1 placeholder test executed.
- Ran: `pnpm build` from `frontend/`; result: passed. Build emitted Rolldown `INVALID_ANNOTATION` warnings from `@vueuse/core` and a chunk-size warning (`index-*.js` 641.10 kB / 186.44 kB gzip).

### Review note

Vue 3 migration itself is consistent with ADR-0093 as a platform decision. The blocking issues are not the framework choice; they are bootstrap/auth wiring, unsafe mail rendering, validation coverage loss and data-flow drift from the new ADR contract.

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

## Дополнение: ревью текущего backend SRP split diff

**Объём:** `backend/src/integrations/omniroute/client.rs`, новые `backend/src/integrations/omniroute/client/*` modules, `backend/src/domains/projects/core/read_model.rs`, новые `backend/src/domains/projects/core/read_model/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0081, ADR-0082, ADR-0047, ADR-0048.

### Major: новые split modules сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает tracked facade changes, но новые module directories остаются `?? backend/src/integrations/omniroute/client/` и `?? backend/src/domains/projects/core/read_model/`.

**Файлы:** `backend/src/integrations/omniroute/client.rs`, `backend/src/domains/projects/core/read_model.rs`, `backend/src/integrations/omniroute/client/*`, `backend/src/domains/projects/core/read_model/*`

Оба facade-файла теперь объявляют child modules через `mod ...;`. Если в PR/commit попадут только tracked facade changes без новых source files, Rust не найдет modules и backend не соберется. Дополнительный риск: `git diff --stat` сейчас показывает только tracked deletions/insertions и не отражает содержимое новых module files, поэтому обычный diff-review легко пропустит самую важную часть split-а.

**Impact:** partial commit/PR ломает compile path для Omniroute и Projects. Это не runtime bug в текущем worktree, потому что файлы на диске есть, но это реальный integration risk до staging.

**Fix:** перед commit/staging добавить все новые files из `backend/src/integrations/omniroute/client/` и `backend/src/domains/projects/core/read_model/` вместе с facade changes. Для review использовать `git diff --stat --cached` после staging или явно проверять untracked files.

### Warning: Project read-model SQL behavior не покрыт дефолтным validation gate

**Статус:** Открытый validation gap. `make backend-validate` запускает `backend-test-unit`, то есть `cargo test --manifest-path backend/Cargo.toml --lib`, а не integration tests. Прямой запуск `cargo test --manifest-path backend/Cargo.toml --test projects -- --nocapture` без `HERMES_TEST_DATABASE_URL` проходит с skipped test bodies: `live_project_context` возвращает `None`.

**Файлы:** `Makefile`, `backend/tests/projects.rs`, `backend/src/domains/projects/core/read_model/*`

Текущий split перемещает SQL-heavy read-model methods: stats, messages, documents, people, timeline и active reviewed targets. Компиляция ловит module/import ошибки, но дефолтная backend validation не исполняет live SQL assertions для ADR-0047/0048 behavior: rejected keyword links excluded, confirmed non-keyword links included, body text not exposed, people/timeline/stats counted from active reviewed links.

**Impact:** будущая регрессия в SQL shape или reviewed-link semantics может пройти `make backend-validate`, если `HERMES_TEST_DATABASE_URL` не задан. Для текущего механического split-а подтвержденных behavior regressions не найдено, но review coverage по Project read model остается слабее, чем по Omniroute fake-server tests.

**Fix:** для этого slice перед merge выполнить Project live gate с реальным PostgreSQL, например через dev DB and `HERMES_TEST_DATABASE_URL`, либо перевести критичные `ProjectStore::project_detail` regression tests на Testcontainers, чтобы они не skip-ались в default validation.

### Review note

Blocking code regressions внутри самих split modules не подтверждены. Omniroute public surface сохранен через `pub use` (`OmniRouteClientConfig`, `OmniRouteError`, `OmniRouteChatResult`, `OmniRouteEmbedResult`), OpenAI-compatible paths и thinking-tag stripping остались в том же behavior boundary. Project read-model SQL bodies и row mappers перенесены без смыслового изменения, helper visibility сужена до `domains::projects::core`, что соответствует текущим callers из `store.rs` и `projection.rs`.

---

## Дополнение: ревью текущего Mail/Persons SRP split diff

**Объём:** `backend/src/domains/mail/core/models.rs`, новые `backend/src/domains/mail/core/models/*` modules, `backend/src/domains/persons/enrichment.rs`, новые `backend/src/domains/persons/enrichment/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0041, ADR-0046, ADR-0055, ADR-0076, ADR-0084, ADR-0086, ADR-0090.

### Major: новые Mail/Persons split modules сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/domains/mail/core/models/` и `?? backend/src/domains/persons/enrichment/` вместе с facade changes.

**Файлы:** `backend/src/domains/mail/core/models.rs`, `backend/src/domains/mail/core/models/*`, `backend/src/domains/persons/enrichment.rs`, `backend/src/domains/persons/enrichment/*`

Оба facade-файла теперь объявляют child modules через `mod ...;`. Если tracked facade changes попадут в commit без новых module files, backend перестанет компилироваться. Это тот же integration risk, что уже зафиксирован для OmniRoute/Project split-а, теперь он распространяется и на Mail core model / Persons enrichment split.

**Impact:** partial commit/PR ломает compile path для mail core models или persons enrichment. В текущем worktree сборка проходит, потому что untracked files физически присутствуют.

**Fix:** перед commit/staging добавить новые split directories вместе с facade files и проверять staged diff, а не только tracked `git diff`.

### Warning: Persona enrichment live SQL ветки не исполняются без `HERMES_TEST_DATABASE_URL`

**Статус:** Открытый validation gap. `cargo test --manifest-path backend/Cargo.toml --test persons -- --nocapture` прошел 23/23, но live PostgreSQL branches явно skipped из-за unset `HERMES_TEST_DATABASE_URL`.

**Файлы:** `backend/tests/persons.rs`, `backend/src/domains/persons/enrichment/*`

Split переносит SQL-heavy behavior: update compatibility `persons` projection, materialize Owner Persona trust Relationship, sync Persona notes memory card и `ui:favorite` preference. Compile/clippy ловят module/import ошибки, но live DB assertions не исполняются в default environment.

**Impact:** для текущего механического split-а code-level regression не подтвержден, но будущая SQL/schema regression в этих ветках может пройти default `make backend-validate`, если не запускать live/Testcontainers-backed coverage.

**Fix:** перед merge выполнить `persons` integration gate с реальным PostgreSQL через `HERMES_TEST_DATABASE_URL` либо перевести критичные enrichment tests на Testcontainers, чтобы они не skip-ались в обычной validation цепочке.

### Review note

Blocking code regressions внутри Mail/Persons split modules не подтверждены. Mail core model public surface сохранен через facade re-exports, provider credential compatibility checks остались account-scoped and secret-kind scoped. Persons enrichment public surface сохранен (`EnrichedPerson`, `PersonEnrichmentStore`, `PersonEnrichmentError`), helper visibility сужена до `domains::persons::enrichment`, а trust/notes/favorite materialization behavior перенесен без расширения Persona compatibility layer.

---

## Дополнение: ревью текущего API Support Stores SRP split diff

**Объём:** `backend/src/domains/api_support/stores.rs`, новые `backend/src/domains/api_support/stores/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0073, ADR-0056, ADR-0082.

### Major: новый API support split directory сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/domains/api_support/stores/` вместе с tracked facade change `M backend/src/domains/api_support/stores.rs`.

**Файлы:** `backend/src/domains/api_support/stores.rs`, `backend/src/domains/api_support/stores/*`

Facade теперь объявляет child modules через `mod ...;`. Если в commit попадет только tracked facade file без новых module files, backend не соберется. Это тот же integration risk, что уже зафиксирован для остальных SRP split-ов.

**Impact:** partial commit/PR ломает compile path для shared API support store helpers, включая event store, graph store, AI runtime service, Telegram/WhatsApp store factories, settings store and account setup service.

**Fix:** перед commit/staging добавить `backend/src/domains/api_support/stores/` вместе с facade changes и проверять staged diff.

### Review note

Blocking code regressions внутри API support stores split modules не подтверждены. Handler-facing API сохранен через `pub(crate) use` facade; `ApiError::DatabaseNotConfigured` boundary сохранен через общий `database_pool` helper; AI runtime routing продолжает использовать readiness-aware model selection from AI Control Center; router-level local API auth не переносился в store helpers и остается вне этого boundary.

---

## Дополнение: ревью текущего Mail Rules SRP split diff

**Объём:** `backend/src/domains/mail/rules.rs`, новые `backend/src/domains/mail/rules/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0041, ADR-0046, ADR-0052, ADR-0055, ADR-0092.

### Major: новый Mail rules split directory сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/domains/mail/rules/` вместе с tracked facade change `M backend/src/domains/mail/rules.rs`.

**Файлы:** `backend/src/domains/mail/rules.rs`, `backend/src/domains/mail/rules/*`

Facade теперь объявляет child modules через `mod ...;`. Если в commit попадет только tracked facade file без новых module files, backend не соберется. Это тот же integration risk, что уже зафиксирован для остальных SRP split-ов.

**Impact:** partial commit/PR ломает compile path для mail rule models, mode parsing, store persistence, condition evaluation, row mapping and validation.

**Fix:** перед commit/staging добавить `backend/src/domains/mail/rules/` вместе с facade changes и проверять staged diff.

### Review note

Blocking code regressions внутри Mail rules split modules не подтверждены. Public surface сохранен через facade re-exports (`EmailRule`, `NewEmailRule`, `RuleMode`, `RuleAction`, `RuleMatchResult`, `EmailRuleStore`, `EmailRuleError`); `RuleMode` string contract не изменен; condition/action parsing behavior подтвержден существующими unit tests. Split не добавляет provider write execution и не обходит ADR-0052 capability boundary.

---

## Дополнение: ревью текущего Ollama Client SRP split diff

**Объём:** `backend/src/integrations/ollama/client.rs`, новые `backend/src/integrations/ollama/client/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0009, ADR-0049, ADR-0081, ADR-0082, ADR-0022.

### Major: новый Ollama client split directory сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/integrations/ollama/client/` вместе с tracked facade change `M backend/src/integrations/ollama/client.rs`.

**Файлы:** `backend/src/integrations/ollama/client.rs`, `backend/src/integrations/ollama/client/*`

Facade теперь объявляет child modules через `mod ...;`. Если в commit попадет только tracked facade file без новых module files, backend не соберется. Это тот же integration risk, что уже зафиксирован для остальных SRP split-ов.

**Impact:** partial commit/PR ломает compile path для local AI runtime client, включая version/tags catalog calls, chat, embeddings, transport response decoding, Qwen thinking-tag sanitization and public result/error DTOs.

**Fix:** перед commit/staging добавить `backend/src/integrations/ollama/client/` вместе с facade changes и проверять staged diff.

### Review note

Blocking code regressions внутри Ollama client split modules не подтверждены. Public surface сохранен через facade re-exports (`OllamaClientConfig`, `OllamaError`, `OllamaChatResult`, `OllamaEmbedResult`) and the existing `OllamaClient` methods. Fake-server tests подтвердили `/api/version`, `/api/tags`, `/api/chat`, `/api/embed`, missing-model handling, malformed JSON handling and Qwen thinking block stripping. Split не меняет local-default posture from ADR-0009 and does not add remote-provider behavior.

---

## Дополнение: ревью текущего WhatsApp Web Store SRP split diff

**Объём:** `backend/src/integrations/whatsapp/client/store.rs`, новые `backend/src/integrations/whatsapp/client/store/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0051, Communications architecture.

### Major: новый WhatsApp store split directory сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/integrations/whatsapp/client/store/` вместе с tracked facade change `M backend/src/integrations/whatsapp/client/store.rs`.

**Файлы:** `backend/src/integrations/whatsapp/client/store.rs`, `backend/src/integrations/whatsapp/client/store/*`

Facade теперь объявляет child modules через `mod ...;`. Если в commit попадет только tracked facade file без новых module files, backend не соберется. Это тот же integration risk, что уже зафиксирован для остальных SRP split-ов.

**Impact:** partial commit/PR ломает compile path для WhatsApp Web fixture account setup, session persistence, fixture ingestion, recent message queries and Decision/Task candidate refresh.

**Fix:** перед commit/staging добавить `backend/src/integrations/whatsapp/client/store/` вместе с facade changes и проверять staged diff.

### Warning: WhatsApp live DB ветки не исполняются без `HERMES_TEST_DATABASE_URL`

**Статус:** Открытый validation gap. `cargo test --manifest-path backend/Cargo.toml --test whatsapp -- --nocapture` прошел 4/4, но live PostgreSQL bodies явно skipped из-за unset `HERMES_TEST_DATABASE_URL`.

**Файлы:** `backend/tests/whatsapp.rs`, `backend/src/integrations/whatsapp/client/store/*`

Split переносит SQL-heavy behavior: session upsert/list/update, raw source ingestion, projection into `communication_messages`, candidate refresh and recent message query. Compile/clippy and non-live provider checks passed, but live DB assertions need a configured database or Testcontainers-backed conversion to execute in default validation.

**Fix:** перед merge выполнить WhatsApp integration gate с реальным PostgreSQL через `HERMES_TEST_DATABASE_URL` либо перевести критичные WhatsApp tests на Testcontainers, чтобы они не skip-ались в обычной validation цепочке.

### Review note

Blocking code regressions внутри WhatsApp store split modules не подтверждены. Public store surface сохранен (`setup_fixture_account`, `upsert_session`, `list_sessions`, `ingest_fixture_message`, `recent_messages`). ADR-0051 posture preserved: fixture/manual companion state remains the implemented path, live runtime/sends are not enabled, raw records remain append-only with provider provenance, and projected messages still use `channel_kind = 'whatsapp_web'`.

---

## Дополнение: ревью текущего Telegram Accounts / Decision Models / Obligation Models SRP split diff

**Объём:** `backend/src/integrations/telegram/client/accounts.rs`, новые `backend/src/integrations/telegram/client/accounts/*` modules, `backend/src/domains/decisions/models.rs`, новые `backend/src/domains/decisions/models/*` modules, `backend/src/domains/obligations/models.rs`, новые `backend/src/domains/obligations/models/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0050, ADR-0083, ADR-0091, ADR-0089, ADR-0088.

### Major: новые Telegram/Decision/Obligation split directories сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/integrations/telegram/client/accounts/`, `?? backend/src/domains/decisions/models/` и `?? backend/src/domains/obligations/models/` вместе с tracked facade changes.

**Файлы:** `backend/src/integrations/telegram/client/accounts.rs`, `backend/src/integrations/telegram/client/accounts/*`, `backend/src/domains/decisions/models.rs`, `backend/src/domains/decisions/models/*`, `backend/src/domains/obligations/models.rs`, `backend/src/domains/obligations/models/*`

Facade files теперь объявляют child modules через `mod ...;`. Если в commit попадут только tracked facade files без новых module directories, backend не соберется. Риск выше обычного, потому что `git diff --stat` по tracked paths показывает только сокращение facade-файлов и не включает содержимое untracked modules.

**Impact:** partial commit/PR ломает compile path для Telegram account setup/lifecycle, Decision model DTOs and validation helpers, Obligation model DTOs and validation helpers.

**Fix:** перед commit/staging добавить все три split directories вместе с facade changes и ревьюить staged diff, а не только обычный tracked `git diff`.

### Major: Telegram live account setup может оставить partial account при сбое host vault

**Статус:** Открыто. Это не новое поведение от split-а, но текущий затронутый код сохраняет risk.

**Файлы:** `backend/src/integrations/telegram/client/accounts/live_setup.rs`, `backend/src/integrations/telegram/client/accounts/credential_bindings.rs`

`setup_live_blocked_account` сначала upsert-ит provider account, затем вызывает `store_live_account_credentials`. Внутри credential path отдельно upsert-ится `secret_references`, затем пишется host-vault payload, затем создается `communication_provider_account_secret_refs` binding. Если host vault locked/uninitialized, filesystem write fails или DB binding падает, API вернет ошибку, но account row уже может остаться в `live_blocked` / `tdlib_qr_authorized` state, а в части failure modes останется и secret reference metadata без usable binding.

**Impact:** UI/runtime могут видеть частично созданный Telegram account после неуспешного setup. Это может давать misleading account status, усложнять retry/idempotency и нарушать ожидание ADR-0050/0076, что live credential setup является account-scoped vault-backed operation, а не наполовину примененной metadata mutation.

**Fix:** сделать host-vault unlock/write preflight до provider-account mutation, либо оформить setup как явный двухфазный flow с `needs_credentials` state и идемпотентным recovery. Если сохраняется single request, DB mutations вокруг account/secret metadata должны иметь compensating cleanup при external vault failure.

### Warning: live DB behavior для Telegram/Decisions/Obligations не покрывается default backend validation

**Статус:** Открытый validation gap. `make backend-validate` запускает `cargo test --manifest-path backend/Cargo.toml --lib`, а live integration tests в `telegram`, `decisions`, `decisions_api`, `obligations`, `obligations_api` и `task_candidates` skip-ают PostgreSQL bodies без `HERMES_TEST_DATABASE_URL`.

**Файлы:** `Makefile`, `backend/tests/telegram.rs`, `backend/tests/decisions.rs`, `backend/tests/decisions_api.rs`, `backend/tests/obligations.rs`, `backend/tests/obligations_api.rs`, `backend/tests/task_candidates.rs`

Для Decision/Obligation model split compile coverage хорошо ловит потерянные re-exports, enum contracts and helper visibility, но source-backed persistence behavior, review routes, graph projections and Telegram candidate refresh assertions остаются live-DB ветками. Без `HERMES_TEST_DATABASE_URL` regression в SQL shape или persistence adapters может пройти default validation.

**Fix:** перед merge выполнить соответствующие integration gates с реальным PostgreSQL (`HERMES_TEST_DATABASE_URL`) либо перевести критичные Decision/Obligation/Telegram candidate-refresh tests на Testcontainers, чтобы они не skip-ались в обычной validation цепочке.

### Review note

Blocking split-induced regressions в Telegram account modules, Decision models и Obligation models не подтверждены. Telegram account methods остались на `TelegramStore`, account-scoped secret purposes and lifecycle semantics сохранены. Decision facade сохраняет public surface (`Decision`, `NewDecision`, evidence/impact DTOs, status/review enums), validation visibility сужена до `domains::decisions`, string contracts из ADR-0089 не изменены. Obligation facade сохраняет public surface (`Obligation`, `NewObligation`, evidence DTOs, entity/source kinds, status/review/risk enums), beneficiary validation and risk/status contracts from ADR-0088 не изменены.

---

## Дополнение: ревью текущего Persons Trust SRP split diff

**Объём:** `backend/src/domains/persons/trust.rs`, новые `backend/src/domains/persons/trust/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0084, ADR-0086; `docs/persons/architecture.md`.

### Major: новый Persons trust split directory сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/domains/persons/trust/` вместе с tracked facade change `M backend/src/domains/persons/trust.rs`.

**Файлы:** `backend/src/domains/persons/trust.rs`, `backend/src/domains/persons/trust/*`

Facade теперь объявляет child modules через `mod ...;`. Если в commit попадет только tracked facade file без новых module files, backend не соберется. Риск особенно легко пропустить, потому что `git diff --stat` по tracked paths показывает сокращение `trust.rs`, но не показывает содержимое untracked `promises`, `risks`, `obligation_projection`, `health_projection`, `rows`, `models` и `errors`.

**Impact:** partial commit/PR ломает compile path для `PersonPromiseStore`, `PersonRiskStore`, DTO re-exports и `PersonTrustError`, а значит заденет Persona intelligence handlers и shared API store wiring.

**Fix:** перед commit/staging добавить `backend/src/domains/persons/trust/` вместе с `backend/src/domains/persons/trust.rs` и ревьюить staged diff, а не только обычный tracked `git diff`.

### Warning: live PostgreSQL ветки для promise/risk projection не исполняются без `HERMES_TEST_DATABASE_URL`

**Статус:** Открытый validation gap. `cargo test --manifest-path backend/Cargo.toml --test persons person_ -- --nocapture` прошел 14/14, но тесты `person_promise_create_materializes_user_confirmed_obligation_without_task_against_postgres` и `person_risk_report_and_resolve_materializes_health_status_cache_against_postgres` явно skipped из-за unset `HERMES_TEST_DATABASE_URL`.

**Файлы:** `backend/tests/persons.rs`, `backend/src/domains/persons/trust/*`

Split переносит SQL-heavy behavior: create/list person promises, projection into source-backed Obligations, report/resolve risks and temporary `health_status` compatibility cache. `make backend-validate` ловит compile/clippy/unit regressions, но не исполняет live SQL assertions for obligation evidence metadata, due date preservation, risk severity mapping and health cache recalculation.

**Impact:** для текущего механического split-а code-level regression не подтвержден, но будущая SQL/schema regression в этих ветках может пройти default validation без реальной БД или Testcontainers-backed coverage.

**Fix:** перед merge выполнить `persons` integration gate с реальным PostgreSQL через `HERMES_TEST_DATABASE_URL` либо перевести критичные trust projection tests на Testcontainers, чтобы они не skip-ались в обычной validation цепочке.

### Review note

Blocking split-induced regressions внутри Persons trust modules не подтверждены. Public surface сохранен через facade re-exports (`PersonPromise`, `PersonRisk`, `PersonPromiseStore`, `PersonRiskStore`, `PersonTrustError`); promise-to-obligation metadata keys, status mapping and evidence source remain unchanged; risk-to-health compatibility projection перенесен без расширения legacy `health`/`watchtower` surface. Split не добавляет Contact/CRM API, schema changes или Relationship persistence behavior сверх текущего compatibility boundary.

---

## Дополнение: ревью текущего Dev Email Sync SRP split diff

**Объём:** `backend/src/bin/hermes_email_sync_dev.rs`, новые `backend/src/bin/hermes_email_sync_dev/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0032, ADR-0041, ADR-0042, ADR-0046, ADR-0055, ADR-0076, ADR-0080.

### Major: новый Dev Email Sync split directory сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/bin/hermes_email_sync_dev/` вместе с tracked facade change `M backend/src/bin/hermes_email_sync_dev.rs`.

**Файлы:** `backend/src/bin/hermes_email_sync_dev.rs`, `backend/src/bin/hermes_email_sync_dev/*`

Explicit bin target в `backend/Cargo.toml` указывает на `src/bin/hermes_email_sync_dev.rs`, поэтому facade использует явные `#[path = "hermes_email_sync_dev/*.rs"]` module attributes. Если в commit попадет только tracked entrypoint без новых module files, binary compile снова упадет на missing modules.

**Impact:** partial commit/PR ломает `hermes-email-sync-dev`, то есть dev cache sync command для IMAP/iCloud mail ingestion.

**Fix:** перед commit/staging добавить `backend/src/bin/hermes_email_sync_dev/` вместе с `backend/src/bin/hermes_email_sync_dev.rs` и проверять staged diff.

### Warning: live provider sync не проверен текущей validation

**Статус:** Открытый validation gap. `cargo check --manifest-path backend/Cargo.toml --bin hermes-email-sync-dev` подтверждает compile path, но не подключается к IMAP/iCloud provider and does not exercise real fetch/projection against a configured mailbox.

**Файлы:** `backend/src/bin/hermes_email_sync_dev/*`, `README.md`, `backend/README.md`

Split переносит env parsing, provider-account upsert, checkpoint lookup, IMAP fetch and blob-backed projection. Compile coverage ловит module/import/type ошибки, но не подтверждает provider connectivity, credential validity, checkpoint progression или реальную запись mail blobs under `docker/data/mail`.

**Impact:** для текущего refactor-а code-level regression не подтвержден, но live dev sync behavior остается неисполненным без explicit env/provider setup.

**Fix:** перед merge выполнить dev smoke with configured non-production IMAP/iCloud account and local PostgreSQL, либо добавить fixture-backed binary smoke that exercises config/checkpoint/projection without real provider credentials.

### Review note

Blocking split-induced regressions внутри Dev Email Sync modules не подтверждены. Env variable names, iCloud/IMAP-only restriction, provider account metadata JSON, mailbox checkpoint lookup, `latest_messages` fetch mode, blob-backed pipeline projection and output report shape сохранены. Секретная граница немного усилена: env-provided mailbox password теперь хранится в `ResolvedSecret`, а JSON report по-прежнему не содержит password/token/secret fields.

---

## Дополнение: ревью текущего Mail Communication Queries / Finance Analytics SRP split diff

**Объём:** `backend/src/domains/mail/handlers/communication_queries.rs`, новые `backend/src/domains/mail/handlers/communication_queries/*` modules, `backend/src/domains/mail/handlers/finance_analytics.rs`, новые `backend/src/domains/mail/handlers/finance_analytics/*` modules, `IMPLEMENTATION_STATUS.md`.
**ADR:** ADR-0080, ADR-0085; `docs/domains/communications.md`.

### Major: новые Mail handler split directories сейчас untracked

**Статус:** Открыто до staging/commit. `git status --short` показывает `?? backend/src/domains/mail/handlers/communication_queries/` и `?? backend/src/domains/mail/handlers/finance_analytics/` вместе с tracked facade changes.

**Файлы:** `backend/src/domains/mail/handlers/communication_queries.rs`, `backend/src/domains/mail/handlers/communication_queries/*`, `backend/src/domains/mail/handlers/finance_analytics.rs`, `backend/src/domains/mail/handlers/finance_analytics/*`

Facade files теперь объявляют child modules через `mod ...;`. Если commit попадет только с tracked facade files без новых module directories, backend не соберется. Риск повышен тем, что ordinary `git diff --stat` по tracked files показывает только сокращение facades и не показывает содержимое untracked modules.

**Impact:** partial commit/PR ломает route handlers for threads, email search, personas, drafts, invoices, analytics health/senders, message explain/smart-cc and pin/important response DTOs.

**Fix:** перед commit/staging добавить оба split directories вместе с facade files и ревьюить staged diff, а не только обычный tracked `git diff`.

### Review note

Blocking split-induced regressions внутри Communication Queries / Finance Analytics modules не подтверждены. Route function names and request/response DTO names сохранены через facade re-exports; `HERMES_SEARCH_INDEX_PATH` search behavior unchanged; draft CRUD remains local store behavior; invoice DTOs moved to finance-owned invoices module; local trash/provider write semantics are not changed; no provider credential, raw MIME, attachment byte or provider mutation path is introduced.

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
