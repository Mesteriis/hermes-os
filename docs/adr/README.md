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
- [ADR-0229: Platform Clock contract and deterministic conformance](ADR-0229-platform-clock-contract-and-deterministic-conformance.md)
- [ADR-0230: Blob Platform — opaque references and owner-local metadata](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md)
- [ADR-0231: Private Blob data session and Vault route](ADR-0231-private-blob-data-session-and-vault-route.md)
- [ADR-0232: Browser client identity and same-origin Gateway session](ADR-0232-browser-client-device-identity-and-same-origin-session.md)
- [ADR-0233: Scoped local recovery export and PostgreSQL dump](ADR-0233-whole-instance-backup-and-fenced-restore.md)
- [ADR-0234: Browser-local key binding for synchronised passkeys](ADR-0234-browser-local-key-binding-for-synchronised-passkeys.md)
- [ADR-0235: Private-LAN developer mode](ADR-0235-private-lan-developer-mode.md)
- [ADR-0236: Integration owners, protocol adapters и configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md)
- [ADR-0237: Временный private-LAN development без owner authority](ADR-0237-temporary-private-lan-development-without-owner-authority.md)
- [ADR-0238: Secure-file FD boundary](ADR-0238-secure-file-fd-boundary.md)
- [ADR-0249: Communications profile for storage_control_v1](ADR-0249-communications-storage-control-v1-admission-profile.md)
- [ADR-0250: Communications profile for nats_data_plane_v1](ADR-0250-communications-nats-data-plane-v1-admission-profile.md)
- [ADR-0251: Opening client_gateway_v1 for owner contracts](ADR-0251-client-gateway-v1-opening-for-owner-contracts.md)
- [ADR-0252: first_owner_v1 Communications admission](ADR-0252-first-owner-v1-communications-admission.md)
- [ADR-0253: Communications legacy surface disposition](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md)
- [ADR-0254: Communications derived search index](ADR-0254-communications-derived-search-index-and-private-content-boundary.md)
- [ADR-0255: Managed owner-key leases](ADR-0255-managed-owner-key-leases-for-derived-projections.md)
- [ADR-0256: Owner-declared client RPC route admission](ADR-0256-owner-declared-client-rpc-route-admission.md)
- [ADR-0257: Event-backed Blob custody transfer](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md)
- [ADR-0258: Correlated duplex managed-control transport](ADR-0258-correlated-duplex-managed-control-transport.md)
- [ADR-0259: Separate typed platform-control path](ADR-0259-separate-typed-platform-control-path.md)
- [ADR-0260: Communications attachment lifecycle event authority](ADR-0260-communications-attachment-lifecycle-event-authority.md)
- [ADR-0261: Communications attachment-anchor handoff](ADR-0261-communications-attachment-anchor-handoff.md)
- [ADR-0262: Mail attachment Blob-admission extension](ADR-0262-mail-attachment-blob-admission-extension.md)
- [ADR-0263: Mail integration settings and Storage admission artifacts](ADR-0263-mail-integration-settings-and-storage-admission.md)
- [ADR-0264: Communications message evidence history query](ADR-0264-communications-message-evidence-history-query.md)
- [ADR-0265: Provider operational client transport admission](ADR-0265-provider-operational-client-transport-admission.md)
- [ADR-0266: Telegram Kernel admission and event-only Communications handoff](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md)
- [ADR-0267: Kernel-staged runtime artifacts and integration state roots](ADR-0267-kernel-staged-runtime-artifacts-and-integration-state-roots.md)
- [ADR-0268: Telegram release composition](ADR-0268-telegram-release-assembly-unit-and-signed-distribution-fragment.md)
- [ADR-0269: Mail release composition](ADR-0269-mail-release-assembly-unit-and-signed-distribution-fragment.md)
- [ADR-0270: Mail capability split](ADR-0270-mail-kernel-admission-and-route-specific-event-handoff.md)
- [ADR-0271: Zulip phase gate](ADR-0271-zulip-kernel-admission-and-event-only-communications-handoff.md)
- [ADR-0272: Zulip release composition](ADR-0272-zulip-release-assembly-unit-and-signed-distribution-fragment.md)
- [ADR-0273: Attachment Security engine](ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md)
- [ADR-0274: Attachment Security Blob custody](ADR-0274-attachment-security-evidence-bound-blob-custody.md)
- [ADR-0275: Target-bound cross-owner Blob custody](ADR-0275-target-bound-cross-owner-blob-custody-delegation.md)
- [ADR-0276: WhatsApp phase gate](ADR-0276-whatsapp-kernel-admission-host-bridge-and-event-only-communications-handoff.md)
- [ADR-0277: Gmail API outbound mutation gate](ADR-0277-mail-gmail-api-outbound-mutation-gate.md)
- [ADR-0278: Gmail OAuth setup and refresh gate](ADR-0278-mail-gmail-oauth-setup-and-refresh-gate.md)
- [ADR-0279: Durable Blob custody scope and operation-scoped grants](ADR-0279-durable-blob-custody-scope-and-operation-scoped-grants.md)
- [ADR-0280: Mail event-gated outbound MIME attachments](ADR-0280-mail-event-gated-outbound-mime-attachments.md)
- [ADR-0281: Communications frontend clean-room composition](ADR-0281-communications-frontend-clean-room-composition.md)
- [ADR-0282: Full Communications and Settings capability reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md)
- [ADR-0283: Telegram automation management and preview boundary](ADR-0283-telegram-automation-management-and-preview-boundary.md)
- [ADR-0284: Telegram one-to-one audio calls operational boundary](ADR-0284-telegram-one-to-one-audio-calls-operational-boundary.md)
- [ADR-0285: Owner-local upgrade jobs and Telegram Calls realtime backfill](ADR-0285-owner-local-upgrade-jobs-and-telegram-calls-realtime-backfill.md)
- [ADR-0286: WhatsApp operational read and realtime boundary](ADR-0286-whatsapp-operational-read-and-realtime-boundary.md)
- [ADR-0287: Telegram operational realtime replay boundary](ADR-0287-telegram-operational-realtime-replay-boundary.md)
- [ADR-0288: Managed successor quiesce and Storage fence order](ADR-0288-managed-successor-quiesce-and-storage-fence-order.md)
- [ADR-0289: Telegram folder reassignment convergence boundary](ADR-0289-telegram-folder-reassignment-convergence-boundary.md)
- [ADR-0290: Telegram account runtime reconfiguration boundary](ADR-0290-telegram-account-runtime-reconfiguration-boundary.md)
- [ADR-0291: Zulip account, history, operational query and replay boundary](ADR-0291-zulip-account-history-query-and-replay-boundary.md)
- [ADR-0292: Managed integration settings apply and credential binding](ADR-0292-managed-integration-settings-apply-and-credential-binding.md)
- [ADR-0293: Scoped Vault credential retirement and deletion](ADR-0293-scoped-vault-credential-retirement-and-deletion.md)
- [ADR-0294: Mail account credential lifecycle and portability](ADR-0294-mail-account-credential-lifecycle-and-portability.md)
- [ADR-0295: Owner write-only Vault provisioning through Core Gateway](ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md)
- [ADR-0296: Owner module Settings through Core Gateway](ADR-0296-owner-module-settings-through-core-gateway.md)
- [ADR-0297: Fresh owner-proof effective module Settings export](ADR-0297-fresh-owner-proof-effective-module-settings-export.md)
- [ADR-0298: Mail operational read projection and client contract](ADR-0298-mail-operational-read-projection-and-client-contract.md)
- [ADR-0299: Mail sync run history and provider-path health](ADR-0299-mail-sync-run-history-and-provider-path-health.md)
- [ADR-0300: Loopback full-stack development assembly](ADR-0300-loopback-full-stack-development-assembly.md)
- [ADR-0301: Bundled module discovery and development admission](ADR-0301-bundled-module-discovery-and-development-admission.md)
- [ADR-0302: Bundled managed Settings and first runtime bootstrap](ADR-0302-bundled-managed-settings-and-runtime-bootstrap.md)

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
provider session state остаётся у integration owner. Exact `vault_v1`
production packages, storage format и conformance tests реализованы;
whole-instance backup открыт ADR-0233.
ADR-0224 выделяет Storage Control в отдельный managed control-plane process.
Kernel supervises PostgreSQL, PgBouncer и Storage Control; modules выполняют
business SQL напрямую через PgBouncer, а Storage Control владеет bootstrap,
roles/grants/budgets, migration admission и readiness. Runtime credentials
выдаёт Vault, а PgBouncer не считается единственной security boundary. Target
принят, но production packages и process-level isolation tests отсутствуют.
ADR-0225 зафиксировал исходный recovery-only production graph. Последующие
атомарные gates открыли managed platform runtimes, NATS, Blob, Scheduler,
public client Gateway, whole-instance backup и первый owner Communications.
Текущий Kernel по-прежнему честно сообщает `module_control_plane`; отдельный
production state `ready` не заявляется.
ADR-0226 запрещает AI прямой доступ к таблицам и query APIs других owners.
Cross-owner AI context собирает отдельный use-case workflow через явные public
contracts в distinct generated request с common `AiContextReceiptV1` и
concrete use-case context. Global fragment union, opaque payload bytes,
generic Context API и durable Context projection остаются заблокированы.
ADR-0228 вводит отдельный full-platform development profile для local
development всех platform components с software trust adapters и local services.
Он не является deployment profile и никогда не служит evidence для production
gates.
ADR-0229 открывает `clock_v1`: UTC and monotonic reading, explicit
discontinuity policy and deterministic fake clock. It does not open Scheduler,
module timers or timezone/DST calendar evaluation.
ADR-0230 фиксирует Blob Platform boundary: opaque references, owner-local
metadata, Vault-scoped encryption authority, bounded range/path handling and
fenced retention/GC. `blob_v1` открыт после runtime conformance.
ADR-0231 фиксирует следующий mandatory Blob vertical slice: private direct
socket authenticated by a short-lived generation-bound session grant and
ciphertext-only inherited Vault routing. Kernel never receives Blob plaintext.
ADR-0232 включает browser как отдельный first-party client: он получает
owner-approved, revocable, device-bound WebAuthn ES256 identity и только
short-lived same-origin HttpOnly Gateway session. Его owner-neutral
`browser_client_v1` gate открыт отдельно; ADR-0251 затем открывает
`client_gateway_v1` для owner contracts без Gateway-owned business facade.
ADR-0233 открывает `whole_instance_backup_v1`: signed/encrypted media включает
Control Store, Vault, PostgreSQL, Blob, Scheduler и Event Hub topology через
component-owned offline ports, с empty-target restore и generation fencing.
ADR-0234 допускает synchronised WebAuthn passkeys только как одну часть
двухключевой browser identity: session требует ещё и подписи отдельного
non-extractable browser-local WebCrypto key. Новый Mac с синхронизированным
passkey должен пройти новый CLI-approved pairing.
ADR-0235 заменён: persistent LAN owner bypass оказался несовместим с owner
device proof boundary.
ADR-0236 предлагается как уточнение integration granularity: integration
является owner/runtime boundary, protocol/SDK client — owner-local adapter, а
настроенное подключение — opaque configuration instance. Решение не выбирает
первый owner и не открывает `first_owner_v1`.
ADR-0237 оставляет `--dangerous-lan-development` только временным technical
listener без owner APIs: он не сохраняется и не даёт owner authority.
ADR-0238 вводит один FD-bound secure-file contract для bounded no-symlink
readers private material и release inputs; rollout readers остаётся явным
admission prerequisite.
ADR-0239 остаётся историей раннего Mail/IMAP slice. ADR-0252 заменяет временный
owner exception exact admission домена Communications; provider integrations
остаются отдельными units и не входят в owner inventory домена.
ADR-0240 фиксирует Telegram как отдельного integration owner с собственными
operational contracts/state и только typed evidence boundary в Communications.
ADR-0256 реализован как owner-neutral descriptor-declared ClientRpc routing;
Kernel/Gateway не импортируют owner implementations и не декодируют payload.
ADR-0265 запрещает считать legacy Communications REST provider transport.
ADR-0266 задаёт первый exact Telegram phase gate: Kernel владеет только
admission/routing/fencing control plane, а Telegram → Communications handoff
остаётся event-only через integration outbox, NATS и Communications inbox.
ADR-0267 убирает native artifact path и provider session-store directory из
settings: exact runtime dependency приходит из verified managed binding, а
private state root stage-ит Kernel без знания provider semantics.
ADR-0268 выделяет Telegram release composition в отдельную integration-owned
assembly unit: она материализует canonical descriptor/settings/storage bytes и
неподписанный exact artifact fragment, а generic distribution compiler
подписывает только полный release без передачи signing authority integration.
ADR-0269 применяет ту же authority boundary к Mail как к отдельному integration
owner: Mail-owned assembly unit материализует canonical
descriptor/settings/storage bytes и unsigned fragment без native dependency,
а Kernel/Communications/Gateway и Mail runtime не зависят от этой build unit.
ADR-0270 разделяет Mail operational sync/delivery и provider credential
purposes на независимые capability units: integration использует Kernel/Core
для admission и opaque routing, а Communications получает Mail evidence
только через durable typed events.
ADR-0271 задаёт отдельный Zulip phase gate: Kernel/Core владеет только
platform admission, leases, fencing и opaque client routing; Zulip provider
evidence пересекает Communications boundary только через owner-local outbox,
NATS и Communications inbox. Command и operation query становятся разными
capability units, а runtime обязан перейти на один correlated V2 frame pump.
ADR-0272 выделяет Zulip release composition в отдельную integration-owned
assembly unit с exact runtime/settings/storage artifacts и двухэлементным
unsigned fragment. Она не имеет signing authority и не входит в Kernel,
Gateway или Communications.
ADR-0273 вводит отдельный `attachment_security` engine owner: integration
публикует provider-neutral scan candidate, engine durably join-ит его с
canonical Communications `blob_admitted` и публикует typed safety verdict из
собственного outbox. Kernel/Core получает отдельный managed Engine launch
contract и остаётся control plane, а Communications не импортирует scanner
implementation.
ADR-0274 закрывает обнаруженный live conformance разрыв: direct read
integration-owned Blob остаётся запрещён, revision-2 candidate переносит
bounded source custody proof, а engine выполняет evidence-bound transfer в
собственную Blob custody перед one-use read. Kernel не декодирует candidate и
не переносит bytes/verdict.
ADR-0275 устраняет скрытое смешение module owner и human owner в Blob custody:
same-owner proof сохраняет прежний fence, а cross-owner delegation обязана
криптографически bind-ить exact target owner/registration/capability. Audience
принадлежит public owner contract, поэтому integration не импортирует target
runtime implementation, а Kernel не выбирает recipient по business event.
ADR-0276 задаёт отдельный WhatsApp phase gate: Kernel/Core владеет только
admission, leases, fencing, private host-route staging и opaque public client
routing; WhatsApp evidence пересекает Communications boundary только через
owner-local outbox, NATS и Communications inbox. Host bridge, command и query
являются разными capability units, а runtime обязан использовать один
correlated V2 control reader без cloned FD.
ADR-0277 открывает отдельный Gmail delivery gate внутри Mail integration:
outbound-only GrantSet разрешает bounded Gmail HTTPS mutation, owner-local
durable acceptance/query и neutral event replay без IMAP/SMTP/attachment
capabilities или Communications facade.
ADR-0278 задаёт Mail-owned Gmail OAuth setup/refresh gate: Core/Kernel
переносит только opaque owner routes и action-specific Vault ciphertext,
Mail владеет PKCE/operation/binding state, а access и refresh credentials
остаются разными secret classes и capability responsibilities. Gate открыт
после live exact-form/CAS/revoke/negative-output conformance; Communications в
credential lifecycle не участвует.
ADR-0279 разделяет ephemeral Blob access fence и durable at-rest custody:
descriptor объявляет exact custody scope и operation set, Kernel выдаёт только
operation-scoped session, а ciphertext, Vault content-key revision и technical
quota ledger переживают restart/re-registration без generic read/write grant.
ADR-0282 расширяет завершённый frontend ownership/transport cutover до полного
capability reconstruction: Communications остаётся provider-neutral domain,
Mail/Telegram/WhatsApp/Zulip — независимыми integrations, Settings —
app-composition, а cross-owner и AI use cases получают отдельные workflow
units и atomic admission gates.
ADR-0284 разделяет Telegram Calls на independently admitted history, signaling
и real tgcalls media gates: provider calls остаются Telegram integration
surface, а cross-provider evidence и transcription принадлежат отдельным
composition/workflow units. History и signaling gates реализованы отдельными
Query/Command/Realtime capabilities и durable owner-local operation journal;
real audio и Calls umbrella остаются закрыты.
ADR-0285 вводит owner-local upgrade job без fake Scheduler schedule:
owner-neutral Job Platform protocol несёт exact upgrade command, а Telegram
Calls persistence владеет durable execution, lease и checkpoint для
restart-safe V3-to-V4 realtime backfill. Реализованный V6 bundle остаётся
DDL-only; owner executor сохраняет прежние cursors через отдельный replay-order
mapping, а Kernel/Scheduler/Communications не получают Telegram handler или
owner SQL.
ADR-0286 разделяет WhatsApp operational closure на отдельные read и realtime
gates: integration владеет typed projections, bounded search и replay journal,
а Kernel/Gateway только fence-ят exact routes и grants. Metadata-only history
не превращается в fake content; upgrade требует bounded provider resync, а
frontend остаётся вторичным integration-owned consumer.
ADR-0287 добавляет отсутствующую Telegram operational realtime capability:
integration владеет account-scoped ordered journal и explicit cursor reset, а
Kernel/Gateway только допускают exact opaque route. Lifecycle/query aliases и
выдача internal durable envelope клиенту запрещены.
ADR-0288 закрывает общую managed-successor гонку: durable `revoking` остаётся
authority fence, supervisor до physical Storage fence запрещает autorestart
exact predecessor worker, а новый runtime generation резервируется только
после fence и join. Provider-specific retries и перенос lifecycle в integration
запрещены.
ADR-0289 фиксирует честную Telegram folder reassignment semantics: один durable
command сходится к exact target через fresh provider delta и обязательную
финальную проверку, а partial success повторно планируется от текущего TDLib
state. Provider atomicity, stale saved plan и fake terminal `ok` запрещены.
ADR-0290 заменяет fake lifecycle restart отдельным Telegram-owned
`telegram.reconfiguration.v1`: client задаёт только exact intent и expected
epoch, runtime получает fresh Vault leases, физически заменяет TDLib client и
завершает durable target epoch только после restore. Kernel переносит opaque
route и grant, не интерпретируя Telegram lifecycle.
ADR-0291 разделяет полный Zulip experience на account lifecycle, bounded
provider history convergence, owner-local operational query и realtime replay:
Kernel/Core допускают только exact opaque routes и leases, Zulip integration
владеет projection/storage/runtime, а Communications получает neutral evidence
только через durable events.
ADR-0292 устраняет обход Settings Registry при managed integration launch:
Kernel выполняет provider-neutral desired/effective replacement, а credential
revision хранится только как integration-owned Vault binding. Settings, Vault,
integration persistence, runtime и release assembly остаются отдельными
функциональными units.
ADR-0293 закрывает недостающий Vault lifecycle primitive: exact scoped
`retire` удаляет active ciphertext и создаёт durable tombstone, а отдельный
`delete` повышает tombstone до deleted. Kernel согласует только declared
action/grant/runtime fences и не интерпретирует provider logout; integration
выбирает purpose через свой typed lifecycle contract.
ADR-0294 переносит credential revisions из Mail Settings в Mail-owned CAS
bindings: Bind и sanitized Query являются отдельными generated contracts,
текущий runtime quiesce-ит изменённый provider path, а exact Vault revision
активируется только Settings successor generation. Retire/Delete, explicit
Retry и lifecycle Status ведут Mail-owned per-purpose journal, quiesce-ят все
provider paths до exact Vault mutation и сохраняют account tombstone; typed
portability остаётся отдельным незакрытым gate.
ADR-0295 вводит отсутствующий first-party write-only provisioning path:
Core Gateway требует operation-bound fresh device proof, Kernel проверяет
exact approved Vault-purpose capability и переносит только HPKE ciphertext, а
Vault атомарно сохраняет mutation и durable idempotency receipt без record ID
или credential read-back. Backend и client adapters остаются разными gates.
ADR-0296 открывает отсутствующий public Settings path без экспорта private
owner-control protocol: Core Gateway принимает только typed provider-neutral
update/apply intent, Kernel требует fresh active-device proof и сохраняет
authority у Settings Registry, а managed integration replacement остаётся
generic successor operation ADR-0292.
ADR-0297 добавляет отсутствующий fresh-proof export effective Settings:
Core Gateway возвращает только typed client-visible values после проверки
current revision, admitted schema hash и active device, не импортируя Mail или
raw runtime descriptor. Mail собирает свой versioned portability artifact и
resumable multi-receipt import только в first-party integration UI.
ADR-0298 разделяет Mail provider operational projection и Communications
canonical content: Mail владеет bounded folders/threads/messages query, Core
Gateway только маршрутизирует exact contract, а full body app получает через
отдельный Communications content contract по opaque observation anchor.
ADR-0299 отделяет Mail-owned sync run journal и provider-path health от
Scheduler schedules и Communications analytics: exact query возвращает только
bounded sanitized run evidence, restart помечает stale generation как
interrupted, а newsletter detection остаётся Communications-derived use case.
ADR-0300 вводит отдельную непроизводственную assembly boundary для root
`make dev`: loopback Core Gateway и Vite соединяются exact same-origin proxy с
ephemeral server-side proof, readiness проверяется до открытия browser, а
private-LAN technical profile не получает owner authority.
ADR-0301 закрывает отсутствующий generic seam между signed bundled artifact и
pending registration: Kernel проверяет installed manifest и создаёт только
proposal, owner отдельно approve/bind/start-ит units, а development assembly
координирует exact platform/domain/integration plan без provider secrets.
ADR-0302 определяет deterministic development bootstrap для managed Settings и
runtime: assembly применяет только declared typed defaults и generic owner
control operations, не забирая provider semantics у integration units.
ADR-0303 фиксирует provider-owned QR linking: Telegram передаёт transient
TDLib link через existing opaque authorization route и рендерит QR локально,
а WhatsApp оставляет QR внутри owner-visible Tauri WebView. Kernel не становится
generic QR/account service, browser не подделывает native pairing.
ADR-0304 заменяет ложную Zulip bot-only identity на Settings schema major 3 с
`zulip.account_email`: Zulip integration владеет email/API semantics, Kernel
применяет только generic typed settings и не выбирает bot/user behavior.
