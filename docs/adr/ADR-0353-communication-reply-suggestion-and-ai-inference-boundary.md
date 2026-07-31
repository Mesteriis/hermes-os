# ADR-0353: Communication reply suggestion and AI inference boundary

Статус: Принято

Дата: 2026-07-30

Состояние реализации: architecture agreement, Communications-owned source
contract/runtime handoff и public AI contract unit `hermes-ai-contracts`
реализованы. Communications имеет durable command/results, inbox/hash fencing,
current-revision validation, target-bound Blob custody и commit-before-Ack.
AI contract unit имеет concrete reply request/result, common context receipt,
deterministic request digest и provider-neutral bounded local-only generation
port. Provider result обязан вернуть typed completeness и bounded confidence;
engine не фабрикует эти значения. Всё перечисленное имеет Cargo/architecture
evidence. Live event-only
source preparation ещё не доказано, поэтому gate
`communications_ai_context_source_v1` остаётся `planned`.
`hermes-ai-inference-core` также реализован как отдельная engine unit с
revision-fenced lifecycle, fixed prompt/policy receipt и sanitized terminal
results. `hermes-ai-inference-persistence` реализован отдельной owner-local
PostgreSQL unit: typed lifecycle, request/source receipts, provider-reported
effective settings revision, recoverable runs и terminal candidate сохраняются без
source message body или cross-owner SQL. `hermes-ai-inference-runtime`
реализует exact managed `request_rpc`, target-bound Blob custody/read,
provider-neutral outbound route и restart recovery; отдельный
`hermes-ai-inference-assembly` материализует только unsigned descriptor,
settings schema и Storage bundle inputs. Все пять AI engine build units
реализованы, но live managed inference ещё не доказан. Workflow ещё не
завершён. Для Ollama реализованы все шесть отдельных staged units —
`hermes-ollama-ai-api`,
`hermes-ollama-ai-core`, `hermes-ollama-ai-http`,
`hermes-ollama-ai-persistence`, `hermes-ollama-ai-runtime` и
`hermes-ollama-ai-assembly`: exact non-secret settings, fixed
loopback/model policy, request digest, structured result, owner-local
revision-fenced PostgreSQL lifecycle и terminal `uncertain` transition без
automatic retry. Persistence не хранит source content, prompt или HTTP request
body. HTTP unit реализует bounded `/api/tags` discovery и `/api/chat` dialect,
exact model binding, fixed JSON/non-streaming/non-thinking request и
fail-closed response framing без redirects. Managed integration runtime
реализует exact provider request RPC, commit-before-HTTP lifecycle, model
digest confirmation после generation и crash recovery только в `uncertain`,
без повторной отправки. Assembly материализует только unsigned descriptor,
settings schema, owner-local Storage bundle и release fragment. Dev release
compiler включает exact runtime и Storage artifacts в подписанный distribution
manifest. Real managed conformance запускает этот signed eventless Integration
через Kernel с настоящими Vault/Storage и disposable PostgreSQL/PgBouncer,
фиксирует terminal `ProviderUnavailable`, после restart возвращает exact
persisted response без второй HTTP-попытки и отклоняет request-ID conflict.
Это доказывает admission, storage binding и отрицательный replay contour, но не
является live успешным inference: установленный Ollama runtime и успешный
provider request на текущей машине отсутствуют. Поэтому gates
`ai_inference_v1`, `ollama_ai_provider_v1` и
`communication_reply_suggestion_v1` также остаются `planned`. Для reply
workflow реализованы первые три из пяти отдельных units:
`hermes-communication-reply-suggestion-api` с concrete generated
Start/Get/realtime contract и
`hermes-communication-reply-suggestion-core` с revision/digest-fenced
state machine, а также `hermes-communication-reply-suggestion-persistence` с
owner-local idempotent run state, source-result inbox/hash fence, exact
source-prepare outbox, recoverable state и client-safe realtime replay.
Persistence не хранит source body, prompt или provider metadata. Managed
runtime, assembly и live orchestration evidence ещё отсутствуют, поэтому этот
staged slice не открывает workflow gate.

Уточняет:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0222](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0226](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0231](ADR-0231-kernel-blob-service-and-owner-scoped-custody.md);
- [ADR-0253](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0315](ADR-0315-communications-message-body-content-read.md);
- [ADR-0339](ADR-0339-capability-routed-module-request-rpc.md);
- [ADR-0354](ADR-0354-integration-implemented-request-rpc-extension-ports.md);
- [ADR-0355](ADR-0355-capability-scoped-integration-event-hub-launch-configuration.md).

## Контекст

Clean-room inventory требует `communication_reply_suggestion_v1`, но active
backend пока не имеет AI public contract, inference owner, provider adapter или
разрешённого module-to-module body handoff.

Существующий `communications_content_read_v1` нельзя переиспользовать как
workflow port. Он выдаёт one-use capability только authenticated client session,
имеет `client_blob` transport и прямо исключает AI context. Передача этого
ticket другому runtime нарушила бы authority binding и превратила бы Gateway в
private-content facade.

Historical AI Reply подтверждает только product semantics:

- один canonical message является required source;
- caller явно выбирает tone и language;
- результат содержит reply subject и body;
- candidate сначала показывается для review и только затем может быть передан
  compose workflow;
- variants являются отдельным bounded fan-out, а не скрытым default behavior.

Legacy REST routes, shared in-process `AiRuntimePort`, prompt strings,
Communications-owned model selection и fallback `Ok(None)` не являются
clean-room контрактами.

## Решение

### Четыре owners/gates, а не один facade

Вертикальный срез состоит из четырёх независимо принимаемых gates:

```text
Communications domain
  communications_ai_context_source_v1
        |
        | target-bound source receipt
        v
communication_reply_suggestion workflow
        |
        | exact CommunicationReplySuggestionInferenceRequestV1
        v
AI inference engine
  ai_inference_v1
        |
        | typed local provider generation request
        v
Ollama integration
  ollama_ai_provider_v1
```

Kernel и Gateway согласуют descriptor, grants, routes, runtime generations,
storage/settings bindings и hard bounds. Они не импортируют ни один из этих
owner packages, не читают message body, не строят prompt и не выбирают модель.

Domain, workflow, engine и integration являются разными owners и разными
единицами сборки. Совпадение процесса разработки или малый размер кода не
разрешает объединить их.

### Communications-owned source handoff

`communications_ai_context_source_v1` добавляет public contract unit
`hermes-communications-ai-source-api`. Communications runtime реализует exact
durable prepare command/result:

```text
communications / ai_reply_source_prepare / v1
communications / ai_reply_source_prepared / v1
```

Prepare command содержит только:

- 16-byte workflow run ID;
- 16-byte canonical message ID;
- expected canonical message revision;
- target owner/runtime/capability binding для
  `communication_reply_suggestion`;
- correlation/causation metadata.

Он не содержит provider, account, body, subject, participant address, Blob
locator или arbitrary purpose.

Communications:

1. проверяет inbox ID/hash до mutation;
2. авторизует logical human owner и exact current canonical revision;
3. требует non-deleted message и admitted UTF-8 body;
4. создаёт bounded target-bound Blob source copy;
5. сохраняет preparation result и exact outbox bytes атомарно;
6. публикует только typed metadata, declared size/digest и opaque
   evidence-bound custody proof.

Client content ticket из ADR-0315 не используется. Provider fallback,
cross-owner SQL, direct socket и body bytes в NATS запрещены.

### AI public contracts

Public unit `hermes-ai-contracts` принадлежит engine owner `ai` и содержит:

- общий `AiContextReceiptV1`;
- exact `CommunicationReplySuggestionInferenceRequestV1`;
- exact `CommunicationReplySuggestionInferenceResultV1`;
- provider-neutral generation port для approved AI provider integrations.

Это не generic context API. Reply request имеет concrete fields:

- receipt;
- target-bound source reference;
- normalized tone enum;
- normalized language enum;
- reply subject policy;
- target-bound typed private source content with bounded provider-neutral
  sender, subject and body needed by reply semantics;
- maximum output bytes/tokens;
- local-only egress policy revision.

Запрещены stringly typed model/provider identity, arbitrary prompt, maps,
Protobuf `Any`, opaque business payload и repeated heterogeneous fragments.
Schema digest, contract revision и deterministic request digest входят в
receipt.

Inference result содержит только:

- workflow run ID;
- exact request/context digest;
- bounded UTF-8 subject/body candidate;
- resolved language and tone;
- model/prompt/policy receipt without credentials or private model response;
- completeness, confidence and sanitized terminal status.

Candidate не является Communication, draft или provider command.

### AI inference engine

`ai_inference_v1` принадлежит engine owner `ai`:

```text
hermes-ai-contracts
hermes-ai-inference-core
hermes-ai-inference-persistence
hermes-ai-inference-runtime
hermes-ai-inference-assembly
```

Core валидирует concrete request, budgets, policy and result. Persistence
хранит run lifecycle, request/source digests, selected settings revisions,
sanitized failures and typed result, но не долговечную копию message body или
generic context cache. Runtime принимает only exact AI requests, читает
target-bound source bytes и вызывает единственный descriptor-approved Ollama
provider contract через capability router. Multi-provider routing и AI-owned
runtime settings не входят в V1 и требуют отдельного gate.
Core применяет deterministic UTF-8-safe context framing и bounded 2000-byte
body excerpt, сохраняя reference-поведение без unsafe Unicode slicing; поэтому
private provider request остаётся внутри конституционного 64 KiB/30-second
`request_rpc` bound и не расширяет platform transport скрытым исключением.

AI engine:

- не импортирует Communications или workflow implementation;
- не вызывает Communications query API;
- не получает cross-owner SQL;
- не принимает caller-selected provider/model;
- не имеет hidden generic module-settings apply;
- не выдаёт credentials provider runtime;
- не записывает business truth;
- не выполняет automatic remote fallback.

Первая production revision имеет только `local_only` egress. Remote provider
egress требует отдельного ADR с explicit consent, redaction and credential
lease evidence.

### Ollama integration

`ollama_ai_provider_v1` принадлежит integration owner `ollama`:

```text
hermes-ollama-ai-api
hermes-ollama-ai-core
hermes-ollama-ai-http
hermes-ollama-ai-persistence
hermes-ollama-ai-runtime
hermes-ollama-ai-assembly
```

Integration владеет Ollama HTTP dialect, endpoint validation, model discovery,
timeouts, response framing and provider errors. Она реализует только approved
AI provider contract и не импортирует Communications, reply workflow or AI
engine implementation.

V1 допускает только loopback Ollama endpoint. Redirects, non-loopback target,
caller URL, automatic model download и implicit model substitution запрещены.
Endpoint и model selection приходят только из Ollama-owned effective settings,
согласованных Kernel Settings Registry через существующий integration settings
apply. Availability не означает permission to download.

Persistence хранит только request ID/digest, settings revision, selected model
digest и bounded terminal provider result. Она не хранит private prompt/input,
HTTP body, credentials или model response envelope. Exact replay того же
request ID/digest возвращает сохранённый terminal result; другой digest для
того же ID отклоняется. После неоднозначного HTTP outcome run остаётся typed
`uncertain` и не отправляется в Ollama повторно автоматически, потому что
Ollama `/api/chat` не предоставляет доказанного idempotency key.

Private content передаётся integration runtime только через bounded typed local
`request_rpc`; оно не входит в NATS, logs, traces, health или settings.

### Reply-suggestion workflow

Owner `communication_reply_suggestion` имеет отдельные units:

```text
hermes-communication-reply-suggestion-api
hermes-communication-reply-suggestion-core
hermes-communication-reply-suggestion-persistence
hermes-communication-reply-suggestion-runtime
hermes-communication-reply-suggestion-assembly
```

Client contract предоставляет:

```text
StartReplySuggestion(message_id, expected_revision, tone, language)
  -> accepted run_id

GetReplySuggestion(run_id)
  -> pending | ready(candidate) | rejected
```

V1 поддерживает exact tone enum `professional | friendly | concise | formal`
и language enum `source | english | russian | spanish`. Free-form prompt,
arbitrary language/model/provider, variants matrix и automatic Compose mutation
не входят в gate.

Workflow:

1. атомарно сохраняет idempotent run;
2. публикует Communications source prepare command;
3. принимает source result через inbox/hash fence;
4. собирает concrete AI request и `AiContextReceiptV1`;
5. вызывает exact AI inference request through `request_rpc`;
6. проверяет returned request/context digest;
7. сохраняет terminal candidate и client-safe realtime invalidation.

Frontend review использует только generated workflow client. Apply-to-compose
позже передаёт approved candidate отдельному compose/delivery workflow; он не
является частью inference и не мутирует Communications.

### Durability, privacy and fencing

- Start acceptance не означает source preparation или inference completion.
- Operation/run ID обеспечивает idempotency; payload hash mismatch rejected.
- Every owner хранит свой inbox/outbox/state only in its Storage namespace.
- Runtime generation, grant epoch, settings revision and Blob custody proof
  проверяются на каждом authority boundary.
- Timeout after ambiguous provider call не повторяется автоматически без same
  run/provider idempotency evidence; state становится typed `uncertain`.
- Restart resumes durable accepted work without creating a second candidate.
- SSE содержит только run ID, state, revision and occurred time.
- Message body, candidate body, prompt, provider response, Blob proof, model
  endpoint and errors do not enter SSE/logs/health.
- Wrong human owner, stale message revision, invalid UTF-8, expired custody,
  revoke, oversize input/output and digest mismatch fail closed.

## Phase gates

### `communications_ai_context_source_v1`

Открывается только при наличии public source contract, Communications-owned
inbox/outbox implementation, target-bound Blob custody, stale/edit/delete
negative matrix and live event-only preparation evidence.

### `ai_inference_v1`

Открывается только вместе с five AI engine units, common receipt and exact
reply request/result, owner-local run state, settings/fencing, Blob materialize,
provider `request_rpc`, restart/idempotency/privacy negatives and live managed
inference evidence.

### `ollama_ai_provider_v1`

Открывается только с separate API/core/http/persistence/runtime/assembly units,
loopback endpoint guard, exact settings, request digest/idempotency/uncertain
fencing, model/timeout/error conformance, private-content non-disclosure and
live Ollama request evidence. Mock or canned response не является production
evidence.

### `communication_reply_suggestion_v1`

Открывается атомарно только после трёх dependency gates и при наличии five
workflow units, generated Start/Get/realtime contracts, durable source and AI
orchestration, exact typed candidate, restart/replay/revoke/owner/stale/privacy
negative matrix, Gateway/SSE/browser review flow and full architecture/Cargo/
Clippy/test gates.

Наличие ADR, prompt unit test, skeleton или frontend card не открывает ни один
gate.

## Отклонённые варианты

### AI Reply method в Communications

Смешивает canonical evidence domain, workflow orchestration, model policy and
provider execution.

### Gateway fetches body and calls AI

Превращает transport boundary в private-content and AI facade.

### Workflow reuses client content ticket

Нарушает session/recipient binding ADR-0315 и скрывает новый cross-owner grant.

### AI engine queries Communications

Делает inference owner cross-domain orchestrator, запрещённый ADR-0226.

### Ollama adapter inside AI engine

Смешивает engine and integration build units and provider lifecycle.

### Return empty candidate when runtime is unavailable

Маскирует missing authority/runtime как successful AI result. V1 возвращает
typed unavailable/rejected/uncertain state.
