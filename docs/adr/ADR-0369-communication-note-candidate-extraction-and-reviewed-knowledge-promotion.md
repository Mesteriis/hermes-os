# ADR-0369: Communication note candidate extraction and reviewed Knowledge promotion

Статус: Принято

Дата: 2026-08-01

Состояние реализации: staged. Приняты exact source, extraction и candidate
contracts, pure deterministic extraction core/lifecycle и отдельный owner-local
PostgreSQL persistence unit с run state, request replay, inbox, outbox и
replayable realtime. Managed runtimes, Review-owned decision, promotion
workflow, Knowledge command consumer и aggregate managed conformance ещё не реализованы; поэтому
`communication_note_candidate_extraction_v1` остаётся `planned`.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0207](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0226](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0253](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0315](ADR-0315-communications-message-body-content-read.md).

## Контекст

Legacy `ExtractMessageNotes` находился внутри Communications implementation,
читал её message store и возвращал эвристический `title/content/tags` result.
Этот результат не создавал canonical Knowledge note, не имел отдельного Review
lifecycle и смешивал source ownership с extraction use case.

Clean-room перенос не может вернуть тот же handler как Communications facade,
дать Knowledge доступ к Communications storage или превратить найденный текст
в durable truth без решения владельца. Communications, extraction workflow,
Review и Knowledge имеют разные причины изменения и должны оставаться
раздельными independently buildable owners.

## Решение

### Owners и units

Source contract принадлежит Communications:

- `hermes-communications-note-source-api`.

Extraction принадлежит workflow owner
`communication_note_candidate_extraction`:

- `hermes-communication-note-candidate-api`;
- `hermes-communication-note-candidate-core`;
- `hermes-communication-note-candidate-persistence`;
- будущие отдельные runtime и assembly units.

Review получает отдельный note-candidate contract/core/persistence/runtime/
assembly slice. Он не расширяет task-candidate payload generic union и не
хранит Knowledge truth.

Knowledge получает собственные command API, core, persistence, runtime и
assembly units. Только Knowledge создаёт durable verified note. Promotion
между terminal Review decision и Knowledge command выполняет отдельный
`reviewed_note_candidate_promotion` workflow.

Ни один domain package не импортирует implementation, persistence или runtime
другого domain. Cross-owner flow использует только typed command/event/result
contracts, а workflow импортирует только exact public contracts. Kernel,
Gateway и Event Hub остаются owner-neutral и не интерпретируют note payload.

### Candidate semantics

`CommunicationNoteCandidateV1` является reviewable proposal, а не Knowledge
note, Decision, Document или accepted business truth. Candidate содержит:

- deterministic candidate ID и immutable digest;
- bounded owner-visible title и excerpt;
- bounded typed topic hints `financial`, `legal`, `decision_statement` и
  `deadline_statement`;
- exact source basis, confidence и Communications evidence reference.

Provider/account/model identity, arbitrary maps, prompt, raw source bytes и
target-domain identifiers запрещены. Candidate presentation передаётся между
owners только через target-bound Blob custody; durable envelopes несут IDs,
digests, evidence и receipt, но не private plaintext.

### Deterministic extraction V1

V1 сохраняет проверяемое legacy-поведение без legacy coupling. Pure core
проверяет bounded UTF-8 subject/body и создаёт не более одного candidate, если
в source найден хотя бы один marker из закрытых versioned наборов:

- finance: `invoice`, `payment`, `amount` и принятые локализованные варианты;
- legal: `contract`, `agreement`, `nda` и локализованные варианты;
- decision statement: `decided`, `approved`, `confirmed` и локализованные
  варианты;
- deadline statement: `deadline`, `due date`, `by` и локализованные варианты.

Title берётся из непустого subject, иначе используется фиксированный neutral
fallback. Excerpt строится из первых пяти непустых строк body, ограничивается
по символам и никогда не становится canonical Knowledge content до Review.
Topic hints дедуплицируются в фиксированном порядке. Пустой source или source
без markers возвращает пустой список и не фабрикует candidate.

V1 не использует AI Engine, `AiContextReceiptV1` или Ollama. Если качество
потребует inference, новый revision вводит distinct typed AI use-case contract;
Ollama остаётся concrete integration домена AI и не становится частью
Communications, Knowledge или workflow core.

### Event-only flow

```text
Authenticated client Start
  -> communication_note_candidate_extraction client_rpc
  -> durable PrepareCommunicationNoteSource command
  -> Communications source producer
  -> target-bound Blob + prepared/rejected result event
  -> extraction workflow owner-local state
  -> durable SubmitNoteCandidateForReview command + Review-bound Blob
  -> Review note-candidate owner
  -> authenticated owner-device approve/reject
  -> durable NoteCandidateApprovedForPromotion event
  -> reviewed_note_candidate_promotion workflow
  -> Knowledge CreateNoteFromReviewedCandidate command
  -> Knowledge owner-local note + terminal durable result
  -> Review-owned promotion result
```

Reject никогда не создаёт Knowledge note. Approve означает только принятое
Review decision; ровно один source-backed Knowledge note появляется после
terminal successful Knowledge result. Duplicate envelopes replay-ятся по exact
bytes/hash, conflicting operation reuse и stale revisions отклоняются.

Client query/request идёт через generated contracts и Core Gateway. Status
доставляется через общий replayable SSE с cursor recovery. Periodic polling и
handwritten business REST не вводятся.

## Phase gate

`communication_note_candidate_extraction_v1` может стать `implemented` только
после одновременного evidence:

1. source, extraction, Review, promotion и Knowledge units существуют отдельно;
2. owner-local PostgreSQL inbox/outbox/state/realtime и exact replay доказаны;
3. managed release запускает distinct runtime/module/owner identities;
4. реальный Communications source приводит к Review candidate без seeded rows;
5. до approve Knowledge note отсутствует, reject создаёт ноль notes, approve —
   ровно одну note;
6. wrong-owner, stale revision, generation/grant и Blob custody fences доказаны;
7. restart восстанавливает state и SSE cursors без plaintext в event/log/error;
8. architecture, Cargo, managed-runtime и full pre-push gates зелёные.

Наличие ADR, client skeleton или pure core само по себе gate не закрывает.

## Последствия

- Communications остаётся canonical evidence/source owner и не получает
  Knowledge behavior.
- Extraction остаётся workflow, Review владеет human decision, Knowledge —
  durable verified note truth.
- Legacy observable heuristic можно восстановить без cross-domain storage и
  без generic facade.
- Полный slice требует новых production units и отдельного admission phase gate.
