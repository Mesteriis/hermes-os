# Статус приведения к документации

Дата последнего обновления: 2026-06-13 23:27 CEST

## Выполнено

* [x] AI Persona email отображается как `name@sh-inc.ru`.
* [x] Memory Engine нормализует source-backed Persona facts.
* [x] Memory Engine собирает entity context pack с source citations.
* [x] Memory Engine выявляет required fact gaps.
* [x] Memory Engine выявляет stale-memory review candidates.
* [x] Memory Engine собирает cross-domain context pack для root entity и связанных сущностей.
* [x] Timeline Engine выполняет bounded timeline assembly и event-log projection runner.
* [x] Выполнен первичный аудит расхождений между документацией и реализацией для mail, Telegram и frontend decomposition.
* [x] `backend/src/domains/mail/handlers/mod.rs` разделен на bounded handler modules; все файлы в `backend/src/domains/mail/handlers/` меньше 700 строк.
* [x] Создан план architecture-gate работ: `docs/superpowers/plans/2026-06-13-documentation-alignment-architecture-gates.md`.
* [x] Восстановлены текущие architecture/code boundary guards и backend lint gate, выявленные pre-commit hook.
* [x] `frontend/src/lib/components/shared/AccountSetupModal.svelte` сокращен с 1219 до 97 строк и оставлен как modal shell/router.
* [x] Mail, Calendar, Telegram и WhatsApp setup flows вынесены в `frontend/src/lib/components/account-setup/`; все новые account setup компоненты меньше 500 строк.
* [x] `frontend/src/lib/pages/telegram/TelegramPage.svelte` сокращен с 842 до 476 строк; header, action rail и status messages вынесены в focused Telegram widgets.
* [x] `frontend/src/lib/pages/settings/widgets/AISettingsControlCenter.svelte` сокращен с 622 до 65 строк; AI provider, routing, prompt studio, runs, header, tabs, status и rail вынесены в focused widgets.
* [x] `frontend/src/lib/pages/communications/widgets/CommunicationsMessageDetail.svelte` сокращен с 550 до 318 строк; message body, attachments, headers, related actions и timeline вынесены в focused communication widgets.
* [x] Communications-owned CSS вынесен из `frontend/src/lib/pages/pages.css` в `frontend/src/lib/pages/communications/communications*.css`; root `pages.css` сокращен с 6852 до 5822 строк, новые communication CSS chunks меньше 700 строк.
* [x] Telegram-owned CSS вынесен из `frontend/src/lib/pages/pages.css` в `frontend/src/lib/pages/telegram/telegram*.css`, а Telegram QR setup styles — в `frontend/src/lib/components/account-setup/telegramQr.css`; root `pages.css` сокращен до 4737 строк, новые Telegram CSS chunks меньше 700 строк.
* [x] Account setup, shared setup controls, account modal и compose review CSS вынесены из `frontend/src/lib/pages/pages.css` в `frontend/src/lib/components/account-setup/accountSetup.css` и `frontend/src/lib/components/shared/*`; root `pages.css` сокращен до 4259 строк, новые CSS chunks меньше 700 строк.
* [x] Settings, appearance, integrations, AI settings и shared `HermesSelect` CSS вынесены из `frontend/src/lib/pages/pages.css` в owner-файлы `frontend/src/lib/pages/settings/*.css` и `frontend/src/lib/components/shared/hermesSelect.css`; root `pages.css` сокращен до 2540 строк, новые CSS chunks меньше 700 строк.
* [x] Agents-owned CSS вынесен из `frontend/src/lib/pages/pages.css` в `frontend/src/lib/pages/agents/agents.css`; root `pages.css` сокращен до 2324 строк, новый CSS chunk меньше 700 строк.
* [x] Calendar-owned CSS вынесен из `frontend/src/lib/pages/pages.css` в `frontend/src/lib/pages/calendar/calendar.css`; root `pages.css` сокращен до 2042 строк, новый CSS chunk меньше 700 строк.
* [x] Documents/Notes-owned CSS вынесен из `frontend/src/lib/pages/pages.css` и `frontend/src/lib/styles/app.css` в owner-файлы `frontend/src/lib/pages/documents/documents.css` и `frontend/src/lib/pages/notes/notes.css`; root `pages.css` сокращен до 1912 строк, `app.css` сокращен до 1008 строк, новые CSS chunks меньше 700 строк.
* [x] Projects-owned CSS вынесен из `frontend/src/lib/pages/pages.css` и `frontend/src/lib/styles/app.css` в `frontend/src/lib/pages/projects/projects.css`; root `pages.css` сокращен до 1778 строк, `app.css` сокращен до 998 строк, новый CSS chunk меньше 700 строк.
* [x] Tasks-owned CSS вынесен из `frontend/src/lib/pages/pages.css` и `frontend/src/lib/styles/app.css` в `frontend/src/lib/pages/tasks/tasks.css`; root `pages.css` сокращен до 1628 строк, `app.css` сокращен до 995 строк, новый CSS chunk меньше 700 строк.
* [x] Persons-owned CSS вынесен из `frontend/src/lib/pages/pages.css` и `frontend/src/lib/styles/app.css` в `frontend/src/lib/pages/persons/persons.css`; root `pages.css` сокращен до 1416 строк, `app.css` сокращен до 990 строк, новый CSS chunk меньше 700 строк.
* [x] Timeline-owned CSS вынесен из `frontend/src/lib/pages/pages.css` и `frontend/src/lib/styles/app.css` в `frontend/src/lib/pages/timeline/timeline.css`; неиспользуемый `timeline-slider` CSS удален; root `pages.css` сокращен до 1349 строк, `app.css` сокращен до 987 строк, новый CSS chunk меньше 700 строк.
* [x] Organizations-owned CSS вынесен из `frontend/src/lib/pages/pages.css` и `frontend/src/lib/styles/app.css` в `frontend/src/lib/pages/organizations/organizations.css`; root `pages.css` сокращен до 1178 строк, `app.css` сокращен до 985 строк, новый CSS chunk меньше 700 строк.
* [x] Knowledge/Review-owned CSS вынесен из `frontend/src/lib/pages/pages.css` и `frontend/src/lib/styles/app.css` в `frontend/src/lib/pages/knowledge/knowledge.css` и `frontend/src/lib/pages/review/review.css`; root `pages.css` сокращен до 457 строк и больше не является God File, `app.css` сокращен до 973 строк, новые CSS chunks меньше 700 строк.
* [x] Sidebar Settings-owned CSS вынесен из `frontend/src/lib/components/shell/sidebar.css` в `frontend/src/lib/pages/settings/widgets/sidebarSettings.css`; `sidebar.css` сокращен до 590 строк и больше не является God File, новый CSS chunk содержит 270 строк.
* [x] Shared panel/editor/strip CSS и page-owned panel selectors вынесены из `frontend/src/lib/components/shared/panels.css` в owner-файлы компонентов и страниц; `panels.css` сокращен с 1780 до 697 строк и больше не является God File, новые CSS chunks меньше 700 строк.
* [x] Shell layout и shell theme CSS вынесены из `frontend/src/lib/styles/app.css` в `frontend/src/lib/styles/shell.css` и `frontend/src/lib/styles/shellTheme.css`; `app.css` сокращен с 973 до 640 строк и больше не является God File.
* [x] Communications store разделен на bounded modules `state`, `loaders`, `compose`, `actions`, `selectors` и `formatters`; `frontend/src/lib/stores/communications.ts` сокращен с 899 до 5 строк и оставлен как public facade, все store modules меньше 700 строк.
* [x] Accounts service разделен на bounded modules `calendar`, `drawer`, `labels`, `mailImport`, `mailSetup`, `mailWizard`, `shared`, `telegram` и `types`; `frontend/src/lib/services/accounts.ts` сокращен с 1011 до 9 строк и оставлен как public facade, все account service modules меньше 700 строк.
* [x] Communications service разделен на bounded modules `actions`, `compose`, `constants`, `formatters`, `loaders`, `related`, `rendering`, `resources`, `types`, `workbench` и `workflow`; `frontend/src/lib/services/communications.ts` сокращен с 1437 до 11 строк и оставлен как public facade, все communication service modules меньше 700 строк.
* [x] Telegram service разделен на bounded modules `automation`, `calls`, `constants`, `fixtures`, `lifecycle`, `messages`, `parsing`, `runtime`, `selection`, `types`, `wizard` и `workspace`; `frontend/src/lib/services/telegram.ts` сокращен с 1584 до 11 строк и оставлен как public facade, все Telegram service modules меньше 700 строк.
* [x] API contract types разделены на bounded modules `accounts`, `ai`, `calendar`, `communication`, `contradictions`, `decisions`, `documents`, `graph`, `mail`, `obligations`, `organizations`, `persons`, `projects`, `relationships`, `settings`, `tasks`, `telegram`, `vault` и `whatsapp`; `frontend/src/lib/api/types.ts` сокращен с 2615 до 19 строк и оставлен как public facade, все API type modules меньше 700 строк.
* [x] Telegram TDLib JSON boundary разделен на bounded modules `client`, `identifiers`, `library_paths`, `parsing`, `qr_login`, `qr_login_support`, `requests` и `snapshots`; `backend/src/integrations/telegram/tdjson.rs` сокращен с 2361 до 603 строк и оставлен как public facade, все TDLib modules меньше 700 строк.
* [x] Telegram client/store boundary разделен на bounded modules `accounts`, `chats`, `errors`, `identifiers`, `messages`, `models`, `projection`, `rows`, `store`, `validation` и `vault`; `backend/src/integrations/telegram/client.rs` сокращен с 1793 до 36 строк и оставлен как public facade, все новые client modules меньше 700 строк.
* [x] Telegram runtime boundary разделен на bounded modules `actor`, `commands`, `manager`, `media`, `models`, `state`, `status` и `validation`; `backend/src/integrations/telegram/runtime.rs` сокращен с 1538 до 22 строк и оставлен как public facade, все новые runtime modules меньше 700 строк.
* [x] Mail background sync boundary разделен на bounded modules `service`, `provider`, `store`, `models`, `errors`, `rows` и `validation`; `backend/src/domains/mail/background_sync.rs` сокращен с 1684 до 24 строк и оставлен как public facade, все новые background sync modules меньше 700 строк.
* [x] Calendar handlers boundary разделен на bounded modules `accounts`, `events`, `intelligence`, `meetings`, `scheduling`, `health`, `brain`, `search`, `rules`, `sync`, `reminders` и `analytics`; `backend/src/domains/calendar/handlers/mod.rs` сокращен с 1674 до 171 строки и оставлен как crate-local facade, все calendar handler modules меньше 700 строк.
* [x] Shared API support boundary разделен на bounded modules `stores`, `platform_dtos`, `communications`, `review_lists`, `messaging_integrations`, `automation_calls`, `review_commands`, `query_parsing` и `formatting`; `backend/src/domains/api_support.rs` сокращен с 1762 до 171 строки и оставлен как crate-local facade, все новые API support modules меньше 700 строк.
* [x] App error boundary разделен на bounded modules `types`, `response`, `conversions` и focused response mappings; `backend/src/app/error.rs` сокращен с 1720 до 5 строк и оставлен как crate-local facade, все новые app error modules меньше 700 строк.
* [x] AI Control Center boundary разделен на bounded modules `store`, `providers`, `catalog`, `routes`, `prompts`, `vault`, `models`, `errors`, `rows`, `presets` и `validation`; `backend/src/ai/control_center.rs` сокращен с 1908 до 25 строк и оставлен как public facade, все новые AI Control Center modules меньше 700 строк.
* [x] AI Core boundary разделен на bounded modules `semantic`, `runs`, `service`, `types`, `agents`, `prompts`, `helpers`, `errors` и `constants`; `backend/src/ai/core.rs` сокращен с 1889 до 24 строк и оставлен как public facade, все новые AI Core modules меньше 700 строк.

## В работе

* [ ] Приведение реализации к документации без новых compatibility layers.
* [ ] Аудит существующих compatibility-модулей, legacy API и устаревших полей.
* [ ] Перевод интеграционных тестов на изолированный Testcontainers-подход.
* [ ] Устранение God Files и God Components перед добавлением новых mail/Telegram возможностей.

## Осталось реализовать

* [ ] Устранить оставшиеся backend source files больше 700 строк без добавления новых God Files.
* [ ] Удалить compatibility storage вокруг `persons` и перейти на Persona-native schema.
* [ ] Удалить legacy Contact/Person CRM API и заменить их Persona-domain контрактами.
* [ ] Подключить Memory Engine к доменным источникам без compatibility cache.
* [ ] Реализовать durable read-model storage для Timeline Engine projections.
* [ ] Реализовать Enrichment Engine approved-source policy, conflict routing и cross-domain candidates.
* [ ] Реализовать Trust Engine contradiction inputs, review recommendations и cross-domain reconciliation.
* [ ] Реализовать Risk Engine cross-domain observations и review routing без терминов `health`/`watchtower`.
* [ ] Реализовать live-provider ingestion для Decisions, Obligations и Polygraph.
* [ ] Удалить или заменить функционал, который отсутствует в актуальной документации.
* [ ] Актуализировать все тесты под документацию как источник истины.
* [ ] Прогнать полный форматтер, линтеры, статический анализ и все тесты.

## Архитектурные проблемы

* В коде ещё есть compatibility layers вокруг `persons`, `health`, `watchtower`, legacy Person/Contact терминологии и старых API.
* В backend остаются source files больше 700 строк за пределами уже разделенных `ai/control_center`, `ai/core`, `app/error`, `domains/api_support`, `mail/handlers`, `mail/background_sync`, `calendar/handlers`, `integrations/telegram/tdjson`, `integrations/telegram/client` и `integrations/telegram/runtime`.
* Во frontend больше не осталось Svelte-компонентов больше 500 строк по текущему scan.
* Во frontend больше не осталось source/service/store files больше 700 строк по текущему scan.
* Во frontend больше не осталось CSS files больше 700 строк по текущему scan; `app.css` сокращен до 640 строк, `pages.css` — до 457 строк, `sidebar.css` — до 590 строк, `panels.css` — до 697 строк.
* Часть интеграционных тестов зависит от общего dev-контейнера, а не от полного цикла Container → Migration → Fixture → Run → Destroy.
* Некоторые реализованные engine baseline ещё не подключены как полноценные доменные процессы.

## Следующий шаг

Продолжить устранение God Files: следующим срезом выбрать backend source file больше 700 строк по документационной важности и blast radius; не добавлять новую функциональность в такие файлы до декомпозиции.
