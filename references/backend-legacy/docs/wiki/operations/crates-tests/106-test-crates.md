---
chunk_id: 106-test-crates
batch_id: batch-20260628T214902
group: crates
role: test
source_status: pending
source_count: 1
generated_by: code-wiki-ru
---

# 106-test-crates — crates/test

- Target index: [[operations/crates-tests]]
- Batch: `batch-20260628T214902`
- Source files: `1`

## Резюме

Страница `operations/crates-tests.md` должна быть создана в русской Obsidian‑wiki. Она документирует smoke‑тесты инфраструктуры из `crates/testkit/tests/smoke_test.rs`: характеризационные тесты, проверяющие запуск контейнера PostgreSQL, применение миграций, изоляцию тестовых баз данных и работу фабрик сущностей. Страница описывает назначение, перечень сценариев и ключевые утверждения, полностью опираясь на предоставленный исходный файл.

## Предложенные страницы

#### `operations/crates-tests.md`

```markdown
# Тесты инфраструктуры тестового окружения

## Назначение

Smoke-тесты в `crates/testkit/tests/smoke_test.rs` — это характеризационные тесты (characterization tests). Они **не изменяют** поведение, а фиксируют текущее состояние тестовой инфраструктуры. Любое изменение, способное нарушить работу тестового окружения, приведёт к падению этих тестов, заставляя разработчика проверить совместимость.

Тесты используют контейнер PostgreSQL с расширением `pgvector`, поднимаемый через библиотеку Testcontainers.

## Тестовые сценарии

### 1. Создание изолированной базы данных – `test_context_creates_isolated_database` (AC1)
- Проверяет, что контейнер PostgreSQL стартует и проходит health‑check.
- Выполняет запрос `SELECT 1` и проверяет результат `== 1`.
- Подтверждает, что база данных способна отвечать на запросы.

### 2. Применение миграций – `test_context_runs_migrations`
- Проверяет, что миграции были применены к тестовой базе.
- Запрашивает количество успешных миграций из таблицы `_sqlx_migrations` (условие `success = true`).
- Ожидает, что количество > 0.

### 3. Изоляция баз данных – `test_context_databases_are_isolated`
- Создаёт два экземпляра `TestContext`.
- Сравнивает их строки подключения, полученные через `connection_string()`.
- Утверждает, что строки различны – каждый контекст получает уникальную базу данных.

### 4. Фабрика контактов – `testkit_contact_factory_creates_person`
- Проверяет `ContactFactory` (`testkit::factories::contact::ContactFactory`) на реальном контейнере.
- Создаёт персону с именем `"Characterization Test Person"` и email `"char-test@example.com"`.
- Утверждает, что идентификатор персоны непустой.
- Дополнительно проверяет существование записи в таблице `persons` с этим идентификатором через `SELECT EXISTS(...)`.

### 5. Фабрика писем – `testkit_email_factory_creates_email`
- Проверяет `EmailFactory` (`testkit::factories::email::EmailFactory`) на реальном контейнере.
- Создаёт письмо с темой, отправителем и телом.
- Утверждает, что `raw_record_id` непустой.

## Технические детали
- Все тесты асинхронные (`#[tokio::test]`).
- Перед каждым тестом создаётся новый `TestContext`, автоматически поднимающий контейнер, запускающий миграции и предоставляющий пул соединений (`pool()`).
- Строки подключения проверяются на уникальность, чтобы гарантировать изоляцию тестов.
```

## Покрытие источников

- **`crates/testkit/tests/smoke_test.rs`**
  - Общее назначение smoke‑тестов: характеризационные тесты, проверяющие Testcontainers‑инфраструктуру, миграции и фабрики.
  - `test_context_creates_isolated_database`: AC1 — контейнер проходит health‑check, запрос `SELECT 1` возвращает `1`.
  - `test_context_runs_migrations`: проверка таблицы `_sqlx_migrations`, количество успешных миграций > 0.
  - `test_context_databases_are_isolated`: строки подключения `connection_string()` двух `TestContext` различны.
  - `testkit_contact_factory_creates_person`: создание персоны через `ContactFactory`, проверка непустого ID и существования в `persons`.
  - `testkit_email_factory_creates_email`: создание письма через `EmailFactory`, проверка непустого `raw_record_id`.

## Исходные файлы

- [`crates/testkit/tests/smoke_test.rs`](../../../../crates/testkit/tests/smoke_test.rs)

## Кандидаты на drift

На основе предоставленного контекста видимых расхождений между кодом и документацией не обнаружено.
