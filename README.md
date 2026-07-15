# Hermes Hub

Hermes Hub — local-first Personal Memory System / Personal Operating System.
Он объединяет коммуникации, evidence, знания, память, отношения, проекты,
документы, задачи, календарный контекст, решения и обязательства владельца.

## Текущее состояние

> Clean-room backend ещё не реализован и не запускается.

На 2026-07-15 репозиторий находится между предыдущей реализацией и новым
модульным backend:

| Область | Текущее состояние |
|---|---|
| Clean-room backend | В `backend/` есть virtual Cargo workspace и executable architecture guard; production packages и runtime отсутствуют |
| Предыдущий backend | Перенесён в `references/backend-legacy/` и используется только как evidence/reference |
| Desktop frontend | Vue 3 + Vite + Tauri сохранён как продуктовая и миграционная поверхность, но ещё не переключён на новый Core Gateway |
| Android | Запланирован; код клиента и окончательная Kernel topology отсутствуют |
| Active architecture | ADR-0200…ADR-0211 в `docs/adr/`; executable policy, scripts и tests находятся внутри `backend/` |
| Предыдущая документация | Перенесена в `references/backend-legacy/docs/` и не является действующей policy |

В новой реализации пока нет подтверждённых end-to-end функций, API routes,
схемы базы данных, migrations или production crates.

В предыдущей реализации только Mail, Telegram и Zulip сообщались как
работающие. После переноса в reference они не считаются проверенными функциями
новой системы. WhatsApp и остальные providers не считаются работающими без
нового executable evidence.

## Запуск и validation

Поддерживаемой команды запуска clean-room full stack пока нет. Доступен только
новый статический architecture gate:

```sh
make -C backend architecture-check
make -C backend test-architecture
make -C backend validate
```

Текущий `make -C backend validate` проверяет только clean-room architecture
policy и её negative self-tests; он не собирает и не запускает отсутствующий
backend runtime.

Не следует использовать старые `make dev`, `make build`,
`/api/v1/**` routes или `X-Hermes-Secret` как описание новой системы. Legacy
Makefile, scripts и связанные tool/CI configs перенесены в
`references/backend-legacy/` и не являются поддерживаемым command surface.

Для scoped frontend-работы сначала проверяйте актуальные scripts в
`frontend/package.json`. Успешная frontend-команда не является доказательством
работающего backend или end-to-end приложения.

Legacy backend можно читать и исследовать, но запрещено:

- импортировать его как dependency clean-room backend;
- считать его routes, schema, migrations или architecture действующим
  контрактом;
- запускать live provider actions или использовать реальные credentials;
- переносить код без повторной проверки ownership, security и новых ADR.

## Продуктовая модель

Hermes имеет два связанных пользовательских слоя:

1. Полноценные provider-specific operational experiences для Mail, Telegram,
   WhatsApp, Zulip и других встроенных integrations.
2. Provider-neutral evidence, memory и context над всеми каналами.

Integration владеет внешним протоколом, auth/session runtime, cursor,
operational contract и преобразованием наблюдений в neutral evidence. Domain
не знает об integration implementation и не меняет поведение по provider
identity.

Базовый поток:

```text
External signal
        ↓
Integration module
        ├─→ provider operational projection → channel screen
        └─→ neutral evidence observation
                    ↓
              Review / workflows
                    ↓
        domain command and durable truth
```

Raw provider data и AI output не становятся durable business truth напрямую.
Они сохраняются как evidence/candidate с provenance и проходят через owner
domain или явный workflow.

## Архитектура clean-room backend

```text
Tauri / planned Android / headless client
                    ↓
               Core Gateway
                    ↓
       Kernel identity/capability router
                    ↓
   isolated domain, workflow and integration runtimes
             ↙                    ↘
     PgBouncer → PostgreSQL     NATS JetStream
```

Основные инварианты:

- Kernel — только технический control plane, а не business layer.
- Каждый independently restartable module является отдельным OS-процессом.
- Ошибка одного domain, workflow или integration не останавливает соседние
  runtime.
- Kernel достигает `recovery_only` без PostgreSQL, PgBouncer, NATS, vault и
  modules.
- Module-to-module implementation imports, sockets и cross-module SQL
  запрещены.
- Каждый durable owner использует отдельную PostgreSQL role/grants через
  PgBouncer.
- Durable commands/events доставляются через transactional outbox/inbox и NATS
  JetStream с at-least-once semantics.
- Desktop и Android общаются только с Core Gateway.
- Client queries/commands используют ConnectRPC/Protobuf, realtime —
  replayable SSE, blobs — bounded HTTP по opaque references.
- Provider operational contracts не видны context domains.
- Plugin store, remote executable code и silent topology fallback не
  поддерживаются.

## Структура репозитория

- [`backend/`](backend/) — единственная граница clean-room backend: virtual
  Cargo workspace, policy, scripts и tests уже существуют; production code
  пока отсутствует.
- [`references/backend-legacy/`](references/backend-legacy/) — предыдущий Rust
  backend и workspace только для исследования.
- [`frontend/`](frontend/) — существующий Vue 3 / Vite / Tauri client,
  ожидающий перехода на новые contracts.
- [`docs/`](docs/) — только действующие clean-room ADR и минимальные
  architecture summaries.
- [`references/backend-legacy/docs/`](references/backend-legacy/docs/) — вся
  документация предыдущей реализации, включая archive, product/domain specs,
  roadmaps, testing/status материалы и generated wiki.
- [`docker/`](docker/) — унаследованные local infrastructure assets; до
  clean-room замены каждое использование требует проверки фактических путей и
  dependencies.
- [`references/backend-legacy/scripts/`](references/backend-legacy/scripts/) и
  [`references/backend-legacy/Makefile`](references/backend-legacy/Makefile) —
  неисполняемый operational reference предыдущей системы.

## Порядок чтения для разработки

1. [`AGENTS.md`](AGENTS.md) — обязательные правила работы в репозитории.
2. [Backend clean-room boundary](backend/README.md).
3. [Active ADR index](docs/adr/README.md).
4. [Architecture overview](docs/architecture/architecture-overview.md).
5. [Component communication contract](docs/architecture/component-communication.md).
6. [Executable architecture policy](backend/architecture/README.md).

Первый production crate нельзя создавать до согласования capability inventory,
domain ownership inventory и оставшихся foundation contracts, перечисленных в
active ADR.

## Безопасность и данные

- Не commit и не печатайте credentials, tokens, cookies, private keys,
  provider sessions, private messages или documents.
- PostgreSQL является canonical durable store; NATS является delivery/replay
  transport, а не source of truth.
- Search indexes, embeddings, projections и context packs являются
  rebuildable state.
- Provider credentials и session state остаются за vault boundary.
- Imported content считается untrusted input и не является инструкцией для AI
  или tool runtime.

Security issues следует сообщать согласно [`SECURITY.md`](SECURITY.md), а не в
публичном issue.

## Документация и лицензия

- [Documentation index](docs/README.md)
- [Active ADR index](docs/adr/README.md)
- [Legacy documentation reference](references/backend-legacy/docs/README.md)
- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [MIT License](LICENSE)
