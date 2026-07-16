# Активные ADR clean-room backend

Все ADR предыдущей реализации были вынесены из active documentation 2026-07-15.
Исторический индекс находится в
[legacy documentation reference](../../references/backend-legacy/docs/archive/adr/README.md).
Legacy ADR являются только evidence и контекстом; они не возвращаются в active
policy через ссылки из новых документов.

## Статусы

- `Предложено` — решение обсуждается и ещё не принято.
- `Принято` — решение обязательно для новой реализации.
- `Заменено` — решение полностью заменено более новым active ADR.
- `Отклонено` — решение рассмотрено и не используется.

Поле `Состояние реализации` отделяет принятое решение от факта его реализации.
Статус `Принято` сам по себе не означает, что код уже существует.

## Активные решения

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md)
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md)
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md)
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md)
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md)
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md)
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md)
- [ADR-0207: Канонический реестр бизнес-доменов Hermes](ADR-0207-canonical-business-domain-registry.md)
- [ADR-0208: Allowlist разработки доменов и запрет проекций](ADR-0208-domain-development-allowlist-and-projection-freeze.md)
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md)
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md)
- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md)
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md)
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md)
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md)
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md)
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md)
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md)
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md)
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md)
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md)
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md)
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md)
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md)
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md)
- [ADR-0225: Первый production slice — recovery-only Kernel и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md)
- [ADR-0226: Контекст для AI только через use-case workflows](ADR-0226-ai-context-acquisition-through-use-case-workflows.md)
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md)
- [ADR-0228: Development simulation profile](ADR-0228-development-simulation-profile.md)

Эти ADR фиксируют runtime, communication, storage, infrastructure lifecycle и
границу между provider-specific experience и provider-neutral context, а также
единый client gateway для desktop и Android. Конституция Kernel ограничивает
его техническим control plane и фиксирует boot/recovery state machine.
ADR-0225 закрывает inventory первого production slice: разрешены только шесть
foundation packages recovery-only Kernel, а domains, integrations, workflows и
engines пока имеют пустой фактический inventory. Любое расширение требует
открытия соответствующего phase gate через ADR, policy и executable evidence.
Канонический реестр фиксирует тринадцать начальных business domains и отделяет
их от integrations, workflows и projections.
Текущий implementation allowlist разрешает только Communications, Contacts,
Organizations, Tasks, Calendar, Documents и AI; остальные домены и все
product projections заблокированы.
Event Hub является Kernel control plane над NATS catalog/subscriptions, а
Telemetry Hub обеспечивает независимые от PostgreSQL/NATS локальные logs,
metrics, traces и crash diagnostics через отдельный supervised Collector.
ADR-0211 помещает весь production backend code в `backend/src`, а policy,
scripts, infrastructure и tests — в отдельные backend-owned roots внутри
`backend/`.
ADR-0212 запрещает compile-graph aggregation, отделяет Kernel/Gateway от
owner-specific packages и фиксирует owner-local package topology, включая
узкий Communications ingress для всех integrations. Telegram в ADR является
примером protocol-specific split, а не особым архитектурным случаем.
ADR-0213 определяет SRP через owner, ответственность и причину изменения,
задаёт практическую интерпретацию SOLID/KISS/DRY/YAGNI и проверяемую автономность
каждого module в build, tests, lifecycle, data и failure boundaries.
ADR-0214 отделяет Scheduler от Kernel/Event Hub, оставляет исполняемый job code
в module-владельце и фиксирует durable schedules, owner-local execution,
default reconciliation и горячее изменение runtime policy без загрузки кода из
database.
ADR-0215 разрешает любому локальному process пройти недоверенную регистрацию,
но до явного approval оставляет его без capabilities. Effective grants являются
пересечением module request, owner settings и hard Kernel policy; `managed` и
`external` lifecycle имеют разные restart guarantees, а обязательная подпись
binary не является admission condition первой версии.
ADR-0216 сохраняет registrations, grant epochs и desired infrastructure state
в private kernel-owned SQLite через отдельный persistence adapter. Kernel
стартует и поднимает local recovery surface без PostgreSQL, PgBouncer, NATS,
Vault и modules; business data и secrets в Control Store запрещены.
ADR-0217 запрещает обязательный bootstrap configuration file и любые
Hermes-specific environment overlays. Default data directory определяется
операционной системой, explicit `--data-dir` выбирает отдельный instance, а
недоверенный Control Store оставляет только restricted local recovery.
ADR-0218 отделяет logical OwnerAuthority от OS identity и module
processes. Каждое device имеет отдельную отзываемую ES256 keypair,
private key остаётся в platform signer, а online recovery недоверенного
Control Store ограничен sanitized `status/validate/export`.
ADR-0219 сохраняет open `pending` registration без publisher signature,
но требует signed distribution entry либо owner-pinned digest для
любого `managed` process. Kernel проверяет exact bytes перед каждым
launch, не скачивает code и не выполняет automatic rollback.
ADR-0220 фиксирует binary `DurableEnvelopeV1`, exact contract/schema binding,
byte-for-byte outbox-to-NATS delivery, пять message kinds, отдельный technical
DLQ record и строгую границу между internal data plane и client SSE.
ADR-0221 разделяет signed distribution inventory, runtime descriptor,
effective grants и observed state. `ModuleDescriptorV1` является exact
Protobuf declaration, а capability становится единицей approval, readiness,
dependency resolution и revoke; managed binding pin-ит descriptor digest.
ADR-0222 делает Settings Registry обязательным Kernel component. Module
владеет schema и смыслом полей, Kernel — typed desired/effective revisions в
private Control Store, validation/application и supervised restart. Secrets,
business/runtime state и Scheduler records настройками не являются.
ADR-0223 выделяет Vault в отдельный verified managed process. Kernel вычисляет
grants и маршрутизирует только HPKE ciphertext, а Vault хранит bounded credential
material в SQLCipher с record-level AEAD и выдаёт process-bound leases. Bulk
provider session state остаётся у integration owner. Решение принято, но
production packages, storage format и conformance tests ещё не реализованы.
ADR-0224 выделяет Storage Control в отдельный managed control-plane process.
Kernel supervises PostgreSQL, PgBouncer и Storage Control; modules выполняют
business SQL напрямую через PgBouncer, а Storage Control владеет bootstrap,
roles/grants/budgets, migration admission и readiness. Runtime credentials
выдаёт Vault, а PgBouncer не считается единственной security boundary. Target
принят, но production packages и process-level isolation tests отсутствуют.
ADR-0225 разрешает первый строго ограниченный production graph, но он ещё не
реализован. Kernel может достичь только `recovery_only`, активирует лишь
`supervisor` и локальный `core_gateway`, не запускает внешние сервисы и не
содержит business owners. Managed launch, NATS, Blob, public Clock, Scheduler,
public client gateway, whole-instance backup и первый owner закрыты отдельными
фазовыми воротами.
ADR-0226 запрещает AI прямой доступ к таблицам и query APIs других owners.
Cross-owner AI context собирает отдельный use-case workflow через явные public
contracts в distinct generated request с common `AiContextReceiptV1` и
concrete use-case context. Global fragment union, opaque payload bytes,
generic Context API и durable Context projection остаются заблокированы.
ADR-0228 вводит отдельный full-platform development profile для local
development всех platform components с software trust adapters и local services.
Он не является deployment profile и никогда не служит evidence для production
gates.
