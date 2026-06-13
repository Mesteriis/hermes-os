# Статус приведения к документации

Дата последнего обновления: 2026-06-13 19:44 CEST

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

## В работе

* [ ] Приведение реализации к документации без новых compatibility layers.
* [ ] Аудит существующих compatibility-модулей, legacy API и устаревших полей.
* [ ] Перевод интеграционных тестов на изолированный Testcontainers-подход.
* [ ] Устранение God Files и God Components перед добавлением новых mail/Telegram возможностей.

## Осталось реализовать

* [ ] Продолжить разнос крупных frontend CSS файлов по владельцам компонентов и страниц: `pages.css`, `panels.css`, `app.css`, `sidebar.css`.
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
* В backend остаются source files больше 700 строк за пределами уже разделенного `mail/handlers`.
* Во frontend больше не осталось Svelte-компонентов больше 500 строк по текущему scan.
* Во frontend остаются крупные service/source files больше 700 строк, включая `frontend/src/lib/services/accounts.ts`; их нельзя расширять новыми возможностями без предварительной декомпозиции.
* Во frontend остаются крупные shared CSS files (`pages.css` — 1178 строк, `panels.css` — 1780 строк, `app.css` — 985 строк, `sidebar.css` — 841 строк), которые блокируют ownership-based компонентную декомпозицию.
* Часть интеграционных тестов зависит от общего dev-контейнера, а не от полного цикла Container → Migration → Fixture → Run → Destroy.
* Некоторые реализованные engine baseline ещё не подключены как полноценные доменные процессы.

## Следующий шаг

Продолжить декомпозицию `frontend/src/lib/pages/pages.css` по оставшимся page owners, затем разнести `panels.css`, `app.css` и `sidebar.css` по владельцам компонентов.
