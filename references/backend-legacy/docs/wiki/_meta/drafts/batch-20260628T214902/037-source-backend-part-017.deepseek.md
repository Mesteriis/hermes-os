### Summary / Резюме

Обновление страницы `components/backend.md` — добавляется детальное описание домена календаря (`domains/calendar`) на русском языке. Основано исключительно на встроенных исходных файлах и охватывает структуру модулей, ключевые сущности, хранилища, сервисы (интеллект, здоровье, планирование, встречи, напоминания, правила) и общие архитектурные решения (интеграция с наблюдениями, транзакционность, идентификаторы).

### Proposed pages / Предлагаемые страницы

**`components/backend.md`**

```markdown
# Компоненты бэкенда

> **Примечание:** в этом документе описана только часть компонента **Calendar**. Сведения об остальных доменах бэкенда недоступны в данном фрагменте исходного кода.

## Домен «Календарь» (`domains/calendar`)

Домен календаря организован в виде набора модулей, которые покрывают работу с событиями, синхронизацию, аналитику, планирование, заметки по встречам и другие аспекты.

### Структура модуля

Согласно `mod.rs`, в состав домена входят следующие публичные модули:

- `brain` – интеграция с «мозгом» (AI/автоматизация)
- `command_service` – командная служба
- `core` – ядро
- `events` – события календаря и источники
- `evidence` – связывание сущностей с наблюдениями (`observations`)
- `health` – мониторинг состояния (Watchtower)
- `intelligence` – интеллектуальная классификация и анализ событий
- `meetings` – заметки, исходы, записи и транскрипты встреч
- `reminders` – напоминания о событиях
- `rules` – календарные правила
- `scheduling` – дедлайны, Focus-блоки и поиск свободных слотов
- `service` – основной сервис
- `sync` – синхронизация

### Ключевые сущности и хранилища

#### `events`

**`CalendarEvent`** – модель события календаря. Поля включают:
`event_id`, `observation_id`, `source_event_id`, `account_id`, `source_id`, `title`, `description`, `location`, `start_at`, `end_at`, `timezone`, `all_day`, `recurrence_rule`, `status`, `visibility`, `event_type`, `importance_score`, `readiness_score`, `sync_status`, `conference_url`, `conference_provider`, `preparation_reminder_minutes`, `travel_buffer_minutes`, `created_at`, `updated_at`.

Маппинг строки PostgreSQL в модель происходит в `row_to_event` (`events/rows.rs`).

**`CalendarSourceStore`** – хранилище источников календаря. Методы:
- `create` – создание источника
- `create_with_observation` – создание источника с одновременной привязкой к `observation`
- `list_by_account` – список источников аккаунта
- `get` – получение по `source_id`

Модель `CalendarSource` содержит поля:
`source_id`, `account_id`, `provider_calendar_id`, `name`, `color`, `timezone`, `visibility`, `read_only`, `sync_enabled`, `capabilities`, `created_at`, `updated_at`.

Все операции с источниками могут интегрироваться с платформенным слоем наблюдений через связывание сущностей в рамках транзакции (`link_vault_owned_entity_in_transaction`). Генерация идентификаторов использует формат `<prefix>:v1:<timestamp_hex>`.

#### `evidence`

Модуль содержит вспомогательные функции для связывания календарных сущностей (в т.ч. внутри транзакций) с наблюдениями:

- `link_calendar_entity` – вне транзакции
- `link_calendar_entity_in_transaction` – внутри транзакции

Обе функции объединяют `base_metadata` и `extra_metadata` (с приоритетом дополнительного) и вызывают методы уровня платформы (`crate::platform::observations::link_domain_entity`).

#### `health`

Сервис **`CalendarWatchtowerService`** предоставляет мониторинговые методы:

- `events_needing_preparation` – события в ближайшие 24 часа со статусом `scheduled` и низким показателем готовности (`readiness_score < 0.5`)
- `events_without_outcomes` – завершённые события без связанных заметок (`meeting_notes`)
- `weekly_brief` – статистика на предстоящую неделю: количество предстоящих событий, просроченных дедлайнов, событий без заметок
- `meeting_load_analysis` – дневная загрузка за последние 7 дней (количество событий и часы)
- `time_distribution` – распределение времени по категориям (использует `categorize_time` из `CalendarIntelligenceService`)
- `focus_balance` – баланс между фокус-блоками и встречами
- `back_to_back_meetings` – обнаружение групп встреч без перерыва (back-to-back) через `detect_back_to_back`

#### `intelligence`

Сервис **`CalendarIntelligenceService`** реализует эвристический анализ событий. Подмодули:

- **`classification`** – классификация события на основе заголовка (`classify_event`) по ключевым словам (meeting, deadline, focus, travel, tax, legal, finance, birthday, reminder и др.), а также по количеству участников и продолжительности.
- **`fingerprint`** – эвристический «отпечаток» события (`heuristic_fingerprint`), который вычисляет тип, важность, язык и намёк на повторяемость.
- **`scoring`** – оценка важности (`calculate_importance`) и готовности (`calculate_readiness`), а также выявление рисков (`detect_risks`) — отсутствие повестки, документов, участников и т.д.
- **`analytics`** – категоризация времени (`categorize_time`) и обнаружение цепочек встреч без перерыва (`detect_back_to_back`).
- **`conference`** – определение провайдера конференции (`detect_conference_provider`) по URL (поддерживаются Google Meet, Zoom, Microsoft Teams, Jitsi, WebEx) и извлечение URL (`extract_conference_url`).
- **`location`** – разбор строки местоположения (`parse_location`), определение онлайн/офлайн, выделение названия (Office, Home, Cafe, Airport, Hotel) и задание буфера на дорогу (15 минут для офлайн).
- **`models`** – модели `EventAnalysis`, `EventFingerprint`, `LocationInfo`, `BackToBackGroup`.

#### `meetings`

Поддомен для работы с заметками, исходами, записями и транскриптами встреч.

**Модели**:
- `MeetingNote` (заметка): `id`, `event_id`, `content`, `format`, `source`, `linked_note_id`, `created_at`, `updated_at`
- `MeetingOutcome` (исход/результат): `id`, `event_id`, `outcome_type`, `title`, `description`, `owner_person_id`, `due_date`, `source`, `confidence`, `linked_entity_id`, `created_at`, `updated_at`
- `EventRecording` (запись): `id`, `event_id`, `file_path`, `source`, `duration_seconds`, `transcript_id`, `processing_status`, `created_at`, `updated_at`
- `EventTranscript` (транскрипт): `id`, `event_id`, `text`, `language`, `summary`, `model`, `created_at`

**Хранилища**:
- `MeetingNoteStore`: `list`, `create`, `create_with_observation` (связывание с observation)
- `MeetingOutcomeStore`: `list`, `add`, `add_with_observation`, `set_linked_entity_id_in_transaction`, `follow_up_status`
- `EventRecordingStore`: `list`, `add`, `add_with_observation`, `find_by_file_path`, `attach_transcript`
- `EventTranscriptStore`: `get`, `add_with_observation`

Все хранилища поддерживают связывание с наблюдениями через функции из `evidence` (либо вне транзакции, либо внутри).

#### `reminders`

Модель **`CalendarReminder`** содержит:
`id`, `event_id`, `reminder_type`, `minutes_before`, `condition_json`, `message`, `source`, `is_active`, `last_triggered_at`, `created_at`, `updated_at`.

Хранилище **`CalendarReminderStore`** позволяет:
- `list` – список напоминаний для события
- `create` / `create_with_source` / `create_with_observation` – создание с возможной привязкой к наблюдению
- `set_active` / `set_active_with_source` / `set_active_with_observation` – переключение активности

#### `rules`

**`CalendarRule`** – правило календаря с полями:
`rule_id`, `name`, `natural_language_description`, `compiled_dsl` (DSL в виде JSON), `enabled`, `approval_mode` (по умолчанию `suggest_only`), `last_run_at`, `created_at`, `updated_at`.

Хранилище **`CalendarRuleStore`**:
- `list` – все правила
- `create` / `create_with_observation` – создание правила, `rule_id` генерируется в формате `rule:v1:<timestamp_hex>`
- `update` / `update_with_observation` – частичное обновление через `RuleUpdate`
- `delete` / `delete_with_observation` – удаление

Все мутационные операции обёрнуты в транзакции и поддерживают связывание с наблюдениями.

#### `scheduling`

**`DeadlineEvent`** – событие-дедлайн с полями:
`id`, `source_entity_type`, `source_entity_id`, `title`, `due_at`, `severity` (по умолчанию `medium`), `status`, `linked_calendar_event_id`, `created_at`, `updated_at`.

**`DeadlineStore`** реализует `list` (с фильтром по статусу или без) и `create` / `create_with_observation`.

**`FocusBlock`** – блок сфокусированной работы:
`id`, `title`, `start_at`, `end_at`, `purpose`, `linked_project_id`, `protection_level` (по умолчанию `medium`), `status`, `created_at`, `updated_at`.

**`FocusBlockStore`** реализует `list` (с опциональным временным диапазоном) и `create` / `create_with_observation`.

**`SmartSchedulingService`** – эвристический поиск свободных слотов (`find_slots`) на основе существующих событий. Возвращает список `Slot` с полями `start`, `end`, `duration_minutes`.

### Общие архитектурные решения

- **Интеграция с наблюдениями (observations)**. Многие хранилища поддерживают операции с явной привязкой к `observation_id` через методы `*_with_observation`. Связывание происходит либо вне транзакции, либо внутри неё через модуль `evidence` и платформенный слой `crate::platform::observations`.
- **Транзакционность**. Критические операции (создание, обновление, удаление, изменение связей) выполняются в транзакциях PostgreSQL.
- **Идентификаторы**. Используются префиксные ID с временной меткой вида `<prefix>:v1:<timestamp_hex>` (префиксы: `src` для источников, `rule` для правил и др.).
- **Обработка ошибок**. Используется `thiserror` для доменных ошибок (например, `CalendarError`, `MeetingsError`, `CalendarRuleError`, `SchedulingError`, `CalendarHealthError`), многие из которых прозрачно оборачивают `sqlx::Error`.
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `mod.rs` | Публичные модули домена: brain, command_service, core, events, evidence, health, intelligence, meetings, reminders, rules, scheduling, service, sync. |
| `events/rows.rs` | Модель `CalendarEvent`, все поля, функция `row_to_event`. |
| `events/source_store.rs` | `CalendarSourceStore` и его методы, модель `CalendarSource`, структура `VaultOwnedEntityLink`, функция связывания в транзакции, генератор `next_id`. |
| `evidence.rs` | Функции `link_calendar_entity` и `link_calendar_entity_in_transaction`, слияние метаданных, вызов `link_domain_entity`. |
| `health.rs` | `CalendarWatchtowerService` и все его методы, SQL-запросы к `calendar_events`, `meeting_notes`, `deadline_events`, типы возвращаемых данных. |
| `intelligence.rs` | Объявление `CalendarIntelligenceService`, реэкспорт ошибок и моделей. |
| `intelligence/analytics.rs` | Метод `categorize_time`, разбор по event_type и title; метод `detect_back_to_back`, группировка встреч с промежутком ≤5 минут. |
| `intelligence/classification.rs` | Метод `classify_event`, набор ключевых слов, эвристика по участникам и длительности. |
| `intelligence/conference.rs` | Метод `detect_conference_provider`, поддерживаемые провайдеры; метод `extract_conference_url`. |
| `intelligence/errors.rs` | Вариант ошибки `CalendarIntelligenceError::AnalysisFailed`. |
| `intelligence/fingerprint.rs` | Метод `heuristic_fingerprint`, вычисление типа, важности, языка, намёка на повторяемость. |
| `intelligence/location.rs` | Метод `parse_location`, определение онлайн/офлайн, парсинг названия, буфер 15 минут. |
| `intelligence/models.rs` | Модели `EventAnalysis`, `EventFingerprint`, `LocationInfo`, `BackToBackGroup`. |
| `intelligence/scoring.rs` | Методы `calculate_importance`, `calculate_readiness`, `detect_risks`, формулы оценок. |
| `meetings.rs` | Публичные реэкспорты: ошибки, модели, хранилища заметок, исходов, записей, транскриптов. |
| `meetings/errors.rs` | Варианты `MeetingsError::Sqlx`, `Observation`, `NotFound`. |
| `meetings/models.rs` | Модели `MeetingNote`, `MeetingOutcome`, `EventRecording`, `EventTranscript` и их поля. |
| `meetings/notes.rs` | `MeetingNoteStore` и методы `list`, `create`, `create_with_observation`. |
| `meetings/outcomes.rs` | `MeetingOutcomeStore` и методы `list`, `add`, `add_with_observation`, `set_linked_entity_id_in_transaction`, `follow_up_status`. |
| `meetings/recordings.rs` | `EventRecordingStore` и методы `list`, `add`, `add_with_observation`, `find_by_file_path`, `attach_transcript`, `attach_transcript_in_transaction`. |
| `meetings/rows.rs` | Константы колонок для meeting_notes, meeting_outcomes, event_recordings, event_transcripts; строковые мапперы. |
| `meetings/transcripts.rs` | `EventTranscriptStore` и методы `get`, `add_with_observation`. |
| `reminders.rs` | Модель `CalendarReminder`, хранилище `CalendarReminderStore` и методы `list`, `create`, `set_active` с вариациями. |
| `rules.rs` | Модель `CalendarRule`, `RuleUpdate`, хранилище `CalendarRuleStore` и методы `list`, `create`, `update`, `delete` с вариациями. |
| `scheduling.rs` | Модели `DeadlineEvent`, `FocusBlock`, `Slot`; хранилища `DeadlineStore`, `FocusBlockStore`; сервис `SmartSchedulingService` с `find_slots`. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом и документацией не видно. Сравнение с существующими wiki‑страницами или ADR невозможно, так как они не были включены в исходные файлы данного чанка.
