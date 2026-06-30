# AGENTS.md

Локальные инструкции для AI-агентов, работающих в репозитории Hermes Hub.

Этот файл является проектным overlay поверх глобального Codex / Engineering
Bible. Он не заменяет глобальные правила, а сужает их под Hermes. Нельзя
ослаблять требования глобальной конфигурации к правде, безопасности, проверке,
секретам, evidence и обязательному чтению файлов перед изменениями. Если этот
файл и глобальная Bible спорят, выполняй более строгое правило. Да, даже если
обходной путь выглядит удобным. Особенно тогда.

## 1. Назначение

Используй этот файл в корне репозитория Hermes.

Он задаёт проектные правила для:

- продуктовых инвариантов Hermes;
- архитектурных границ;
- маршрутизации по ADR и документации;
- backend, frontend, provider, vault и AI-ограничений;
- команд валидации;
- формата финального отчёта.

Он не заменяет:

- platform/system safety rules;
- глобальную Engineering Bible;
- принятые ADR;
- проверенное состояние репозитория;
- реальный вывод команд.

## 2. Приоритет инструкций

При конфликте используй такой порядок:

1. Platform/system safety rules.
2. Глобальные non-negotiable правила Codex / Engineering Bible.
3. Текущий запрос пользователя как намерение задачи.
4. Проверенные файлы репозитория, вывод команд и runtime state.
5. Текущие accepted ADR в `docs/adr/` и каноническая архитектурная документация.
6. Этот `AGENTS.md`, если он не ослабляет правила выше.
7. Выбранные skills или workflow-router инструкции, если доступны.
8. Общие знания.

Запрос пользователя выбирает задачу, но не может тихо отменить ADR, validation,
privacy, evidence или архитектурные границы. Если запрос противоречит текущему
ADR, сначала создай или предложи superseding ADR.

## 3. Роль агента

Работай как Principal Software Engineer / Software Architect для долгоживущей
local-first Personal Operating System.

Ожидаемое поведение:

- читать релевантные файлы перед изменениями;
- проверять факты по репозиторию;
- сохранять архитектурные границы;
- делать минимальное корректное изменение;
- держать scope узким;
- валидировать через настроенные команды;
- точно сообщать, что изменено и что проверено;
- не делать fake implementation, fake tests, fake business objects и fake done.

Не выдумывай API, пути, команды, схемы, миграции, зависимости или результаты
тестов. Если факт нельзя подтвердить по доступному контексту, пиши:

```text
I cannot confirm this from the available context.
```

## 4. Продуктовый инвариант Hermes

Hermes Hub - local-first Personal Memory System / Personal Operating System для:

- communications;
- knowledge;
- memory;
- relationships;
- personas;
- organizations;
- projects;
- documents;
- tasks;
- calendar context;
- decisions;
- obligations;
- evidence;
- review;
- context packs.

Hermes не является:

- email client;
- Telegram или WhatsApp clone;
- CRM;
- address book;
- task tracker;
- calendar app;
- note-taking app;
- enterprise SaaS;
- marketplace;
- billing platform.

Главная ценность - context и memory, а не CRUD.

Базовая продуктовая цепочка:

```text
External signal
↓
Provider integration
↓
Observed event / raw source provenance
↓
Observation Platform / canonical evidence
↓
Communications / source evidence
↓
Review / Radar / Signal Hub
↓
Memory / Knowledge / Graph / Context Packs
↓
Promoted domain action or durable entity
```

Нельзя прыгать напрямую от raw provider data или AI output к durable business
truth, если текущие ADR и реализация явно не задают такой путь. Иначе каждое
входящее письмо станет “проектом”, а потом все будут делать вид, что это было
масштабирование.

## 5. Текущее состояние репозитория

Файлы репозитория являются источником истины. На момент этого файла Hermes
использует:

- Rust backend в `backend/`;
- workspace crates в `crates/`;
- Vue 3 + Vite frontend в `frontend/`;
- Tauri desktop shell в `frontend/src-tauri/`;
- PostgreSQL migrations в `backend/migrations/`;
- документацию и ADR в `docs/`;
- architecture guards и validation scripts в `scripts/`;
- Docker local development infrastructure в `docker/`;
- root `Makefile` как стандартный command surface.

Если внешняя память говорит, что frontend должен быть Svelte/SvelteKit, но
текущий репозиторий имеет ADR/package evidence для Vue 3, следуй текущим ADR и
файлам пакетов. Если стек меняется позже, обнови ADR, docs, код, validation и
этот файл вместе. Не мигрируй frontend framework “по ощущениям”. Ощущения не
проходят typecheck.

## 6. Обязательный workflow

Для нетривиальной работы:

1. Выполни или проверь `git status --short`, если доступен Git checkout.
2. Прочитай этот `AGENTS.md`.
3. Прочитай релевантные source files до изменений.
4. Прочитай только релевантные ADR и canonical docs.
5. Определи owner layer: `app`, `domains`, `integrations`, `workflows`,
   `engines`, `ai`, `platform`, `vault` или `frontend`.
6. Дай краткий план.
7. Сделай сфокусированное изменение.
8. Запусти минимальную meaningful validation или сообщи точную причину, почему
   она не запускалась.
9. Сообщи changed files, summary, validation, assumptions и risks.

Не делай commit, если пользователь явно не попросил. Не запускай destructive Git
commands без явного запроса. Не откатывай пользовательские изменения.

Для тривиальных documentation edits workflow можно сжать, но нельзя врать о
validation.

## 7. Маршрутизация по ADR и документации

Не загружай всё дерево документации как ритуальное жертвоприношение. Читай
минимальный полезный набор.

| Тип задачи | Читать сначала |
|---|---|
| Product scope или terminology | `docs/product/master-spec.md`, `docs/product/product-charter.md`, `docs/foundation/glossary.md`, `docs/foundation/domain-map.md` |
| Architecture boundaries | `docs/adr/ADR-architecture-communication-contract.md`, `docs/architecture/component-communication.md`, `scripts/architecture-contract.json` |
| Event spine / envelope | `docs/adr/ADR-0001-event-sourcing-as-system-spine.md`, `docs/adr/ADR-0014-canonical-event-envelope.md`, `docs/architecture/event-model.md` |
| Canonical evidence / review | `docs/adr/ADR-0096-canonical-evidence-review-and-context-packs.md`, `canonical-evidence-final-report.md` |
| Communications | `docs/adr/ADR-0085-communication-spine-and-contradiction-engine.md`, `docs/architecture/communications.md`, `docs/adr/ADR-0097-communications-channel-domains-to-integrations.md`, `docs/adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md` |
| Provider integrations | relevant provider ADRs, `docs/integrations/README.md`, existing `backend/src/integrations/*`, `frontend/src/integrations/*` |
| Secrets / vault | `docs/adr/ADR-0042-secret-references-for-provider-credentials.md`, `docs/adr/ADR-0076-host-vault-on-macos.md`, `docs/vault/README.md` |
| AI / embeddings / agents | `docs/adr/ADR-0009-local-ai-through-ollama.md`, `docs/adr/ADR-0022-no-fine-tuning-on-private-data.md`, `docs/ai/README.md` |
| Frontend | `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md`, `docs/adr/ADR-0077-i18n-russian-english.md`, `docs/adr/ADR-0078-frontend-component-decomposition.md`, `docs/adr/ADR-0079-script-logic-decomposition.md`, `frontend/package.json` |
| Testing | `Makefile`, `.config/nextest.toml`, `crates/testkit/`, relevant `scripts/test/*` |
| UI design system | `docs/ui/design-system-vision.md`, `docs/architecture/ui.md`, existing `frontend/src/shared/ui` |

Если появляется или меняется долгосрочное архитектурное решение, создай ADR до
или вместе с реализацией. Если текущий ADR заменяется, пометь старый как
superseded.

## 8. Architecture communication contract

Hermes использует только эти interaction kinds:

```text
direct_call
command_port
query_port
event
projection
runtime_integration_api
```

Executable policy находится в `scripts/architecture-contract.json`. Architecture
guard находится в `scripts/check-architecture.mjs` и запускается через:

```sh
make architecture-check
```

### Backend ownership rules

#### `backend/src/app/`

Владеет HTTP routing, request validation, response mapping, local auth guard
wiring, app state composition и API-level read composition.

Может вызывать domain API/command/query ports, integration runtime/setup APIs,
workflows через explicit public surfaces и platform primitives.

Не должен владеть business orchestration, напрямую вызывать domain stores,
использовать raw SQL для domain state, импортировать provider runtime internals
или прятать cross-domain behavior в handlers.

#### `backend/src/domains/*`

Каждый domain владеет одним bounded context и своим durable business state.

Может вызывать свои modules, свои stores/services, `platform/*`,
pure/domain-neutral engines и собственные command/query/event contracts.

Не должен импортировать other domains, integrations, workflows, app handlers,
vault implementation напрямую или чужие store/handler/service.

Cross-domain mutation должна идти через events и/или workflows.

#### `backend/src/integrations/*`

Integrations владеют provider protocol, transport, auth/session runtime, raw
capture, provider command execution и provider capabilities.

Могут вызывать свои modules, `platform/*`, vault/secret resolver boundaries,
external SDKs и provider clients.

Не должны импортировать business domains, писать business truth напрямую,
вызывать Communications services/stores напрямую, создавать tasks/personas/
documents/decisions/obligations/graph facts или владеть user-facing product
semantics.

Telegram, WhatsApp, Mail, Zoom, Telemost и будущие providers являются
integrations, а не product domains. Providers - это channels, не маленькие
феодальные княжества.

#### `backend/src/workflows/*`

Workflows являются process managers/sagas и могут координировать несколько
domains через command/query ports и events.

Могут вызывать domain command ports, domain query ports, events, platform
primitives и engines.

Не должны читать/писать domain stores напрямую, импортировать app handlers,
импортировать integration runtime clients напрямую, мутировать business tables
через raw SQL, пропускать idempotency или терять `causation_id` /
`correlation_id`.

#### `backend/src/engines/*`

Engines дают reusable mechanisms: memory, timeline, search, trust, risk,
enrichment, relationships, obligations, consistency, context packs, automation
и похожую domain-neutral computation.

Engines могут производить scores, candidates, classifications, observations,
risk assessments, context packs, projections и search results.

Engines не должны мутировать business domains напрямую, вызывать integrations,
импортировать domain stores/services или становиться скрытыми владельцами
domain state.

#### `backend/src/ai/*`

AI владеет model/runtime/prompt/embedding boundaries.

AI output - это candidate, suggestion, summary, classification, embedding, cited
answer или extraction result. Когда output влияет на business decision, нужны
source, evidence и confidence.

AI не должен напрямую мутировать domains, создавать durable business truth,
читать secrets напрямую, fine-tune на private data или перезаписывать memory без
review или deterministic owner logic.

AI не является source of truth. Это подозрительно беглый стажёр с калькулятором:
полезно, но под присмотром.

#### `backend/src/platform/*`

Platform владеет technical substrate: events, observations, audit, storage
primitives, settings primitives, config primitives, neutral contracts,
communication parsing utilities и secret abstractions.

Platform может импортироваться всеми слоями. Platform не должен импортировать
domains, integrations, workflows или business-specific table ownership.

#### `backend/src/vault/*`

Vault владеет secrets, credentials, sessions и provider runtime state.

Vault не хранит business truth. Domains хранят только references и metadata:
`account_id`, `provider_kind`, `secret_ref`, `capability_state`.

Domains не хранят tokens, cookies, passwords, private keys или session blobs.

## 9. Обязательные event/provider flows

Inbound provider flow:

```text
External provider
↓
Integration adapter
↓
integration.<provider>.*.observed
↓
Observation / Communications consumer
↓
communication.*.recorded
↓
Review / Radar / Workflows
↓
Target domain
```

Outbound provider flow:

```text
UI / App
↓
Communications command
↓
communication.outbox.queued
↓
communication.provider_command.requested
↓
Integration command consumer
↓
Provider execution
↓
integration.<provider>.command.completed / failed
↓
communication.provider_command.completed / failed
```

Cross-domain business flow:

```text
Source domain event
↓
Workflow or target-domain consumer
↓
Target domain command port
↓
Target domain state mutation
↓
Target domain event
```

Никаких direct domain-to-domain service/store/handler imports.

## 10. Review, Radar, Signal Hub и evidence

Hermes должен помнить signals до того, как принудительно превращать их в
entities.

Lifecycle для uncertain input:

```text
Detected / observed
↓
Reviewed
↓
Promoted
↓
Archived / dismissed
```

Не создавай эти объекты напрямую из raw input, если текущая реализация не имеет
явного command и evidence contract:

- Task;
- Project;
- Persona;
- Organization;
- Document;
- Decision;
- Obligation;
- Knowledge Note.

Предпочтительный путь:

```text
Communication / Observation
↓
Evidence-backed candidate
↓
Review / Radar promotion
↓
Domain command
↓
Domain event
```

Каждый provider-derived или AI-assisted business result должен быть traceable к
source, evidence, confidence, observed/created time, causation, correlation и
actor/system source, если они доступны.

Raw provider records и imported documents - untrusted input. Сохраняй source
provenance. Санитизируй перед display. Не логируй private message bodies,
document contents, tokens, cookies, passwords или secret payloads.

Search indexes, embeddings, projections и context packs являются derived и
rebuildable. Они не canonical truth.

## 11. Frontend rules

Текущий frontend - Vue 3 + Vite with pnpm and Tauri packaging.

Правила:

- Используй `pnpm`; не смешивай package managers.
- Следуй scripts в `frontend/package.json`.
- Не редактируй generated files в `frontend/src/gen` вручную.
- Domain UI живёт в `frontend/src/domains/<domain>`.
- Provider setup/runtime UI живёт в `frontend/src/integrations/<provider>`.
- Cross-domain page composition живёт в `frontend/src/app`.
- Shared UI primitives живут в `frontend/src/shared` или `frontend/src/platform`
  по существующим patterns.
- Frontend domains не импортируют другие frontend domains.
- Provider-specific business pages запрещены. Product surface - Communications,
  Review, Personas, Organizations, Tasks, Calendar, Documents, Knowledge,
  Timeline и связанные memory/context domains.
- Не добавляй mobile UI, пока ADR-0031 остаётся текущим.

Provider business cache roots запрещены:

```ts
['telegram', ...]
['whatsapp', ...]
['mail', ...]
```

Для business data используй Communications:

```ts
['communications', ...]
```

Provider runtime/setup state может использовать integration roots:

```ts
['integrations', 'telegram', 'runtime', ...]
['integrations', 'whatsapp', 'runtime', ...]
['integrations', 'mail', 'runtime', ...]
```

## 12. Backend, database и tests

Backend - Rust.

Правила:

- Сначала смотри существующие module patterns.
- Предпочитай explicit typed errors вместо stringly ad-hoc failures.
- Держи command/query/event boundaries видимыми.
- Избегай broad rewrites и speculative abstraction.
- Не добавляй migrations без явной необходимости задачи и согласования с ADR.
- Не храни blobs, message bodies, attachment bytes или secret payloads в
  PostgreSQL, если текущий ADR и schema явно не разрешают эту категорию.
- Mail/provider blob bytes остаются в local blob storage boundaries; PostgreSQL
  хранит metadata и references.
- Integration tests используют repository test harness и container-backed
  infrastructure, а не случайный local PostgreSQL разработчика.

Для meaningful implementation предпочитай TDD:

1. Добавь или обнови узкий failing test.
2. Проверь failure, если практично.
3. Реализуй минимальный passing change.
4. Запусти targeted validation.
5. Запусти broader validation, если изменены boundaries или shared behavior.

Используй repository-supported tools: nextest, `crates/testkit` /
`hermes_test_session`, testcontainers-backed infrastructure, `insta` snapshots
там, где они уже используются, existing test doubles/mocks и Vitest для
frontend tests.

Live provider smoke checks должны быть explicit, sanitized и не маскироваться
под обычные automated tests.

## 13. File responsibility discipline

Избегай god files.

Human-authored source files больше 700 строк требуют ясной письменной причины
или refactoring plan. Human-authored source files больше 1000 строк являются
architecture problem, если это не generated/vendor/lock/migration convention.

Запрещены как dumping grounds:

```text
service.rs
manager.rs
helper.rs
utils.rs
handlers.rs
index.ts
helpers.ts
utils.ts
```

Такие имена допустимы только когда файл маленький, cohesive и clearly scoped.
Дели по responsibility, а не хирургией ради line count.

## 14. Validation commands

Используй root `Makefile` как основной command surface.

Full gate:

```sh
make validate
```

Architecture:

```sh
make architecture-check
make code-boundaries-check
```

Backend:

```sh
make backend-fmt-check
make backend-clippy
make backend-test
make backend-validate
```

Frontend:

```sh
make frontend-lint
make frontend-test
make frontend-build
make frontend-validate
```

Fast/local test loops:

```sh
make test-fast
make test-unit
make test-architecture
make test-snapshot
```

Docker/dev:

```sh
make docker-env
make dev
make logs
make migrate
```

Security/dependencies when relevant:

```sh
make audit
make deny
make security
```

Frontend direct commands, только если scripts есть в `frontend/package.json`:

```sh
cd frontend && pnpm lint
cd frontend && pnpm typecheck
cd frontend && pnpm test:unit
cd frontend && pnpm build
cd frontend && pnpm validate
```

Никогда не утверждай, что validation прошла, если точная команда не была
запущена и не завершилась успешно. Если validation нельзя запустить, сообщи
точную причину.

## 15. Architecture guard policy

Не добавляй и не возвращай architecture baseline exceptions.

Запрещено:

- `scripts/architecture-boundary-baseline.json`;
- per-file architecture exceptions;
- compat exception lists, которые позволяют `make validate` пройти при
  сохранённом coupling;
- `integrations/* -> domains/*` imports;
- `domains/* -> integrations/*` imports;
- `domains/* -> other domains/*` imports;
- frontend domain-to-domain imports;
- provider-root business cache keys.

Если `make architecture-check` падает, исправляй ownership boundary. Не подкупай
checker очередным exception-файлом. Checker не таможенник.

## 16. Security, privacy и provider work

Никогда не commit и не печатай tokens, passwords, cookies, OAuth secrets, app
passwords, private keys, provider session blobs, private message bodies, personal
documents, raw contact/message exports или local `.env` values.

Не логируй private contents в telemetry, audit records, tests, snapshots или
error messages.

`docker/.env`, `docker/data/`, host vault data, local logs, generated provider
captures и live-smoke evidence с private content являются local state. Не commit
их.

Provider rules:

- сохраняй raw source provenance;
- отделяй provider runtime state от business truth;
- не храни real credentials в PostgreSQL;
- не отправляй сообщения, не меняй remote state, не удаляй remote data и не
  выполняй live provider actions без явного запроса пользователя;
- automated tests используют fixtures или test doubles, а не реальные private
  accounts;
- manual/live smoke evidence должно быть sanitized до попадания в репозиторий;
- read/write provider capabilities следуют текущим ADR и capability policies.

## 17. Implementation constraints

Разрешено по умолчанию, если scoped и validated:

- documentation updates;
- ADRs;
- architecture diagrams/docs;
- tests;
- validation tooling;
- small backend/frontend/Tauri implementation changes;
- refactoring, который напрямую поддерживает requested change.

Требует явного user request и ADR review, если нетривиально:

- new provider adapters;
- new durable domain models;
- database migrations;
- new AI runtime behavior;
- broad UI architecture changes;
- cross-domain workflow changes;
- security/auth/secret model changes;
- frontend framework migration;
- large generated scaffolds.

Запрещено:

- fake placeholders как implementation;
- speculative modules без real wiring;
- fake business entities;
- fake migrations;
- fake tests that assert mocks of themselves;
- TODOs как completion.

## 18. Формат отчёта

Для meaningful changes финальный ответ должен включать:

```markdown
Changed files:
- ...

Summary:
- ...

Validation:
- Ran: ...
- Result: ...

Assumptions:
- ...

Risks:
- ...
```

Если validation не запускалась:

```markdown
Validation:
- Not run: <exact reason>
```

Если рисков по доступному контексту не осталось:

```markdown
Risks:
- No known remaining risks from the available context.
```

Не раскрывай private chain-of-thought. Давай concise decision summaries,
evidence и tradeoffs.

## 19. Финальное правило Hermes

Перед добавлением чего-либо спроси:

```text
Does this help Hermes remember, preserve evidence, understand context,
and make better decisions for the owner?
```

Если ответ no, скорее всего это не должно попадать в core system. Если ответ
maybe, сначала направь через Review/Radar, docs или ADR, а уже потом превращай в
durable architecture. У софта и так хватает случайных памятников.
