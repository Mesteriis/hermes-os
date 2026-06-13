# Статус приведения к документации

Дата последнего обновления: 2026-06-13 16:30

## Выполнено

* [x] AI Persona email отображается как `name@sh-inc.ru`.
* [x] Memory Engine нормализует source-backed Persona facts.
* [x] Memory Engine собирает entity context pack с source citations.
* [x] Memory Engine выявляет required fact gaps.
* [x] Memory Engine выявляет stale-memory review candidates.
* [x] Memory Engine собирает cross-domain context pack для root entity и связанных сущностей.
* [x] Timeline Engine выполняет bounded timeline assembly и event-log projection runner.

## В работе

* [ ] Приведение реализации к документации без новых compatibility layers.
* [ ] Аудит существующих compatibility-модулей, legacy API и устаревших полей.
* [ ] Перевод интеграционных тестов на изолированный Testcontainers-подход.

## Осталось реализовать

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
* Часть интеграционных тестов зависит от общего dev-контейнера, а не от полного цикла Container → Migration → Fixture → Run → Destroy.
* Некоторые реализованные engine baseline ещё не подключены как полноценные доменные процессы.
* Репозиторий содержит много параллельных незакоммиченных изменений; их нужно разнести перед финальной проверкой.

## Следующий шаг

Удалить compatibility-мышление из Persona/Person API: выбрать первый legacy surface, заменить его Persona-native контрактом и обновить тесты через Testcontainers.
