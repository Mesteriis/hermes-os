# Статус приведения к документации

Дата последнего обновления: 2026-06-13 17:04 CEST

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

## В работе

* [ ] Приведение реализации к документации без новых compatibility layers.
* [ ] Аудит существующих compatibility-модулей, legacy API и устаревших полей.
* [ ] Перевод интеграционных тестов на изолированный Testcontainers-подход.
* [ ] Устранение God Files и God Components перед добавлением новых mail/Telegram возможностей.

## Осталось реализовать

* [ ] Декомпозировать `frontend/src/lib/components/shared/AccountSetupModal.svelte` на доменные wizard-компоненты и оставить modal shell тонким.
* [ ] Декомпозировать `frontend/src/lib/pages/telegram/TelegramPage.svelte` перед расширением Telegram parity.
* [ ] Декомпозировать `frontend/src/lib/pages/settings/widgets/AISettingsControlCenter.svelte`.
* [ ] Декомпозировать `frontend/src/lib/pages/communications/widgets/CommunicationsMessageDetail.svelte`.
* [ ] Разнести крупные frontend CSS файлы по владельцам компонентов и страниц: `pages.css`, `panels.css`, `app.css`, `sidebar.css`.
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
* Во frontend остаются компоненты больше 500 строк: `AccountSetupModal.svelte`, `TelegramPage.svelte`, `AISettingsControlCenter.svelte`, `CommunicationsMessageDetail.svelte`.
* Во frontend остаются крупные shared CSS files, которые блокируют ownership-based компонентную декомпозицию.
* Часть интеграционных тестов зависит от общего dev-контейнера, а не от полного цикла Container → Migration → Fixture → Run → Destroy.
* Некоторые реализованные engine baseline ещё не подключены как полноценные доменные процессы.

## Следующий шаг

Декомпозировать `frontend/src/lib/components/shared/AccountSetupModal.svelte`: вынести mail, calendar, Telegram и WhatsApp setup flows в отдельные компоненты и покрыть чистые account setup helpers frontend unit-тестами.
