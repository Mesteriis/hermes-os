## Summary / Резюме

Чанк добавляет в русскую wiki документацию по backend-движкам (`search`, `speaker_identity`, `timeline`, `trust`) и интеграциям (`ai_runtime`, `mail/accounts`).  
Страница `components/backend.md` должна быть обновлена описанием соответствующих компонентов — их основных структур, методов, моделей и ошибок.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Компоненты backend

## Поисковый движок

Расположение: `backend/src/engines/search/`

### `SearchIndex`

Управляет индексом Tantivy.

- **`open_or_create(path)`** — открывает или создаёт индекс по переданному пути, используя `MmapDirectory`.
- **`upsert_document(document)`** — валидирует документ (`object_id`, `object_kind`, `title` непустые), удаляет предыдущую версию по составному ключу `"search:v1:{object_kind}:{object_id}"` и добавляет новую.
- **`commit()`** — фиксирует изменения писателя и перезагружает читатель.
- **`search(query, limit)`** — выполняет поиск по полям `title` и `body` с парсером запросов Tantivy; возвращает `Vec<SearchResult>`.  
  Валидация: `query` непустой, `limit` > 0.

Память писателя: `INDEX_WRITER_MEMORY_BUDGET_BYTES = 50_000_000` (байт).

### Модели

- **`SearchDocument`** — поля `object_id`, `object_kind`, `title`, `body`. Непустые `object_id`, `object_kind`, `title` обязательны.
- **`SearchResult`** — поля `object_id`, `object_kind`, `title`.
- **`SearchFields`** — схема Tantivy с полями `object_identity` (`STRING`), `object_id` (`STRING | STORED`), `object_kind` (`STRING | STORED`), `title` (`TEXT | STORED`), `body` (`TEXT`). Формирует ключ идентичности через `object_identity(document)`: `"search:v1:{len(kind)}:{kind}:{len(id)}:{id}"`.

### Ошибки `SearchError`

- `Tantivy` (прозрачная)
- `OpenDirectory` (прозрачная)
- `QueryParser` (прозрачная)
- `EmptyField` — поле не может быть пустым
- `InvalidLimit` — лимит должен быть > 0
- `WriterLockPoisoned` — мьютекс писателя отравлен
- `MissingStoredField` — в результате отсутствует ожидаемое хранимое поле

---

## Движок идентификации говорящего

Расположение: `backend/src/engines/speaker_identity/`

### `SpeakerIdentityEngine`

Метод `merge(evidence: &[SpeakerEvidence]) -> SpeakerIdentityMergePlan`.

Алгоритм:
1. Группирует свидетельства по `person_id` (при отсутствии — по нормализованной метке `label`).  
   Нормализация: приведение к нижнему регистру, замена не‑алфавитно‑цифровых на `-`, склейка разделителей. Если результат пуст — `"unknown-speaker"`.
2. Для каждой группы вычисляет:
   - взвешенную уверенность (confidence) как средневзвешенное `confidence` с весами источников, где `confidence` ограничен `[0.0, 1.0]`;
   - `display_label` — первая непустая метка;
   - `person_id` — первый встреченный;
   - `requires_review = confidence < 0.8`.
3. Формирует `SpeakerIdentityMergePlan` со списком кандидатов, количеством неизвестных говорящих и политикой `"dom_webview_hints_are_supporting_evidence_not_truth"`.

### Веса источников (`source_weight`)

| Источник | Вес |
|---|---|
| `ManualConfirmation` | 1.0 |
| `VoiceEmbedding` | 0.85 |
| `CalendarAttendee` | 0.55 |
| `ProviderParticipant` | 0.5 |
| `WhisperDiarization` | 0.45 |
| `WebviewDomHint` | 0.25 |

### Модели

- **`SpeakerEvidence`** — `source`, `label`, `person_id` (опционально), `starts_at_ms` / `ends_at_ms` (опционально), `confidence` (f32), `evidence` (serde_json::Value).
- **`SpeakerIdentityCandidate`** — `speaker_key`, `display_label`, `person_id` (опционально), `confidence`, `evidence_count`, `requires_review`.
- **`SpeakerIdentityMergePlan`** — `candidates: Vec<SpeakerIdentityCandidate>`, `unknown_speaker_count: usize`, `policy: String`.

---

## Движок временной шкалы

Расположение: `backend/src/engines/timeline/`

### `TimelineEngine`

Набор аналитических методов над событиями (`TimelineEventDraft`) и хранимыми событиями (`StoredEventEnvelope`).  
Все методы валидируют события через `validate_event` (все строковые поля непустые, `occurred_at` валидный `DateTime<Utc>`), а сущностные поля — через `validate_non_empty`.

- **`bounded_entity_limit(limit)`** — ограничивает лимит диапазоном `[1, 100]`.
- **`validate_event(event)`** — проверяет непустоту `entity_kind`, `entity_id`, `event_type`, `title`, `source` и корректность `occurred_at`.
- **`period_summary(events, period_start, period_end)`** — возвращает `TimelinePeriodSummary` с общим количеством событий и распределением по `entity_kind` и `event_type`.
- **`recency_signal(events, entity_kind, entity_id, as_of)`** — находит последнее событие до `as_of` для указанной сущности и возвращает `TimelineRecencySignal` (время последнего события, тип, источник, возраст в секундах).
- **`timeline_gaps(events, entity_kind, entity_id, period_start, period_end, max_gap_seconds)`** — выявляет промежутки между событиями, превышающие `max_gap_seconds` (должен быть > 0). Возвращает `Vec<TimelineGap>`.
- **`change_diff(previous_events, current_events, entity_kind, entity_id)`** — сравнивает два набора событий по источнику (`source`). Возвращает `TimelineChangeDiff` с добавленными и удалёнными `TimelineChange`.
- **`cross_domain_timeline(events, period_start, period_end, limit)`** — возвращает упорядоченный по времени список `TimelineEntry` за период, с ограничением по лимиту.
- **`replay_event_log(stored_events, period_start, period_end, limit)`** — воспроизводит лог событий из хранимых конвертов `StoredEventEnvelope` (из платформы), извлекая `entity_kind` и `entity_id` из JSON‑поля `subject`. Возвращает `TimelineReplay`.
- **`run_event_log_projection(events, cursors, projection_name, period_start, period_end, batch_size, timeline_limit)`** (**async**) — выполняет пакетную проекцию событий с использованием `run_projection_batch`. Возвращает `TimelineProjectionRun`.

### Модели

- **`TimelineEventDraft<'a>`** — `entity_kind`, `entity_id`, `event_type`, `title`, `occurred_at`, `source` (все как `&str`).
- **`TimelinePeriodSummary`** — `period_start`, `period_end`, `total_events`, `by_entity_kind: BTreeMap<String, usize>`, `by_event_type: BTreeMap<String, usize>`.
- **`TimelineRecencySignal`** — `entity_kind`, `entity_id`, `last_event_at` (optional), `last_event_type` (optional), `last_event_source` (optional), `age_seconds` (optional).
- **`TimelineGap`** — `entity_kind`, `entity_id`, `gap_start`, `gap_end`, `gap_seconds`, `previous_event_source` (optional), `next_event_source` (optional).
- **`TimelineChangeDiff`** — `entity_kind`, `entity_id`, `added: Vec<TimelineChange>`, `removed: Vec<TimelineChange>`.
- **`TimelineChange`** — `event_type`, `occurred_at`, `source`.
- **`TimelineEntry`** — `entity_kind`, `entity_id`, `event_type`, `title`, `occurred_at`, `source`.
- **`TimelineReplay`** — `last_replayed_position: i64`, `entries: Vec<TimelineEntry>`.
- **`TimelineProjectionRun`** — `processed_count: usize`, `last_processed_position: i64`, `entries: Vec<TimelineEntry>`.

### Ошибки

- **`TimelineEngineError`** — `EmptyField`, `InvalidPeriod` (начало > конец), `InvalidGapThreshold` (≤ 0), `InvalidEventLogField` (невалидное поле в `StoredEventEnvelope`).
- **`TimelineProjectionError`** — `Runner` (оборачивает `ProjectionRunnerError`) или `Timeline` (оборачивает `TimelineEngineError`).

### Вспомогательная валидация (`validation`)

- `validate_non_empty` — поле непустое.
- `required_json_string` — извлекает обязательное строковое поле из `Value` с детальной ошибкой `InvalidEventLogField`.
- `optional_json_string` — извлекает опциональное строковое поле.
- `event_log_source_ref` — формирует строку `"{kind}:{source_id}"` из поля `source`, если оба есть; иначе возвращает `event_id`.

---

## Движок доверия

Расположение: `backend/src/engines/trust/`

### `TrustEngine`

- **`persona_compatibility_score_signal(score: i16) -> TrustRelationshipSignal`** с фиксированными полями:  
  `kind = PersonaCompatibilityScore`, `relationship_type = "trusts"`, `strength_score = 0.5`, `confidence = 1.0`,  
  `trust_score = normalize_compatibility_score(score)`.

- **`source_reliability_signal(affected_source, evidence, confidence) -> TrustSourceReliabilitySignal`** — проверяет непустоту `affected_source` и `evidence`, уверенность `[0.0, 1.0]`. Направление воздействия `direction = Positive`, если `confidence >= 0.5`, иначе `Negative`.

### Модели

- **`TrustSignalKind`** — `PersonaCompatibilityScore` (`as_str() → "persona_compatibility_score"`), `SourceReliability` (`as_str() → "source_reliability"`).
- **`TrustRelationshipSignal`** — `kind`, `relationship_type`, `trust_score`, `strength_score`, `confidence`, `explanation` (все литералы).
- **`TrustSourceReliabilitySignal`** — `kind`, `affected_source`, `evidence`, `confidence`, `direction`, `explanation`.
- **`TrustImpactDirection`** — `Positive` (строка `"positive"`), `Negative` (строка `"negative"`). Метод `from_confidence(confidence)`.

### Вспомогательные функции

- **`normalize_compatibility_score(score)`** — ограничивает `[0, 100]`, делит на 100, умножает на 10000, округляет, делит на 10000.
- **`validate_non_empty`**, **`validate_confidence`** — проверки для `source_reliability_signal`.

### Ошибки `TrustEngineError`

- `EmptyField` — поле непустое.
- `InvalidConfidence` — уверенность вне `[0.0, 1.0]`.

---

## Интеграция с AI Runtime

Расположение: `backend/src/integrations/ai_runtime.rs`

### `AiRuntimeClient`

Перечисление, объединяющее клиентов `OllamaClient` и `OmniRouteClient`.  
Реализует трейт `AiRuntimePort`.

Методы (все **async**):
- **`runtime_name()`** — `"ollama"` или `"omniroute"`.
- **`chat_model()`**, **`embedding_model()`** — модель по умолчанию (делегируется внутреннему клиенту).
- **`version()`** — возвращает версию для Ollama; для OmniRoute всегда `Ok(None)`.
- **`models()`** — список доступных моделей.
- **`validate_required_models()`** — проверка наличия обязательных моделей.
- **`chat(prompt)`** — вызов чата с моделью по умолчанию (`chat_model()`).
- **`chat_with_model(prompt, model)`** — явный выбор модели.
- **`embed(input)`** — эмбеддинг с моделью по умолчанию (`embedding_model()`).
- **`embed_with_model(input, model)`** — явный выбор модели.

Результаты преобразуются в `AiChatResult` / `AiEmbedResult` с полем `total_duration_ns: Option<i64>` (OmniRoute не возвращает длительность).

### Ошибки

- **`AiRuntimeError`** — `Ollama(OllamaError)` или `OmniRoute(OmniRouteError)`.
- Конвертация в `AiRuntimePortError` через `AiRuntimePortError::provider(name, message)`.

---

## Интеграция с почтовыми аккаунтами

Расположение: `backend/src/integrations/mail/accounts/`

Модуль предоставляет сервис настройки учётных записей электронной почты (`EmailAccountSetupService`).

### Константы OAuth Google

- `DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT` — `"https://accounts.google.com/o/oauth2/v2/auth"`
- `DEFAULT_GOOGLE_TOKEN_ENDPOINT` — `"https://oauth2.googleapis.com/token"`
- Набор scopes `DEFAULT_GOOGLE_WORKSPACE_SCOPES`:
  - `https://www.googleapis.com/auth/gmail.readonly`
  - `https://www.googleapis.com/auth/gmail.send`
  - `https://www.googleapis.com/auth/calendar.readonly`
  - `https://www.googleapis.com/auth/contacts.readonly`

### Модели запросов/ответов

- `GmailOAuthSetupRequest`
- `GmailOAuthPendingGrant`
- `ImapAccountSetupRequest`
- `EmailAccountSetupResult`

### Ошибки `EmailAccountSetupError`

- `InvalidRequest { field, message }`
- `MissingProviderField { field }`
- `StoresNotConfigured`
- `Http(reqwest::Error)`
- `Json(serde_json::Error)`
- `DatabaseVault(DatabaseEncryptedVaultError)`
- `HostVault(HostVaultError)`
- `SecretReference(SecretReferenceError)`
- `Secret(SecretResolutionError)`
- `ProviderAccountStore(String)`
```

## Source coverage / Покрытие источников

- `backend/src/engines/search/engine.rs` — структура `SearchIndex`, её методы и константа памяти, логика `upsert_document` (удаление по identity, добавление полей), `commit`, `search`.
- `backend/src/engines/search/errors.rs` — перечисление `SearchError` и его варианты.
- `backend/src/engines/search/mod.rs` — публичный API модуля.
- `backend/src/engines/search/models.rs` — `SearchDocument`, `SearchResult`, `SearchFields`, `object_identity`, `document_identity_term`, `required_stored_text`, `validate_non_empty`.
- `backend/src/engines/speaker_identity/engine.rs` — `SpeakerIdentityEngine::merge`, алгоритм группировки, вычисление весов, `source_weight`, `normalize_label`.
- `backend/src/engines/speaker_identity/mod.rs` — публичный API.
- `backend/src/engines/speaker_identity/models.rs` — `SpeakerIdentitySource`, `SpeakerEvidence`, `SpeakerIdentityCandidate`, `SpeakerIdentityMergePlan`.
- `backend/src/engines/timeline.rs` — `TimelineEngine` и её публичные методы, используемые типы платформы.
- `backend/src/engines/timeline/analysis.rs` — реализации `recency_signal`, `timeline_gaps`, `change_diff` и вспомогательная `events_by_source`.
- `backend/src/engines/timeline/cross_domain.rs` — `cross_domain_timeline`.
- `backend/src/engines/timeline/errors.rs` — `TimelineEngineError`, `TimelineProjectionError`.
- `backend/src/engines/timeline/models.rs` — все модели данных временной шкалы.
- `backend/src/engines/timeline/policy.rs` — `bounded_entity_limit`, `validate_event`.
- `backend/src/engines/timeline/projection.rs` — `run_event_log_projection` (async, использующий `run_projection_batch`).
- `backend/src/engines/timeline/replay.rs` — `replay_event_log`.
- `backend/src/engines/timeline/summaries.rs` — `period_summary`.
- `backend/src/engines/timeline/validation.rs` — `validate_non_empty`, `required_json_string`, `optional_json_string`, `event_log_source_ref`.
- `backend/src/engines/trust/engine.rs` — `TrustEngine::persona_compatibility_score_signal`, `source_reliability_signal`.
- `backend/src/engines/trust/errors.rs` — `TrustEngineError`.
- `backend/src/engines/trust/mod.rs` — публичный API.
- `backend/src/engines/trust/models.rs` — `TrustSignalKind`, `TrustRelationshipSignal`, `TrustSourceReliabilitySignal`, `TrustImpactDirection`, `normalize_compatibility_score`, валидационные функции.
- `backend/src/integrations/ai_runtime.rs` — `AiRuntimeClient`, методы, преобразование ошибок, реализация `AiRuntimePort`.
- `backend/src/integrations/mail/accounts.rs` — реэкспорт публичного API модуля `accounts`.
- `backend/src/integrations/mail/accounts/constants.rs` — константы OAuth и Google scopes.
- `backend/src/integrations/mail/accounts/errors.rs` — `EmailAccountSetupError` и все варианты.

## Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения не видны.  
Включены только файлы исходного кода backend; отсутствуют существующие страницы wiki, ADR или другие документы, с которыми можно было бы сравнить.
