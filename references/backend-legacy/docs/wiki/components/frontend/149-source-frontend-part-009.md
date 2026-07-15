---
chunk_id: 149-source-frontend-part-009
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 149-source-frontend-part-009 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/frontend.md` обновляется для документирования frontend-компонентов, представленных в чанке `#149-source-frontend-part-009`. Включает описание доменных модулей (communications, documents, home, knowledge, notes, organizations, personas, projects), их API-клиентов, query-хуков на базе TanStack Vue Query, Pinia-сторов и типов. Содержимое основано исключительно на встроенных исходных файлах.

## Предложенные страницы

#### `components/frontend.md`

# Компоненты фронтенда

## Обзор архитектуры

Фронтенд использует Vue 3 (Composition API), TypeScript, Pinia для управления состоянием и TanStack Vue Query для выполнения и кеширования HTTP-запросов. Взаимодействие с бэкендом происходит через `ApiClient` из `platform/api/ApiClient`, который предоставляет методы `get`, `post`, `put`.

## Домены

### Communications (Коммуникации)

#### `useThreadReplyActions`

Файл: `frontend/src/domains/communications/views/useThreadReplyActions.ts`

Хук (composable) `useThreadReplyActions(store: CommunicationsStore)` предоставляет:

- `handleReplyToThreadMessage(message, bodyHtml, draftId)` — открывает форму ответа на сообщение через `store.openCompose`.
- `handleSaveThreadReplyDraft(message, bodyHtml, draftId)` — сохраняет черновик ответа. Если `bodyHtml` пуст, выходит. Использует `saveDraftMutation` и `buildComposeDraftPayload`. При успехе вызывает `store.setMailActionStatus('Draft saved')`, при ошибке — `store.setMailActionError`.
- `handleSendThreadReply(message, bodyHtml, draftId)` — отправляет ответ. Если `bodyHtml` пуст, выходит. Использует `sendMailMutation` и `composeFormToSendRequest`. При успехе устанавливает статус с указанием транспорта (`result.transport`), при ошибке — сообщение об ошибке.
- `isThreadReplySending` — `computed` на основе `sendMailMutation.isPending.value`.

Зависимости (импорты, не включённые в этот чанк):
- `useSaveDraftMutation`, `useSendMailMutation` из `../queries/useCommunicationsQuery`
- `buildComposeDraftPayload` из `../forms/composeDraftAutosave`
- `composeFormToSendRequest`, `threadReplyComposeForm` из `../helpers/communicationPageModels`
- Тип `ThreadMessage` из `../types/communications`
- Тип `CommunicationsStore` из `../stores/communications`

### Documents (Документы)

#### API

Файл: `frontend/src/domains/documents/api/documents.ts`

Функции:
- `fetchDocumentProcessing(documentId: string): Promise<DocumentProcessingRecord>` — GET `/api/v1/documents/{documentId}/processing`
- `fetchDocumentProcessingJobs(limit = 50): Promise<DocumentProcessingJobsResponse>` — GET `/api/v1/document-processing/jobs?limit={limit}`
- `retryDocumentProcessingJob(jobId, request): Promise<DocumentProcessingRetryResponse>` — POST `/api/v1/document-processing/jobs/{jobId}/retry`

#### Queries

Файл: `frontend/src/domains/documents/queries/useDocumentsQuery.ts`

- `useDocumentProcessingJobsQuery(limit = 50)` — обёртка TanStack Vue Query `useQuery` с ключом `['document-processing-jobs', limit]`. Вызывает `fetchDocumentProcessingJobs(limit)`.

#### Store

Файл: `frontend/src/domains/documents/stores/documents.ts`

Pinia-стор `useDocumentsStore` (id: `documents-ui`):

Состояние:
- `searchQuery: string`
- `activeFilter: string` (начальное значение `'all'`)
- `documentsError: string`
- `retryingJobId: string | null`

Действия:
- `setSearchQuery(q: string)`
- `setActiveFilter(filter: string)`
- `setDocumentsError(err: string)`
- `setRetryingJobId(id: string | null)`

#### Типы

Файл: `frontend/src/domains/documents/types/documents.ts`

Основные типы:
- `DocumentProcessingStatus`: `'queued' | 'running' | 'succeeded' | 'failed' | 'skipped'`
- `DocumentProcessingStep`: `'extract_text' | 'ocr'`
- `DocumentProcessingArtifactKind`: `'extracted_text' | 'ocr_text'`
- `DocumentProcessingJob`: `job_id`, `document_id`, `step`, `status`, `attempts`, `max_attempts`, `last_error_summary` (nullable), `queued_at`, `started_at` (nullable), `finished_at` (nullable), `created_at`, `updated_at`.
- `DocumentProcessingArtifact`: `artifact_id`, `document_id`, `job_id`, `artifact_kind`, `content_sha256`, `text_content` (nullable), `storage_kind` (nullable), `storage_path` (nullable), `metadata: Record<string, unknown>`, `created_at`.
- `DocumentProcessingRecord`: `document_id`, `jobs: DocumentProcessingJob[]`, `artifacts: DocumentProcessingArtifact[]`.
- `DocumentProcessingJobsResponse`: `items: DocumentProcessingJob[]`.
- `DocumentProcessingRetryRequest`: `command_id: string`.
- `DocumentProcessingRetryResponse`: `job_id`, `status: DocumentProcessingStatus`, `event_id`.
- `DocDisplayItem`: поля для UI — `name`, `source`, `project`, `type`, `date`, `size`, `icon`, `tone`.

### Home (Главная)

#### API

Файл: `frontend/src/domains/home/api/home.ts`

Функции:
- `fetchCommunicationMessages(limit = 50): Promise<CommunicationMessagesResponse>` — GET `/api/v1/communications/messages?limit={limit}`
- `fetchMailboxHealth(): Promise<MailboxHealth>` — GET `/api/v1/communications/analytics/health`

#### Queries

Файл: `frontend/src/domains/home/queries/useHomeQuery.ts`

- `useCommunicationMessagesQuery(limit = 50)` — `useQuery` с ключом `['home', 'communication-messages', limit]`. Возвращает `res.items` (тип `CommunicationMessageSummary[]`).
- `useMailboxHealthQuery()` — `useQuery` с ключом `['home', 'mailbox-health']`. Возвращает `MailboxHealth`.

#### Типы API

Файл: `frontend/src/domains/home/types/api.ts`

- `CommunicationMessageSummary`: `message_id`, `raw_record_id`, `account_id`, `provider_record_id`, `subject`, `sender`, `recipients`, `body_text_preview`, `occurred_at` (nullable), `projected_at`, `channel_kind`, `conversation_id` (nullable), `sender_display_name` (nullable), `delivery_state`, `message_metadata: Record<string, unknown>`, `attachment_count`, `local_state: LocalMessageState`, `local_state_changed_at` (nullable).
- `LocalMessageState`: `'active' | 'trash' | 'all'`
- `CommunicationMessagesResponse`: `items: CommunicationMessageSummary[]`
- `MailboxHealth`: `total_messages`, `unread`, `needs_action`, `waiting`, `done`, `archived`, `spam`, `important`, `with_attachments`, `average_importance`, `oldest_message_days` (nullable).

#### Типы UI

Файл: `frontend/src/domains/home/types/home.ts`

Интерфейсы для отображения (UI):
- `StatCard`: `label`, `value`, `delta`, `icon`, `tone?`
- `FeedItem`: `icon`, `title`, `meta`, `time`, `tag?`, `tone?`
- `TaskItem`: `title`, `assignee`, `due`, `priority`
- `PersonItem`: `name`, `meta`, `icon`
- `ProjectItem`: `name`, `kind`, `progress`, `icon`, `tone`
- `SystemStatusItem`: `label`, `value`, `status` (`'ok' | 'warning' | 'error'`)

### Knowledge (Граф знаний и противоречия)

#### API

Файл: `frontend/src/domains/knowledge/api/knowledge.ts`

Функции:
- `fetchGraphSummary(): Promise<GraphSummary>` — GET `/api/v1/graph/summary`
- `fetchGraphNodes(limit = 20): Promise<GraphNode[]>` — GET `/api/v1/graph/nodes?limit={limit}`
- `searchGraphNodes(query, limit = 20): Promise<GraphNode[]>` — GET `/api/v1/graph/search?q={query}&limit={limit}`. Если `query` пуст, возвращает `[]`.
- `fetchGraphNeighborhood(nodeId, depth = 1): Promise<GraphNeighborhood>` — GET `/api/v1/graph/neighborhood?node_id={nodeId}&depth={depth}`
- `fetchContradictions(limit = 50): Promise<ContradictionListResponse>` — GET `/api/v1/contradictions?limit={limit}`
- `reviewContradiction(observationId, request): Promise<ContradictionObservation>` — PUT `/api/v1/contradictions/{observationId}/review`

#### Queries

Файл: `frontend/src/domains/knowledge/queries/useKnowledgeQuery.ts`

- `useGraphSummaryQuery()` — `useQuery` с ключом `['graph-summary']`, возвращает результат `fetchGraphSummary()`.
- `useContradictionsQuery(limit = 50)` — `useQuery` с ключом `['contradictions', limit]`, возвращает `response.items`.

#### Store

Файл: `frontend/src/domains/knowledge/stores/knowledge.ts`

Pinia-стор `useKnowledgeStore` (id: `knowledge`). Содержит:

Состояние (ref):
- `graphSummary: GraphSummary | null`
- `graphError: string`
- `graphSearchQuery: string`
- `graphSearchResults: GraphNode[]`
- `graphNeighborhood: GraphNeighborhood | null`
- `selectedGraphNode: GraphNode | null`
- `contradictionObservations: ContradictionObservation[]`
- `reviewingContradictionObservationId: string | null`

Вычисляемые свойства (computed):
- `graphCanvasNodes` — строит радиальную раскладку узлов через `buildRadialLayout` (максимум 14 соседей, радиус `RADIUS = 38`).
- `graphCanvasEdges` — строит рёбра через `buildEdges` с преобразованием `relationship_type` (замена `_` на пробел) и пробросом `review_state`.
- `selectedGraphProperties` — свойства выбранного узла (до 8 записей `Record<string, unknown>`, отсортированные по ключу).
- `graphNeighborCounts` — количество соседей по `node_kind`, отсортированное по убыванию.
- `graphFilterChips` — чипсы фильтров на основе `graphSummary.node_counts`, с иконкой и лейблом из `graphNodeKindIcon`/`graphNodeKindLabel`.

Действия (actions):
- `setGraphSummary(summary, error)` — устанавливает `graphSummary` и `graphError`.
- `setGraphSearchResults(results, query)` — устанавливает `graphSearchResults` и `graphSearchQuery`.
- `selectGraphNode(node)` — устанавливает `selectedGraphNode`, сбрасывает `graphNeighborhood`, загружает neighbourhood через `fetchGraphNeighborhood(node.node_id, 1)`. При ошибке записывает в `graphError`.
- `runGraphSearch(query)` — при пустом запросе очищает результаты. Иначе выполняет `searchGraphNodes(query, 20)`, обновляет `graphSearchResults` и `graphSearchQuery`. Ошибки пишет в `graphError`.
- `loadGraphNodeChoices()` — вызывает `fetchGraphNodes(20)`, возвращает массив узлов или `[]` при ошибке (с записью в `graphError`).
- `setContradictionObservations(observations)` — устанавливает `contradictionObservations`.
- `reviewContradictionObservation(observation, reviewState, resolution?)` — устанавливает `reviewingContradictionObservationId`, вызывает `reviewContradiction`. При успехе обновляет `review_state` в локальном массиве `contradictionObservations`. При ошибке записывает в `graphError`. В `finally` сбрасывает `reviewingContradictionObservationId`.

Вспомогательные функции (экспортируются из модуля):
- `buildRadialLayout(center, neighbors, radius)` — центр в (50,50), соседи по кругу.
- `buildEdges(centerId, edges, canvasNodes)` — сопоставляет узлы по id.
- `graphNodeKindIcon(kind)` — возвращает строку иконки (`tabler:*`) для каждого `GraphNodeKind`.
- `graphNodeKindLabel(kind)` — человекочитаемый лейбл (замена `_` на пробел с капитализацией).
- `contradictionSeverityTone(severity)` — возвращает severity без изменений.
- `formatContradictionClaim(observation)` — строка вида `old_claim -> new_claim`.
- `formatContradictionTime(value)` — форматирует дату в `en` локали (MMM DD, HH:MM), при ошибке — `'Unknown date'`.
- `formatContradictionSource(kind, sourceId)` — строка вида `FormattedKind · sourceId`.

Интерфейсы компонентов графа (экспортируются):
- `GraphCanvasNode`: `node_id`, `node_kind`, `label`, `x`, `y`, `isSelected`, `layoutClass`
- `GraphCanvasEdge`: `x1`, `y1`, `x2`, `y2`, `label`, `review_state`
- `GraphFilterChip`: `kind`, `label`, `icon`, `count`

#### Типы

Файл: `frontend/src/domains/knowledge/types/knowledge.ts`

Основные типы:
- `GraphNodeKind`: `'person' | 'email_address' | 'message' | 'document' | 'project' | 'organization' | 'task' | 'event' | 'decision' | 'obligation' | 'knowledge'`
- `GraphRelationshipType`: `'person_has_email_address' | 'person_sent_message' | 'person_received_message' | 'email_address_sent_message' | 'email_address_received_message' | 'project_has_message' | 'project_has_document' | 'project_involves_person' | 'project_involves_email_address' | 'entity_relationship'`
- `GraphReviewState`: `'system_accepted' | 'suggested' | 'user_confirmed' | 'user_rejected'`
- `GraphEvidenceSourceKind`: `'person' | 'message' | 'document' | 'raw_record' | 'relationship' | 'decision' | 'obligation'`
- `GraphNode`: `node_id`, `node_kind`, `stable_key`, `label`, `properties: Record<string, unknown>`, `created_at`, `updated_at`
- `GraphEdge`: `edge_id`, `source_node_id`, `target_node_id`, `relationship_type`, `confidence`, `review_state`, `properties`, `valid_from` (nullable), `valid_to` (nullable), `created_at`, `updated_at`
- `GraphCount`: `key`, `count`
- `GraphSummary`: `node_counts: GraphCount[]`, `edge_counts: GraphCount[]`, `evidence_count`, `latest_projection_at` (nullable), `is_empty`
- `GraphEvidenceSummary`: `edge_id`, `source_kind`, `source_id`, `excerpt` (nullable), `metadata`
- `GraphNeighborhood`: `selected_node`, `nodes: GraphNode[]`, `edges: GraphEdge[]`, `evidence: GraphEvidenceSummary[]`, `edge_limit`, `truncated`, `evidence_limit`, `evidence_truncated`
- `ContradictionSourceKind`: `'communication' | 'document' | 'event' | 'memory' | 'knowledge' | 'decision' | 'obligation' | 'task' | 'relationship' | 'raw_record'`
- `ContradictionSeverity`: `'low' | 'medium' | 'high' | 'critical'`
- `ContradictionReviewState`: `'suggested' | 'user_confirmed' | 'user_rejected'`
- `ContradictionObservation`: `observation_id`, `old_source_kind`, `old_source_id`, `new_source_kind`, `new_source_id`, `affected_entities: unknown`, `conflict_type`, `old_claim`, `new_claim`, `confidence`, `severity`, `review_state`, `metadata`, `reviewed_by` (nullable), `reviewed_at` (nullable), `resolution` (nullable), `created_at`, `updated_at`
- `ContradictionListResponse`: `items: ContradictionObservation[]`
- `ContradictionReviewRequest`: `review_state` (исключая `'suggested'`), `resolution?: string`

### Notes (Заметки)

#### API

Файл: `frontend/src/domains/notes/api/notes.ts`

- `fetchNotes(): Promise<{ items: NoteItem[] }>` — GET `/api/v1/notes`

#### Queries

Файл: `frontend/src/domains/notes/queries/useNotesQuery.ts`

- `useNotesQuery()` — `useQuery` с ключом `['notes']`, вызывает `fetchNotes()`.

#### Store

Файл: `frontend/src/domains/notes/stores/notes.ts`

Pinia-стор `useNotesStore` (id: `notes-ui`):

Состояние:
- `searchQuery: string`
- `activeSources: string[]`
- `activeTags: string[]`
- `notesError: string`

Действия:
- `setSearchQuery(q: string)`
- `toggleSource(source: string)` — добавляет/удаляет из `activeSources`
- `toggleTag(tag: string)` — добавляет/удаляет из `activeTags`
- `setNotesError(err: string)`

#### Типы

Файл: `frontend/src/domains/notes/types/notes.ts`

- `NoteItem`: `title`, `body`, `source`, `tag`, `time`, `icon`

### Organizations (Организации)

#### API

Файл: `frontend/src/domains/organizations/api/organizations.ts`

Функции:
- `fetchOrganizations(limit = 50): Promise<OrganizationListResponse>` — GET `/api/v1/organizations?limit={limit}`
- `fetchOrganization(orgId): Promise<Organization>` — GET `/api/v1/organizations/{orgId}`

`OrganizationListResponse`: `{ items: Organization[] }`

#### Queries

Файл: `frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`

- `useOrganizationsQuery()` — `useQuery` с ключом `['organizations', 'list']`, возвращает `res.items`.
- `useOrganizationQuery(orgId)` — `useQuery` с ключом `['organizations', orgId]`, enabled только при `!!orgId`.

#### Типы

Файл: `frontend/src/domains/organizations/types/organization.ts`

- `Organization`: `organization_id`, `display_name` (обязательные); опциональные: `industry`, `country`, `status`, `watchlist`, `health_status`, `description`, `website`, `legal_name`, `registration_number`, `vat`, `interaction_count`, `priority` — все `string | null | boolean | number | undefined`.

### Personas (Персоны)

#### API

Файл: `frontend/src/domains/personas/api/personas.ts`

Функции:
- `fetchPersons(limit = 50): Promise<PersonListResponse>` — GET `/api/v1/persons`
- `fetchPersonDossier(personId): Promise<PersonDossier>` — GET `/api/v1/persons/{id}/dossier`
- `fetchIdentityCandidates(limit = 50): Promise<PersonIdentityCandidateListResponse>` — GET `/api/v1/identity-candidates`
- `reviewIdentityCandidate(candidateId, reviewState): Promise<void>` — PUT `/api/v1/identity-candidates/{id}/review` с телом `{ review_state }`
- `fetchIdentityTraces(limit = 50): Promise<PersonIdentityTraceListResponse>` — GET `/api/v1/identity-traces`
- `assignIdentityTrace(traceId, personId): Promise<void>` — POST `/api/v1/identity-traces/{id}/assign` с телом `{ person_id }`
- `fetchRelationships(limit = 50): Promise<RelationshipListResponse>` — GET `/api/v1/relationships`
- `reviewRelationship(relationshipId, reviewState): Promise<void>` — PUT `/api/v1/relationships/{id}/review` с телом `{ review_state }`
- `fetchOrganizations(limit = 50): Promise<OrganizationListResponse>` — GET `/api/v1/organizations`
- `fetchOrganization(orgId): Promise<any>` — GET `/api/v1/organizations/{id}`

Типы ответов (объявлены в этом файле):
- `PersonListResponse`: `{ items: EnrichedPerson[] }`
- `PersonIdentityCandidateListResponse`: `{ items: PersonIdentityCandidate[] }`
- `PersonIdentityTraceListResponse`: `{ items: PersonIdentity[] }`
- `OrganizationListResponse`: `{ items: any[] }` (использует `any[]`, а не типизированный `Organization`)
- `RelationshipListResponse`: `{ relationships: Relationship[] }`

#### Queries

Файл: `frontend/src/domains/personas/queries/usePersonasQuery.ts`

- `usePersonsQuery()` — `useQuery` с ключом `['persons', 'list']`, возвращает `res.items`.
- `useIdentityCandidatesQuery()` — `useQuery` с ключом `['persons', 'identity-candidates']`, возвращает `res.items`.
- `useIdentityTracesQuery()` — `useQuery` с ключом `['persons', 'identity-traces']`, возвращает `res.items`.
- `useRelationshipsQuery()` — `useQuery` с ключом `['persons', 'relationships']`, возвращает `res.relationships`.

#### Store

Файл: `frontend/src/domains/personas/stores/personas.ts`

Pinia-стор `usePersonasStore` (id: `personas`). Содержит:

Состояние (ref):
- `selectedPersonIndex: number`
- `loadedDossierPersonId: string | null`
- `personDossier: PersonDossier | null`
- `personDossierError: string`
- `isPersonDossierLoading: boolean`
- `identityCandidatesError: string`
- `identityTracesError: string`
- `relationshipsError: string`
- `assigningIdentityTraceId: string | null`
- `reviewingRelationshipId: string | null`
- `identityCandidates: PersonIdentityCandidate[]`
- `identityTraces: PersonIdentity[]`
- `relationships: Relationship[]`

Вычисляемое:
- `suggestedIdentityCandidates` — фильтр `identityCandidates` по `review_state === 'suggested'`.

Действия:
- `setIdentityCandidates(items)`, `setIdentityTraces(items)`, `setRelationships(items)`
- `selectPerson(index)`, `setPersonDossier(dossier, error)`, `setPersonDossierLoading(loading)`, `setLoadedDossierPersonId(id)`
- `reviewCandidate(candidate, state)` — вызывает `reviewIdentityCandidate`. При ошибке записывает в `identityCandidatesError`.
- `assignTraceToPersona(trace, personId)` — устанавливает `assigningIdentityTraceId`, вызывает `assignIdentityTrace`. При ошибке записывает в `identityTracesError`. Сбрасывает `assigningIdentityTraceId` после выполнения.
- `reviewRelation(relationship, reviewState)` — устанавливает `reviewingRelationshipId`, вызывает `reviewRelationship`. При ошибке записывает в `relationshipsError`. Сбрасывает `reviewingRelationshipId` после выполнения.

Вспомогательные функции (экспортируются):
- `formatIdentityTraceKind(kind)` — маппинг kind в читаемый лейбл (Email, Phone, Telegram и т.д.)
- `formatIdentityTraceValue(trace)` — `trace.value || trace.identity_type`
- `identityTraceConfidence(trace)` — `${Math.round(trace.confidence * 100)}%`
- `formatRelationshipType(type)` — маппинг типа связи в лейбл (Colleague, Manager, Client и т.д.)
- `formatRelationshipScore(score)` — Strong (≥0.8), Moderate (≥0.5), Weak
- `formatRelationshipEndpoint(kind, id)` — `kind:первые_8_символов_id...`
- `personIdentityPairKey(leftPersonId, rightPersonId)` — конкатенация в лексикографическом порядке
- `dossierSectionPreview(dossier)` — первые 10 уникальных слов из `dossier.summary`

#### Типы

Файл: `frontend/src/domains/personas/types/persona.ts`

- `PersonaType`: `'human' | 'ai_agent' | 'organization_proxy' | 'system'`
- `EnrichedPerson`: `person_id`, `display_name`, `email_address`, `preferred_channel` (nullable), `last_interaction_at` (nullable), `linked_projects` (nullable)
- `PersonDossier`: `summary`, `source_refs`, `generated_at`
- `PersonIdentityCandidate`: `candidate_id`, `candidate_kind`, `left_person_id`, `right_person_id` (nullable), `evidence_summary`, `confidence`, `review_state`, `created_at`
- `PersonIdentityReviewState`: `'suggested' | 'user_confirmed' | 'user_rejected'`
- `PersonIdentity`: `id`, `identity_type`, `value`, `source`, `confidence`, `person_id` (nullable)
- `Relationship`: `relationship_id`, `source_entity_id`, `source_entity_kind`, `target_entity_id`, `target_entity_kind`, `relationship_type`, `trust_score`, `strength_score`, `confidence`, `review_state`
- `RelationshipReviewState`: `'suggested' | 'system_accepted' | 'user_confirmed' | 'user_rejected'`
- `PersonItem`: `person_id`, `name`, `role`, `company`, `channel?`, `status?`
- `PersonaOption`: `person_id`, `name`, `company`

### Projects (Проекты)

#### API

Файл: `frontend/src/domains/projects/api/projects.ts`

Функции:
- `fetchProjects(limit = 25): Promise<ProjectListResponse>` — GET `/api/v1/projects?limit={limit}`
- `fetchProjectDetail(projectId): Promise<ProjectDetail>` — GET `/api/v1/projects/{projectId}`

Типы `ProjectListResponse` и `ProjectDetail` импортируются из `../types/project` (не включены в этот чанк).

### Общие зависимости

- `ApiClient` из `../../platform/api/ApiClient` используется во всех доменных API-файлах для выполнения HTTP-запросов.
- TanStack Vue Query (`@tanstack/vue-query`) — `useQuery`, `useMutation` (последнее видно в communications).
- Pinia (`pinia`) — `defineStore`.
- Vue 3 Composition API (`vue`) — `ref`, `computed`.

## Покрытие источников

| Файл | Факты, покрытые на странице |
|---|---|
| `frontend/src/domains/communications/views/useThreadReplyActions.ts` | Функции `handleReplyToThreadMessage`, `handleSaveThreadReplyDraft`, `handleSendThreadReply`, `isThreadReplySending`; зависимости от мутаций и хелперов |
| `frontend/src/domains/documents/api/documents.ts` | API-функции `fetchDocumentProcessing`, `fetchDocumentProcessingJobs`, `retryDocumentProcessingJob` и их эндпоинты |
| `frontend/src/domains/documents/queries/useDocumentsQuery.ts` | Query-хук `useDocumentProcessingJobsQuery`, ключ запроса |
| `frontend/src/domains/documents/stores/documents.ts` | Pinia-стор `useDocumentsStore`, состояние и действия |
| `frontend/src/domains/documents/types/documents.ts` | Все типы документов: статусы, шаги, артефакты, задания, запросы/ответы, `DocDisplayItem` |
| `frontend/src/domains/home/api/home.ts` | API-функции `fetchCommunicationMessages`, `fetchMailboxHealth` |
| `frontend/src/domains/home/queries/useHomeQuery.ts` | Query-хуки `useCommunicationMessagesQuery`, `useMailboxHealthQuery` |
| `frontend/src/domains/home/types/api.ts` | Типы `CommunicationMessageSummary`, `LocalMessageState`, `CommunicationMessagesResponse`, `MailboxHealth` |
| `frontend/src/domains/home/types/home.ts` | UI-интерфейсы `StatCard`, `FeedItem`, `TaskItem`, `PersonItem`, `ProjectItem`, `SystemStatusItem` |
| `frontend/src/domains/knowledge/api/knowledge.ts` | Все API-функции графа и противоречий, эндпоинты |
| `frontend/src/domains/knowledge/queries/useKnowledgeQuery.ts` | Query-хуки `useGraphSummaryQuery`, `useContradictionsQuery` |
| `frontend/src/domains/knowledge/stores/knowledge.ts` | Полный стор `useKnowledgeStore`: состояние, computed, actions, вспомогательные функции раскладки и форматирования |
| `frontend/src/domains/knowledge/types/knowledge.ts` | Все типы графа и противоречий: узлы, рёбра, summary, neighborhood, contradiction observation, severity, review |
| `frontend/src/domains/notes/api/notes.ts` | API-функция `fetchNotes` |
| `frontend/src/domains/notes/queries/useNotesQuery.ts` | Query-хук `useNotesQuery` |
| `frontend/src/domains/notes/stores/notes.ts` | Pinia-стор `useNotesStore`, состояние и действия (включая toggleSource/toggleTag) |
| `frontend/src/domains/notes/types/notes.ts` | Тип `NoteItem` |
| `frontend/src/domains/organizations/api/organizations.ts` | API-функции `fetchOrganizations`, `fetchOrganization`, тип `OrganizationListResponse` |
| `frontend/src/domains/organizations/queries/useOrganizationsQuery.ts` | Query-хуки `useOrganizationsQuery`, `useOrganizationQuery` (с enabled) |
| `frontend/src/domains/organizations/types/organization.ts` | Тип `Organization` со всеми опциональными полями |
| `frontend/src/domains/personas/api/personas.ts` | Все API-функции персон, identity, связей, а также дублирующие функции организаций с `any` |
| `frontend/src/domains/personas/queries/usePersonasQuery.ts` | Query-хуки `usePersonsQuery`, `useIdentityCandidatesQuery`, `useIdentityTracesQuery`, `useRelationshipsQuery` |
| `frontend/src/domains/personas/stores/personas.ts` | Полный стор `usePersonasStore`: состояние, computed `suggestedIdentityCandidates`, actions ревью и назначения, вспомогательные функции форматирования |
| `frontend/src/domains/personas/types/persona.ts` | Все типы: `PersonaType`, `EnrichedPerson`, `PersonDossier`, `PersonIdentityCandidate`, `PersonIdentity`, `Relationship`, `PersonItem`, `PersonaOption` |
| `frontend/src/domains/projects/api/projects.ts` | API-функции `fetchProjects`, `fetchProjectDetail` |

## Исходные файлы

- [`frontend/src/domains/communications/views/useThreadReplyActions.ts`](../../../../frontend/src/domains/communications/views/useThreadReplyActions.ts)
- [`frontend/src/domains/documents/api/documents.ts`](../../../../frontend/src/domains/documents/api/documents.ts)
- [`frontend/src/domains/documents/queries/useDocumentsQuery.ts`](../../../../frontend/src/domains/documents/queries/useDocumentsQuery.ts)
- [`frontend/src/domains/documents/stores/documents.ts`](../../../../frontend/src/domains/documents/stores/documents.ts)
- [`frontend/src/domains/documents/types/documents.ts`](../../../../frontend/src/domains/documents/types/documents.ts)
- [`frontend/src/domains/home/api/home.ts`](../../../../frontend/src/domains/home/api/home.ts)
- [`frontend/src/domains/home/queries/useHomeQuery.ts`](../../../../frontend/src/domains/home/queries/useHomeQuery.ts)
- [`frontend/src/domains/home/types/api.ts`](../../../../frontend/src/domains/home/types/api.ts)
- [`frontend/src/domains/home/types/home.ts`](../../../../frontend/src/domains/home/types/home.ts)
- [`frontend/src/domains/knowledge/api/knowledge.ts`](../../../../frontend/src/domains/knowledge/api/knowledge.ts)
- [`frontend/src/domains/knowledge/queries/useKnowledgeQuery.ts`](../../../../frontend/src/domains/knowledge/queries/useKnowledgeQuery.ts)
- [`frontend/src/domains/knowledge/stores/knowledge.ts`](../../../../frontend/src/domains/knowledge/stores/knowledge.ts)
- [`frontend/src/domains/knowledge/types/knowledge.ts`](../../../../frontend/src/domains/knowledge/types/knowledge.ts)
- [`frontend/src/domains/notes/api/notes.ts`](../../../../frontend/src/domains/notes/api/notes.ts)
- [`frontend/src/domains/notes/queries/useNotesQuery.ts`](../../../../frontend/src/domains/notes/queries/useNotesQuery.ts)
- [`frontend/src/domains/notes/stores/notes.ts`](../../../../frontend/src/domains/notes/stores/notes.ts)
- [`frontend/src/domains/notes/types/notes.ts`](../../../../frontend/src/domains/notes/types/notes.ts)
- [`frontend/src/domains/organizations/api/organizations.ts`](../../../../frontend/src/domains/organizations/api/organizations.ts)
- [`frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`](../../../../frontend/src/domains/organizations/queries/useOrganizationsQuery.ts)
- [`frontend/src/domains/organizations/types/organization.ts`](../../../../frontend/src/domains/organizations/types/organization.ts)
- [`frontend/src/domains/personas/api/personas.ts`](../../../../frontend/src/domains/personas/api/personas.ts)
- [`frontend/src/domains/personas/queries/usePersonasQuery.ts`](../../../../frontend/src/domains/personas/queries/usePersonasQuery.ts)
- [`frontend/src/domains/personas/stores/personas.ts`](../../../../frontend/src/domains/personas/stores/personas.ts)
- [`frontend/src/domains/personas/types/persona.ts`](../../../../frontend/src/domains/personas/types/persona.ts)
- [`frontend/src/domains/projects/api/projects.ts`](../../../../frontend/src/domains/projects/api/projects.ts)

## Кандидаты на drift

Из предоставленного контекста видны следующие потенциальные расхождения:

1. **Типизация организаций в `personas/api/personas.ts`** — в файле `personas/api/personas.ts` тип `OrganizationListResponse` объявлен как `{ items: any[] }`, а `fetchOrganization` возвращает `Promise<any>`. В то же время в `organizations/api/organizations.ts` используется строго типизированный `Organization` (из `organizations/types/organization.ts`). Это может указывать на дублирование API-клиента для организаций без полной типизации в домене персон.

2. **Дублирование API организаций** — функции `fetchOrganizations` и `fetchOrganization` реализованы как в `organizations/api/organizations.ts`, так и в `personas/api/personas.ts` (с одинаковыми эндпоинтами). Непонятно, является ли это осознанным переиспользованием или дублированием; в любом случае это может привести к рассинхронизации при изменениях API.

3. **Типы `DocDisplayItem` vs API-типы** — файл `documents/types/documents.ts` содержит интерфейс `DocDisplayItem` с полями `name`, `source`, `tone`, `icon`, которые похожи на UI-данные, а не на ответы API. Остальные типы в этом файле — строго API-модели. Возможно, `DocDisplayItem` логичнее было бы разместить в UI-типах или отдельном файле, но из контекста это не подтверждено.

4. **Несовпадение `local_state` в `CommunicationMessageSummary`** — тип `LocalMessageState` определён как `'active' | 'trash' | 'all'`, однако в API home запросах значение `local_state` может приходить в ином формате; проверить это по данному чанку невозможно.
