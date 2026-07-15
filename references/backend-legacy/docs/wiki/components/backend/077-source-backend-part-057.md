---
chunk_id: 077-source-backend-part-057
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 6
generated_by: code-wiki-ru
---

# 077-source-backend-part-057 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `6`

## Резюме

В этом чанке добавляются в русскую Obsidian wiki описания нескольких рабочих процессов (workflows) backend, исходный код которых предоставлен: `telegram_media_storage`, `workflow_action_person_projection`, `yandex_telemost_calendar_matching`, `zoom_calendar_matching`, `zoom_participant_identity`, `zoom_signal_detection`. Страница `components/backend.md` пополняется секциями с ключевыми типами, функциями, константами и поведением каждого процесса, извлечёнными строго из вложенных исходников.

## Предложенные страницы

## components/backend.md

```markdown
# Компоненты Backend

На этой странице описаны рабочие процессы (workflows) модуля `backend/src/workflows/`. Каждый процесс отвечает за обработку событий, сохранение данных или взаимодействие с другими доменами платформы.

---

## Telegram Media Storage

**Файл:** `backend/src/workflows/telegram_media_storage.rs`

### Типы данных

- **`TelegramMediaDownloadData`**
  Поля: `account_id`, `provider_chat_id`, `provider_message_id`, `tdlib_file_id`, `provider_attachment_id`, `filename`, `content_type`.
  Методы (логика fallback):
  - `provider_attachment_id()` – если переданный id пуст, возвращается `"tdlib-file:{tdlib_file_id}"`.
  - `content_type()` – если не задан, возвращается `"application/octet-stream"`.
  - `filename()` – возвращает `None`, если переданное имя пустое.

- **`TelegramDownloadedFileData`**
  Поля: `file_id`, `size_bytes`, `expected_size_bytes`, `local_path`, `is_downloading_active`, `is_downloading_completed`, `downloaded_size_bytes`.

- **`TelegramAttachmentAnchor`**
  Поля: `message_id`, `raw_record_id`.

- **`TelegramMediaDownloadProjection`**
  Проекция после сохранения медиа: `account_id`, `provider_chat_id`, `provider_message_id`, `runtime_kind`, `status`, `tdlib_file_id`, `local_path`, `size_bytes`, `expected_size_bytes`, `downloaded_size_bytes`, `is_downloading_active`, `is_downloading_completed`, `attachment_id`, `blob_id`, `scan_status`.

- **`TelegramProviderMediaCommand`**
  Поля: `command_id`, `account_id`, `command_kind`, `provider_chat_id`, `payload` (JSON).

- **`TelegramPreparedMediaSendRequest`**
  Поля: `command_id`, `provider_chat_id`, `media_type`, `local_path`, `caption`, `filename`.

### Ошибки

- **`TelegramMediaStorageError`**
  Варианты: `InvalidRequest(String)`, `Runtime(String)`, `Storage(String)`.

### Функции

#### `persist_downloaded_media`
Сигнатура:
```rust
pub(crate) async fn persist_downloaded_media(
    pool: PgPool,
    request: &TelegramMediaDownloadData,
    file: &TelegramDownloadedFileData,
    anchor: Option<TelegramAttachmentAnchor>,
    blob_root: &Path,
) -> Result<TelegramMediaDownloadProjection, TelegramMediaStorageError>
```
Поведение:
- Формирует начальную проекцию со статусом `"downloaded"`, `"downloading"` или `"remote"` в зависимости от флагов `file`.
- Если загрузка не завершена (`is_downloading_completed == false`), немедленно возвращает проекцию без сохранения блоба.
- При завершённой загрузке:
  - Читает файл по локальному пути (`tokio::fs::read`).
  - Сохраняет блоб через `LocalCommunicationBlobPort::put_blob`.
  - Записывает метаданные блоба через `CommunicationBlobMetadataPort::upsert_blob` с `content_type` из `request`.
  - Выполняет проверку безопасности через `NoopAttachmentSafetyScanner` (заглушка, реально не сканирует).
  - Требует непустой `anchor` (иначе ошибка `InvalidRequest`).
  - Создаёт прикрепление `NewCommunicationAttachment` с disposition `Attachment` и сохраняет через `CommunicationBlobMetadataPort::upsert_attachment`.
  - Дополняет проекцию полями `attachment_id`, `blob_id`, `scan_status`.

#### `media_send_request` (видимая часть, файл обрезан)
Сигнатура:
```rust
pub(crate) async fn media_send_request(
    pool: &PgPool,
    command: &TelegramProviderMediaCommand,
) -> Result<TelegramPreparedMediaSendRequest, TelegramMediaStorageError>
```
Поведение (на основе имеющегося фрагмента):
- Извлекает `media_type` из `payload` команды и вызывает `validate_media_type`.
- Требует наличия `attachment_id` или `blob_id` в `payload`; иначе `InvalidRequest`.
- Если указан `attachment_id`:
  - Получает импортированное вложение через `CommunicationBlobMetadataPort::imported_attachment_by_id`.
- Если указан только `blob_id`:
  - Пытается найти импортированное вложение по `blob_id`, при неудаче загружает блоб через `blob_by_id` и конструирует `ImportedCommunicationAttachment` с `source_kind = "blob_reuse"`.
- Проверяет, что `storage_kind` импортированного вложения равен `"local_fs"`, иначе `InvalidRequest`.
- Отклоняет вложение со статусом `scan_status == "malicious"`.
- Формирует путь к локальному файлу как `DEFAULT_MAIL_SYNC_BLOB_ROOT / storage_path`.
- Собирает и возвращает `TelegramPreparedMediaSendRequest`.

> **Примечание:** исходный файл был обрезан на отметке 12 000 символов; полное тело функции `media_send_request` отсутствует.

---

## Workflow Action Person Projection

**Файл:** `backend/src/workflows/workflow_action_person_projection.rs`

### Функция

**`create_person_projection_in_transaction`**
Сигнатура:
```rust
pub(crate) async fn create_person_projection_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    email: &str,
    display_name: Option<&str>,
    message: Option<&ProjectedMessage>,
) -> Result<String, PersonProjectionError>
```
Поведение:
- Вызывает `PersonProjectionPort::upsert_email_person_in_transaction` для получения (или создания) персоны и identity.
- Если передан `message`, использует `message.observation_id` как идентификатор наблюдения; иначе формирует новое наблюдение через `ObservationPort::capture_in_transaction` с типом `PERSON_MUTATION`, источником `Manual`, и метаданными о команде и событии.
- Связывает email-персону с проекцией через `PersonProjectionPort::link_email_person_projection_in_transaction`, указывая `source_kind = "workflow_action_projection"`.
- Возвращает `person.person_id`.

---

## Yandex Telemost Calendar Matching

**Файл:** `backend/src/workflows/yandex_telemost_calendar_matching.rs`

### Константы

- **`YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER`** = `"yandex_telemost_calendar_matching"`
- **`YANDEX_TELEMOST_CALENDAR_MATCHING_PROJECTION`** = `"yandex_telemost_calendar_matching"`
- **`YANDEX_TELEMOST_CALENDAR_RELATION_TYPE`** = `"conference_call"`
- **`YANDEX_TELEMOST_CALENDAR_PARTICIPANT_SOURCE`** = `"yandex_telemost_cohost_observed"`

### Вспомогательные структуры

- **`TelemostCohostObservation`** – содержит одно поле `email: String`.

### Ошибки

- **`YandexTelemostCalendarMatchingWorkflowError`**
  Варианты: `Json`, `Sqlx`, `Calendar`, `CalendarCore`, `Observation`, `MissingPayloadField`.

### Функции

#### `project_yandex_telemost_calendar_matching_event`
Входная точка для подписки на события. Вызывает `project_yandex_telemost_calendar_matching`.

#### `project_yandex_telemost_calendar_matching`
Сигнатура:
```rust
pub async fn project_yandex_telemost_calendar_matching(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), YandexTelemostCalendarMatchingWorkflowError>
```
- Фильтрует события через `supports_yandex_telemost_calendar_matching_event`.
- Для события `COHOSTS_OBSERVED` вызывает `project_yandex_telemost_cohosts_into_calendar`.
- Для остальных поддерживаемых событий:
  - Извлекает из payload `conference.id` и `conference.join_url`.
  - Ищет событие календаря через `CalendarEventQueryPort::find_yandex_telemost_conference_match`.
  - При нахождении создаёт наблюдение `CALENDAR_EVENT` с информацией о совпадении.
  - Связывает конференцию с календарным событием через `EventRelationPort::link_with_observation` с типом `"conference_call"`.

#### `project_yandex_telemost_cohosts_into_calendar`
```rust
async fn project_yandex_telemost_cohosts_into_calendar(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), YandexTelemostCalendarMatchingWorkflowError>
```
- Извлекает `conference_id` и массив `cohosts` (список email-ов).
- Находит календарное событие по `conference_id`.
- Загружает существующих участников через `EventParticipantPort::list`, строит множество известных email-ов (lowercase).
- Для каждого cohost с новым email:
  - Создаёт наблюдение `CALENDAR_EVENT`.
  - Добавляет участника с ролью `"attendee"`, источником `"yandex_telemost_cohost_observed"` через `add_with_observation`.

#### `supports_yandex_telemost_calendar_matching_event`
```rust
pub fn supports_yandex_telemost_calendar_matching_event(event_type: &str) -> bool
```
Поддерживаемые типы:
- `CONFERENCE_CREATED`
- `CONFERENCE_OBSERVED`
- `CONFERENCE_UPDATED`
- `COHOSTS_OBSERVED`

#### Вспомогательные функции
- **`required_string`** / **`required_nested_string`** – извлекают строковое поле из JSON с обрезкой пробелов; при отсутствии или пустой строке возвращают ошибку `MissingPayloadField`.

---

## Zoom Calendar Matching

**Файл:** `backend/src/workflows/zoom_calendar_matching.rs`

### Константы

- **`ZOOM_CALENDAR_MATCHING_CONSUMER`** = `"zoom_calendar_matching"`
- **`ZOOM_CALENDAR_MATCHING_PROJECTION`** = `"zoom_calendar_matching"`
- **`ZOOM_CALENDAR_RELATION_TYPE`** = `"conference_call"`

### Ошибки

- **`ZoomCalendarMatchingWorkflowError`**
  Варианты: `Json`, `Sqlx`, `Calendar`, `CalendarCore`, `Observation`, `MissingPayloadField`.

### Функции

#### `project_zoom_calendar_matching_event`
Входная точка потребителя событий. Вызывает `project_zoom_calendar_matching`.

#### `project_zoom_calendar_matching`
Сигнатура:
```rust
pub async fn project_zoom_calendar_matching(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomCalendarMatchingWorkflowError>
```
- Обрабатывает только события типа `MEETING_OBSERVED` (из `zoom_event_types`).
- Извлекает:
  - `call_id` из `subject`.
  - `meeting_id` из `payload`.
  - Опционально: `join_url`, `started_at` (RFC 3339 → `DateTime<Utc>`), `ended_at`.
- Ищет событие календаря через `CalendarEventQueryPort::find_zoom_conference_match`.
- При нахождении:
  - Создаёт наблюдение `CALENDAR_EVENT`.
  - Связывает встречу с календарным событием через `EventRelationPort::link_with_observation` с типом `"conference_call"`.

#### Вспомогательные функции
- **`required_subject_string`**, **`required_payload_string`** – извлекают обязательное строковое поле.
- **`optional_payload_string`**, **`optional_payload_datetime`** – извлекают опциональные поля.

---

## Zoom Participant Identity

**Файл:** `backend/src/workflows/zoom_participant_identity.rs`

### Константы

- **`ZOOM_PARTICIPANT_IDENTITY_CONSUMER`** = `"zoom_participant_identity"`
- **`ATTACH_EMAIL_CANDIDATE_LIMIT_PER_PARTICIPANT`** = `10`
- **`ATTACH_EMAIL_CANDIDATE_CONFIDENCE`** = `0.68`

### Вспомогательные структуры

- **`ZoomParticipantObservation`**
  Поля: `display_name: Option<String>`, `email: Option<String>`.

### Ошибки

- **`ZoomParticipantIdentityWorkflowError`**
  Варианты: `Json`, `PersonsIdentity`, `MissingPayloadField`.

### Функции

#### `project_zoom_participant_identity_event`
Входная точка потребителя. Вызывает `project_zoom_participant_identity`.

#### `project_zoom_participant_identity`
Сигнатура:
```rust
pub async fn project_zoom_participant_identity(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomParticipantIdentityWorkflowError>
```
- Обрабатывает только события `MEETING_OBSERVED`.
- Извлекает `meeting_id`, опциональный `topic` и массив `participants` (десериализуется в `Vec<ZoomParticipantObservation>`).
- Для каждого участника, у которого есть и `display_name`, и `email` (после trim и проверки на пустоту):
  - Формирует строку `evidence_summary` с упоминанием meeting_id и topic (если есть).
  - Вызывает `PersonIdentityPort::suggest_attach_email_candidates` с параметрами:
    - `confidence = 0.68`
    - `limit = 10`

#### Вспомогательные функции
- **`required_payload_string`**, **`optional_payload_string`** – извлечение строк из JSON.

---

## Zoom Signal Detection

**Файл:** `backend/src/workflows/zoom_signal_detection.rs`

### Константы

- **`ZOOM_SIGNAL_DETECTION_CONSUMER`** = `"zoom_signal_detection"`

### Ошибки

- **`ZoomSignalDetectionWorkflowError`**
  Варианты: `SignalHub`, `EventLog`, `EventEnvelope`, `MissingField`.

### Функции

#### `project_zoom_signal_detection_event`
Входная точка потребителя событий. Вызывает `project_zoom_signal_detection`.

#### `project_zoom_signal_detection`
Сигнатура:
```rust
pub async fn project_zoom_signal_detection(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomSignalDetectionWorkflowError>
```
- Пытается построить raw-сигнал через `build_zoom_raw_signal`. Если событие не относится к поддерживаемым, возвращает `Ok(())`.
- Сохраняет сигнал идемпотентно через `EventLogPort::append_for_dispatch_idempotent`.
- Загружает сохранённый raw-сигнал по `event_id`.
- Проверяет через `signal_hub_raw_dispatcher_allows_processing`, можно ли продолжать обработку.
- Если можно – передаёт raw-сигнал в `SignalHubSignalService::process_raw_signal`.

#### `build_zoom_raw_signal`
```rust
fn build_zoom_raw_signal(
    event: &EventEnvelope,
) -> Result<Option<NewEventEnvelope>, ZoomSignalDetectionWorkflowError>
```
- Определяет `event_kind` через `zoom_signal_event_kind` (см. ниже).
- Извлекает `account_id` из `event.source`.
- Строит `subject` через `zoom_raw_signal_subject` в зависимости от `event_kind`.
- Формирует `NewEventEnvelope` с типом `"signal.raw.zoom.{event_kind}.observed"`, передавая исходный payload и причинность (`causation_id`).

#### `zoom_signal_event_kind`
Сопоставление типов событий Zoom с ключевыми словами:
- `MEETING_OBSERVED` → `"meeting"`
- `RECORDING_OBSERVED` → `"recording"`
- `TRANSCRIPT_OBSERVED` → `"transcript"`
- Прочие → `None` (событие не преобразуется в сигнал).

#### `zoom_raw_signal_subject`
Строит JSON-объект subject, содержащий:
- `kind: "signal"`, `source_code: "zoom"`, `account_id`, `zoom_event_id`, `zoom_event_type`.
- В зависимости от `event_kind` добавляет идентификаторы:
  - Для `"meeting"` – `entity_id`, `call_id`, `meeting_id`.
  - Для `"recording"` – `entity_id`, `recording_id`, `meeting_id`.
  - Для `"transcript"` – `entity_id`, `transcript_id`, `call_id`, `meeting_id`.

#### `zoom_raw_signal_event_id`
```rust
fn zoom_raw_signal_event_id(zoom_event_id: &str) -> String
```
- Вычисляет SHA256 от `zoom_event_id` и формирует идентификатор `"evt_signal_raw_zoom_{hex}"`, где `{hex}` – hex-представление хеша.
```

## Покрытие источников

- **`backend/src/workflows/telegram_media_storage.rs`** (truncated)
  Покрыты: структуры `TelegramMediaDownloadData`, `TelegramDownloadedFileData`, `TelegramAttachmentAnchor`, `TelegramMediaDownloadProjection`, `TelegramProviderMediaCommand`, `TelegramPreparedMediaSendRequest`; перечисление `TelegramMediaStorageError`; функция `persist_downloaded_media` полностью; функция `media_send_request` частично (до точки обрезки). Поведение документировано в точности по видимому коду.

- **`backend/src/workflows/workflow_action_person_projection.rs`**
  Покрыта функция `create_person_projection_in_transaction` со всем её поведением, включая формирование наблюдения и связывание персоны.

- **`backend/src/workflows/yandex_telemost_calendar_matching.rs`**
  Покрыты: все константы; структура `TelemostCohostObservation`; перечисление ошибок; функции `project_yandex_telemost_calendar_matching_event`, `project_yandex_telemost_calendar_matching`, `project_yandex_telemost_cohosts_into_calendar`, `supports_yandex_telemost_calendar_matching_event`; вспомогательные функции извлечения полей. Описана логика обработки событий и добавления участников.

- **`backend/src/workflows/zoom_calendar_matching.rs`**
  Покрыты: константы; ошибки; функции `project_zoom_calendar_matching_event`, `project_zoom_calendar_matching`; вспомогательные функции для работы с JSON и датами. Описана логика поиска совпадения и связывания.

- **`backend/src/workflows/zoom_participant_identity.rs`**
  Покрыты: константы (лимит, уверенность); структура `ZoomParticipantObservation`; ошибки; функции `project_zoom_participant_identity_event`, `project_zoom_participant_identity`; вспомогательные функции. Описан процесс формирования кандидатов для привязки email.

- **`backend/src/workflows/zoom_signal_detection.rs`**
  Покрыты: константа; ошибки; функции `project_zoom_signal_detection_event`, `project_zoom_signal_detection`, `build_zoom_raw_signal`, `zoom_signal_event_kind`, `zoom_raw_signal_subject`, `zoom_raw_signal_event_id`. Описаны формирование raw-сигнала и передача в SignalHub.

## Исходные файлы

- [`backend/src/workflows/telegram_media_storage.rs`](../../../../backend/src/workflows/telegram_media_storage.rs)
- [`backend/src/workflows/workflow_action_person_projection.rs`](../../../../backend/src/workflows/workflow_action_person_projection.rs)
- [`backend/src/workflows/yandex_telemost_calendar_matching.rs`](../../../../backend/src/workflows/yandex_telemost_calendar_matching.rs)
- [`backend/src/workflows/zoom_calendar_matching.rs`](../../../../backend/src/workflows/zoom_calendar_matching.rs)
- [`backend/src/workflows/zoom_participant_identity.rs`](../../../../backend/src/workflows/zoom_participant_identity.rs)
- [`backend/src/workflows/zoom_signal_detection.rs`](../../../../backend/src/workflows/zoom_signal_detection.rs)

## Кандидаты на drift

Из предоставленного контекста дрифт между кодом и документацией не виден, так как предыдущая версия wiki-страницы `components/backend.md` не встроена в этот чанк, а ссылки на ADR или другие документы отсутствуют. Видимых расхождений между разными выданными файлами также не обнаружено.
