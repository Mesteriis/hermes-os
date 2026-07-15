### Summary / Резюме

Данный чанк содержит исходный код движков бэкенда `enrichment`, `identity_resolution`, `memory`, `obligation`, `relationships`, `risk` и перечень всех движков из `backend/src/engines/mod.rs`. Страница `components/backend.md` должна быть обновлена, чтобы отразить актуальные публичные API, модели, ошибки и логику этих движков. Для остальных движков исходный код отсутствует, поэтому их документация не может быть подтверждена в рамках данного чанка.

### Proposed pages / Предлагаемые страницы

```
## components/backend.md

# Компоненты бэкенда

> Этот документ описывает движки (engines) доменной логики бэкенда. Основан на актуальном исходном коде из `backend/src/engines`.

## Движки

Все движки перечислены в `backend/src/engines/mod.rs`:

```rust
pub mod automation;
pub mod call_intelligence;
pub mod consistency;
pub mod context_packs;
pub mod enrichment;
pub mod identity_resolution;
pub mod memory;
pub mod obligation;
pub mod relationships;
pub mod risk;
pub mod search;
pub mod speaker_identity;
pub mod timeline;
pub mod trust;
```

В рамках данного чанка предоставлены исходные файлы только для движков `enrichment`, `identity_resolution`, `memory`, `obligation`, `relationships`, `risk`. Документация по ним приведена ниже. Для остальных движков контекст отсутствует.

### memory

Модуль: `backend/src/engines/memory`.

Публичный интерфейс (`MemoryEngine`, структура без полей):

- `persona_notes_memory_card(person_id: &str, notes: &str) -> Option<MemoryCardDraft>`  
  Создаёт черновик карточки памяти «Compatibility notes» для персоны на основе заметок. Если `notes` пустые, возвращает `None`.  
  Поля `MemoryCardDraft`: `title`, `description`, `source` (формат `persons.notes:{person_id}`), `confidence` (фиксировано `1.0`), `importance` (5).

- `persona_fact_memory(person_id: &str, fact_type: &str, value: &str, source: &str, confidence: f64) -> Result<MemoryFactDraft, MemoryEngineError>`  
  Создаёт черновик факта памяти с `affected_entity_kind = "persona"`, `review_state = "accepted"`, `produced_by = "memory_engine"`.  
  Валидирует непустоту `person_id`, `fact_type`, `value`, `source` и принадлежность `confidence` диапазону `[0.0, 1.0]`.

- `context_pack(affected_entity_kind: &str, affected_entity_id: &str, facts: &[MemoryFactDraft], cards: &[MemoryCardDraft], limit: i64) -> Result<MemoryContextPack, MemoryEngineError>`  
  Формирует контекстный пак для сущности.  
  Отбирает факты, относящиеся к сущности (`affected_entity_kind` и `affected_entity_id` совпадают с обрезанными значениями), и все переданные карточки памяти.  
  Для каждой карточки создаётся `MemoryContextItem` с `item_kind = "memory_card"`, для факта — `item_kind = "fact"`.  
  Сортировка по убыванию `confidence`, затем по `item_kind`, затем по `source`.  
  Обрезает результат до `limit` (зажат в `[1, 50]`).  
  Агрегированное `confidence` — среднее арифметическое по всем элементам, округлённое до двух знаков.  
  `source_citations` — уникальные источники в порядке первого вхождения.

- `memory_gaps(affected_entity_kind: &str, affected_entity_id: &str, required_fact_types: &[&str], facts: &[MemoryFactDraft]) -> Result<Vec<MemoryGap>, MemoryEngineError>`  
  Вычисляет недостающие типы фактов.  
  Для каждого `required_fact_types` после дедупликации проверяет, есть ли у сущности принятый (`review_state = "accepted"`) факт с таким типом. Отсутствующие типы превращаются в `MemoryGap` с `source` вида `memory_engine:gap:{entity_kind}:{entity_id}:{fact_type}` и `review_state = "suggested"`.

- `stale_memory_candidates(affected_entity_kind: &str, affected_entity_id: &str, facts: &[MemoryFactState], as_of: DateTime<Utc>, stale_after_days: i64) -> Result<Vec<MemoryStaleCandidate>, MemoryEngineError>`  
  Идентифицирует устаревшие факты.  
  Для каждого факта с `review_state = "accepted"` и `last_verified_at` раньше `as_of - stale_after_days` (или отсутствующим) создаёт `MemoryStaleCandidate` с `review_state = "suggested"`.  
  `stale_after_days` должен быть положительным, иначе `MemoryEngineError::InvalidStaleThreshold`.  
  Сортировка: по возрастанию `last_verified_at`, затем по `source`.

- `cross_domain_context_pack(root_entity_kind: &str, root_entity_id: &str, related_entities: &[MemoryEntityRef], sources: &[MemoryContextSource], limit: i64) -> Result<CrossDomainMemoryContextPack, MemoryEngineError>`  
  Строит кросс-доменный контекстный пак.  
  Принимает корневую сущность, список связанных сущностей с атрибутом `relation_kind`, и элементы-источники (`MemoryContextSource`).  
  Элементы фильтруются по `review_state = "accepted"`.  
  Ранг сущности: 0 для корневой (`relation_kind = "self"`), для связанных — `index + 1` с `relation_kind` из соответствующего `MemoryEntityRef`. Элементы без сопоставления отбрасываются.  
  Сортировка: по возрастанию ранга, по убыванию `confidence`, по `source`.  
  `limit` зажат в `[1, 50]`.  
  Результат содержит `entity_citations` (вида `entity_kind:entity_id`) и `source_citations`.

**Типы, определённые в модуле:**

- `MemoryCardDraft` — `title`, `description`, `source`, `confidence`, `importance`.
- `MemoryFactDraft` — `affected_entity_kind`, `affected_entity_id`, `fact_type`, `value`, `source`, `confidence`, `review_state`, `produced_by`.
- `MemoryFactState` — как `MemoryFactDraft` + `last_verified_at: Option<DateTime<Utc>>`.
- `MemoryContextPack` — `affected_entity_kind`, `affected_entity_id`, `items: Vec<MemoryContextItem>`, `source_citations`, `confidence`, `produced_by`.
- `MemoryContextItem` — `item_kind`, `title`, `body`, `source`, `confidence`, `review_state`.
- `MemoryGap` — `affected_entity_kind`, `affected_entity_id`, `missing_fact_type`, `source`, `review_state`, `produced_by`.
- `MemoryStaleCandidate` — `affected_entity_kind`, `affected_entity_id`, `fact_type`, `value`, `source`, `confidence`, `last_verified_at`, `review_state`, `produced_by`.
- `MemoryEntityRef` — `entity_kind`, `entity_id`, `relation_kind`.
- `MemoryContextSource` — `entity_kind`, `entity_id`, `item_kind`, `title`, `body`, `source`, `confidence`, `review_state`.
- `CrossDomainMemoryContextPack` — `root_entity_kind`, `root_entity_id`, `items: Vec<MemoryCrossDomainContextItem>`, `entity_citations`, `source_citations`, `confidence`, `produced_by`.
- `MemoryCrossDomainContextItem` — `entity_kind`, `entity_id`, `relation_kind`, `item_kind`, `title`, `body`, `source`, `confidence`, `review_state`.

**Ошибки:** `MemoryEngineError` — варианты `EmptyField`, `InvalidConfidence`, `InvalidStaleThreshold`.

### obligation

Модуль: `backend/src/engines/obligation`.

Публичный интерфейс:

- `ObligationEngine` (пустая структура) с единственным методом:

```rust
pub fn detect_candidates(input: &ObligationExtractionInput) -> Result<ObligationExtractionResult, ObligationEngineError>
```

Входные данные (`ObligationExtractionInput`) содержат:
- `source_kind: ObligationEvidenceSourceKind` (варианты `Communication`, `Document`, `CalendarEvent`, `Observation`, `Manual`)
- `source_id: String`
- `text: String`
- `obligated_entity_kind: ObligationEntityKind` (перечисление: `Persona`, `Organization`, `Project`, `Communication`, `Document`, `Task`, `Event`, `Decision`, `Obligation`, `Knowledge`)
- `obligated_entity_id: String`
- опциональные `beneficiary_entity_kind` и `beneficiary_entity_id` (обязательно вместе, иначе ошибка `PartialBeneficiary`).
- Методы-конструкторы: `communication(...)`, `document(...)`, `beneficiary(...)` для заполнения бенефициара. Валидация через `input.validate()` проверяет непустоту обязательных полей и согласованность бенефициара.

Алгоритм извлечения (`detect_commitment`):
- Текст разбивается на предложения (разделители `\n`, `.`, `!`, `?`).
- Для каждого предложения проверяется начало (регистронезависимо):
  - `"i will "` → `kind = Commitment`, confidence = 0.84
  - `"i'll "` → `kind = Commitment`, confidence = 0.84
  - `"please "` → `kind = Request`, confidence = 0.76
- Тело после ключевого слова обрезается, удаляются завершающие знаки препинания.
- Из тела выделяются `due_text` (после `" by "` или `" before "`) и `condition` (после `" when "`, `" once "`, `" if "`).
- Минимальная длина утверждения — 3 символа, иначе предложение игнорируется.
- Создаётся `ObligationCandidate`, из которого также формируются `ObligationTaskCandidate` (confidence = исходный минус 0.08, min 0) и `FollowUpCandidate` (confidence = исходный минус 0.12, min 0).

**Типы результата:**

- `ObligationExtractionResult` — поля `obligations: Vec<ObligationCandidate>`, `task_candidates: Vec<ObligationTaskCandidate>`, `follow_ups: Vec<FollowUpCandidate>`.

**Основные типы:**

- `ObligationCandidate` — содержит `kind`, `obligated_entity_kind/id`, `beneficiary_entity_kind/id` (опционально), `statement`, `quote` (завершённое точкой), `due_text`, `condition`, `confidence`, `review_state: ObligationReviewState` (`Suggested`/`UserConfirmed`/`UserRejected`), `evidence_source_kind/id`.
- `ObligationTaskCandidate` — `source_obligation_statement`, `statement`, `suggested_title`, `due_text`, `confidence`.
- `FollowUpCandidate` — `source_obligation_statement`, `prompt` (вида `"Follow up on: ..."`), `due_text`, `confidence`.

**Ошибки:** `ObligationEngineError::EmptyField`, `PartialBeneficiary`.

### relationships

Модуль: `backend/src/engines/relationships/mod.rs` (без публичного движка, только конструктор кандидатов).

Типы:

- `RelationshipSubject` — `entity_kind`, `entity_id` (с конструктором `new`).
- `RelationshipCandidate` — `candidate_id` (формат `relationship_candidate:v1:{src.kind}:{src.id}:{rel}:{tgt.kind}:{tgt.id}`), `source: RelationshipSubject`, `target: RelationshipSubject`, `relationship_type: String`, `confidence: f64`, `evidence_observation_ids: Vec<String>`.

Статический метод-конструктор:

```rust
RelationshipCandidate::linked_entities_candidate(
    source: RelationshipSubject,
    target: RelationshipSubject,
    relationship_type: impl Into<String>,
    confidence: f64,
    evidence_observation_ids: Vec<String>,
) -> Result<Self, RelationshipEngineError>
```

Валидация:
- `entity_kind`, `entity_id`, `relationship_type` непустые.
- `confidence ∈ [0.0, 1.0]`.
- `evidence_observation_ids` непустой, каждый идентификатор непустой.

**Ошибки:** `RelationshipEngineError` — `EmptyField`, `MissingEvidence`, `InvalidConfidence(String)`.

### risk

Модуль: `backend/src/engines/risk`.

Публичный интерфейс `RiskEngine`:

- `derive_attention_status(risks: &[RiskSignal]) -> RiskAttentionStatus`  
  Итерация по неразрешённым (`resolved == false`) рискам:
  - Если встречается `Critical` или `High` → возвращает `AtRisk` немедленно.
  - Если есть `Medium` или `Low` → запоминает флаг, после цикла возвращает `NeedsAttention`.
  - Иначе `Healthy`.

- `persona_observation(person_id: &str, risk_type: &str, evidence: &str, severity: &str, source: &str) -> Result<RiskObservationDraft, RiskEngineError>`  
  Создаёт `RiskObservationDraft` с `affected_entity_kind = "persona"`, `confidence = 0.5`, `suggested_handling_state` из `RiskSeverity::suggested_handling_state()`, `review_state = "suggested"`.  
  Парсит строку `severity` в `RiskSeverity` (поддерживаются `"low"`, `"medium"`, `"high"`, `"critical"`; регистронезависимо).

**Типы:**

- `RiskSeverity` — варианты `Low`, `Medium`, `High`, `Critical`.
  - `suggested_handling_state`: для `Critical`/`High` → `"review_now"`, для `Medium`/`Low` → `"monitor"`.
  - `as_str()` возвращает строковое представление.
  - `parse(s: &str) -> Result<Self, RiskEngineError>` — ошибка `InvalidSeverity` при неизвестной строке.
- `RiskAttentionStatus` — `Healthy`, `NeedsAttention`, `AtRisk`. Имеет метод `as_persona_health_status()` (`"healthy"`, `"needs_attention"`, `"at_risk"`).
- `RiskSignal` — `severity: RiskSeverity`, `resolved: bool`. Конструкторы `unresolved(...)`, `resolved(...)`.
- `RiskObservationDraft` — `affected_entity_kind/id`, `risk_type`, `evidence`, `source`, `confidence`, `severity`, `suggested_handling_state`, `review_state`.

**Ошибки:** `RiskEngineError` — `InvalidSeverity(String)`, `EmptyField`.

### identity_resolution

Модуль: `backend/src/engines/identity_resolution` (определён в одном файле `mod.rs`).

Типы:

- `IdentityResolutionSubject` — `entity_kind`, `entity_id`. Конструктор `new`.
- `IdentityResolutionCandidate` — `candidate_id` (формат `identity_resolution_candidate:v1:{left.kind}:{left.id}:{right.kind}:{right.id}`), `left`, `right` (оба `IdentityResolutionSubject`), `confidence: f64`, `evidence_observation_ids: Vec<String>`.

Статический метод-конструктор:

```rust
IdentityResolutionCandidate::same_entity_candidate(
    left: IdentityResolutionSubject,
    right: IdentityResolutionSubject,
    confidence: f64,
    evidence_observation_ids: Vec<String>,
) -> Result<Self, IdentityResolutionError>
```

Валидация:
- `left` и `right` не должны быть равны (иначе `SameSubject`).
- `confidence ∈ [0.0, 1.0]`.
- `evidence_observation_ids` непустой, каждый идентификатор непустой (иначе `MissingEvidence` или `EmptyField`).

**Ошибки:** `IdentityResolutionError` — `EmptyField`, `SameSubject`, `MissingEvidence`, `InvalidConfidence(String)`.

### enrichment

Модуль: `backend/src/engines/enrichment`.

Публичные элементы (из `mod.rs`): `EnrichmentEngine`, `EnrichmentEngineError`. Реализация `EnrichmentEngine` отсутствует в чанке.

Типы и функции (из `models.rs`):

- `PreferenceDraft` — `preference_type`, `value`, `source`, `confidence`.
- `EnrichmentCandidateDraft` — `entity_kind`, `entity_id`, `source`, `extracted_claim`, `data: serde_json::Value`, `confidence`, `review_state`, `freshness`, `conflict_marker`.
- `validate_non_empty(field, value)` — возвращает `EnrichmentEngineError::EmptyField` при пустом/пробельном значении.
- `validate_confidence(f64)` — требует `0.0..=1.0`, иначе `EnrichmentEngineError::InvalidConfidence`.

**Ошибки:** `EnrichmentEngineError` — `EmptyField(field)`, `InvalidConfidence(f64)`, `InvalidData` (описание: «enrichment candidate data must be a JSON object»).

### Остальные движки

Для движков `automation`, `call_intelligence`, `consistency`, `context_packs`, `search`, `speaker_identity`, `timeline`, `trust` исходный код в данном чанке отсутствует. Документация по ним не может быть подтверждена.
```

### Source coverage / Покрытие источников

| Файл | Факты, покрытые на странице |
|---|---|
| `backend/src/engines/mod.rs` | Полный перечень движков: automation, call_intelligence, consistency, context_packs, enrichment, identity_resolution, memory, obligation, relationships, risk, search, speaker_identity, timeline, trust. |
| `backend/src/engines/enrichment/errors.rs` | Ошибки: `EmptyField`, `InvalidConfidence`, `InvalidData`. |
| `backend/src/engines/enrichment/mod.rs` | Публичный реэкспорт `EnrichmentEngine` и `EnrichmentEngineError`. |
| `backend/src/engines/enrichment/models.rs` | Типы `PreferenceDraft`, `EnrichmentCandidateDraft`, функции `validate_non_empty`, `validate_confidence`. |
| `backend/src/engines/identity_resolution/mod.rs` | Типы `IdentityResolutionSubject`, `IdentityResolutionCandidate`, конструктор `same_entity_candidate`, ошибки `IdentityResolutionError`, правила валидации (пустые поля, обязательное evidence, неравенство субъектов, диапазон confidence). Тест на `MissingEvidence`. |
| `backend/src/engines/memory.rs` | Публичные методы `MemoryEngine`: `persona_notes_memory_card`, `persona_fact_memory`, `context_pack`, `memory_gaps`, `stale_memory_candidates`, `cross_domain_context_pack`. Экспортируемые типы. |
| `backend/src/engines/memory/cards.rs` | Логика `persona_notes_memory_card`: создание карточки с заголовком «Compatibility notes», source = `persons.notes:{person_id}`, confidence = 1.0, importance = 5. |
| `backend/src/engines/memory/context.rs` | Алгоритм `context_pack`: фильтрация фактов по сущности, формирование `MemoryContextItem`, сортировка (confidence desc, item_kind, source), ограничение `limit.clamp(1,50)`, уникальные `source_citations`, агрегация confidence как среднее арифметическое, поле `produced_by = "memory_engine"`. |
| `backend/src/engines/memory/cross_domain.rs` | Алгоритм `cross_domain_context_pack`: ранг сущности (0 для корневой, 1+ для связанных), фильтр по `review_state = "accepted"`, сортировка (ранг, confidence desc, source), ограничение лимита, `entity_citations` и `source_citations`, агрегация confidence. |
| `backend/src/engines/memory/errors.rs` | Ошибки `MemoryEngineError`: `EmptyField`, `InvalidConfidence`, `InvalidStaleThreshold`. |
| `backend/src/engines/memory/facts.rs` | Логика `persona_fact_memory`: фиксированные `affected_entity_kind = "persona"`, `review_state = "accepted"`, `produced_by = "memory_engine"`. Валидация полей и confidence. |
| `backend/src/engines/memory/gaps.rs` | Логика `memory_gaps`: дедупликация `required_fact_types`, поиск принятых фактов сущности, формирование `MemoryGap` с `source` вида `memory_engine:gap:{kind}:{id}:{type}` и `review_state = "suggested"`. |
| `backend/src/engines/memory/models.rs` | Все типы: `MemoryCardDraft`, `MemoryFactDraft`, `MemoryFactState`, `MemoryContextPack`, `MemoryContextItem`, `MemoryGap`, `MemoryStaleCandidate`, `MemoryEntityRef`, `MemoryContextSource`, `CrossDomainMemoryContextPack`, `MemoryCrossDomainContextItem`. |
| `backend/src/engines/memory/stale.rs` | Логика `stale_memory_candidates`: проверка `review_state = "accepted"`, сравнение `last_verified_at` с `as_of - stale_after_days`, ошибка при `stale_after_days <= 0`, сортировка по `last_verified_at` (asc) и `source`. |
| `backend/src/engines/memory/validation.rs` | Функции валидации: `validate_memory_card`, `validate_memory_fact`, `validate_memory_fact_state`, `validate_memory_entity_ref`, `validate_memory_context_source`, `validate_non_empty`, `validate_confidence`. |
| `backend/src/engines/obligation/detection.rs` | Функции `detect_commitment`, `sentences`, `split_due_text`, `split_condition`, `ensure_sentence_terminator`. Правила: префиксы "i will", "i'll", "please"; confidence 0.84 (commitment) и 0.76 (request); маркеры due_text и condition. |
| `backend/src/engines/obligation/engine.rs` | `ObligationEngine::detect_candidates`: валидация входа, обход предложений, вызов `detect_commitment`, заполнение `ObligationTaskCandidate` и `FollowUpCandidate` из каждого кандидата. |
| `backend/src/engines/obligation/errors.rs` | Ошибки: `EmptyField`, `PartialBeneficiary`. |
| `backend/src/engines/obligation/mod.rs` | Публичный экспорт: `ObligationEngine`, `ObligationEngineError`, все типы. |
| `backend/src/engines/obligation/models.rs` | Все типы: `ObligationExtractionInput` (и его конструкторы `communication`, `document`, `beneficiary`, `validate`), `ObligationEntityKind` (10 вариантов), `ObligationEvidenceSourceKind` (5 вариантов), `ObligationReviewState` (3 варианта), `ObligationExtractionResult`, `ObligationCandidateKind`, `ObligationCandidate`, `ObligationTaskCandidate`, `FollowUpCandidate`. Функция `validate_non_empty`. |
| `backend/src/engines/relationships/mod.rs` | Типы `RelationshipSubject`, `RelationshipCandidate`, конструктор `linked_entities_candidate`, ошибки `RelationshipEngineError`, правила валидации (пустые поля, evidence, confidence). Тест на `MissingEvidence`. |
| `backend/src/engines/risk/engine.rs` | Методы `RiskEngine::derive_attention_status` (логика приоритетов) и `persona_observation` (fixed confidence 0.5, `affected_entity_kind = "persona"`, `review_state = "suggested"`). |
| `backend/src/engines/risk/errors.rs` | Ошибки: `InvalidSeverity`, `EmptyField`. |
| `backend/src/engines/risk/mod.rs` | Публичный экспорт: `RiskEngine`, `RiskEngineError`, `RiskAttentionStatus`, `RiskObservationDraft`, `RiskSeverity`, `RiskSignal`. |
| `backend/src/engines/risk/models.rs` | Типы: `RiskAttentionStatus` и его `as_persona_health_status`, `RiskSeverity` (варианты, `parse`, `suggested_handling_state`), `RiskObservationDraft`, `RiskSignal`, функция `validate_non_empty`. |

### Drift candidates / Кандидаты на drift

В предоставленном контексте прямых расхождений между кодом и существующей документацией не обнаружено (исходная wiki-страница не встроена в чанк). Однако есть потенциальные зоны риска:

1. **Отсутствие реализации `EnrichmentEngine`** — модуль `enrichment` реэкспортирует `EnrichmentEngine`, но файл `engine.rs` не включён в чанк. Если wiki описывает методы этого движка, такое описание не может быть верифицировано.

2. **Неподтверждённые движки** — для `automation`, `call_intelligence`, `consistency`, `context_packs`, `search`, `speaker_identity`, `timeline`, `trust` исходный код отсутствует. Если wiki содержит документацию по ним, то между чанком и wiki может существовать drift.

3. **Отсутствие публичной структуры движков `relationships` и `identity_resolution`** — эти модули предоставляют только функции-конструкторы и типы, без обёрточного движка. Если wiki упоминает наличие объекта-движка с состоянием, это не подтверждается кодом.
