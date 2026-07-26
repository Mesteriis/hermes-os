# ADR-0281: Communications frontend clean-room composition

Статус: Принято
Дата: 2026-07-26
Состояние реализации: выполняется по атомарным frontend slices. Exact
client-surface admission, app-owned compiled-adapter registry и active canonical
Communications page реализованы. Active Mail, Telegram, WhatsApp и Zulip
operational pages используют только свои generated contracts; mutation и sync
controls проверяют exact capabilities. WhatsApp browser page не обходит
host-only provider execution. Zulip page экспонирует только доказанные
stream/direct commands и terminal status, не создавая общий chat projection.
App-level Settings workbench, platform-owned System Control и provider-owned
read-only panels для Mail, Telegram, WhatsApp и Zulip реализованы поверх
sanitized bootstrap projection. Provider gates остаются закрыты до удаления
соответствующих scoped legacy REST/query/realtime surfaces.
`settings_frontend_composition_v1` открыт: historical `domains/settings`
orchestrator, mixed Communications settings surfaces, shared runtime setup
wizard и неadmitted settings bridges удалены из active source.

Зависит от:

- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0213: ownership and SRP](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0222: Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0253: Communications legacy disposition](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0256: owner-declared client RPC routes](ADR-0256-owner-declared-client-rpc-route-admission.md);
- [ADR-0263: Mail integration settings](ADR-0263-mail-integration-settings-and-storage-admission.md);
- [ADR-0265: provider operational client transport](ADR-0265-provider-operational-client-transport-admission.md).

## Контекст

Frontend содержит два несовместимых слоя:

1. active recovery shell, который монтирует только System Control;
2. historical Vue surfaces для Communications и Settings, которые смешивают
   canonical evidence, Mail, Telegram, WhatsApp, application settings,
   workflows и legacy REST.

Текущий client bootstrap дополнительно проецирует один
`communications.query.v1` сразу на Mail, Telegram и WhatsApp navigation routes.
Это делает capability домена неявным разрешением UI другой единицы сборки.
Наличие bundled Vue component при этом не доказывает, что owner contract,
runtime, grant и settings текущей integration готовы.

Clean-room перенос не восстанавливает старый page controller, REST facade,
provider switch внутри Communications или общий settings orchestrator.
Отсутствующий owner contract остаётся явным migration gap.

## Решение

### Ownership и единицы сборки

Frontend повторяет backend ownership:

```text
frontend/src/domains/communications
  canonical provider-neutral evidence presentation and owner client adapter

frontend/src/integrations/<provider>
  provider operational presentation, generated client adapter and settings

frontend/src/platform
  owner-neutral Gateway transport, bootstrap and System Control

frontend/src/app
  route selection and cross-owner page/settings composition

frontend/src/shared
  stateless visual primitives only
```

Domain Communications не импортирует integration implementation, provider
contract, provider SDK, provider types или integration settings. Integration не
импортирует Communications domain/runtime и не создаёт canonical business
truth. `app` может одновременно знать публичные presentation contracts разных
owners, но не содержит их storage, query logic или business transformations.

Vue component не становится отдельной единицей сборки сам по себе. Единицей
является owner surface с собственными contract adapter, tests, admission и
reason to change.

### Exact client-surface admission

Kernel bootstrap сообщает только availability известных first-party surfaces.
Он не поставляет labels, markup, frontend code или provider payload.

Каждая surface имеет отдельный stable wire ID и exact admission capability:

| Surface | Required capability | Frontend owner |
|---|---|---|
| canonical Communications | `communications.query.v1` | `domains/communications` |
| Mail operations | `mail.delivery.query.v1` | `integrations/mail` |
| Telegram operations | `telegram.query.v1` | `integrations/telegram` |
| WhatsApp operations | `whatsapp.query.v1` | `integrations/whatsapp` |
| Zulip operations | `zulip.query.v1` | `integrations/zulip` |
| Settings recovery | none; local recovery surface | `platform/system-control` |

Capability одной integration не допускает другую integration и не расширяет
Communications. Command, authorization, lifecycle и optional feature controls
внутри provider page дополнительно проверяют свои exact capabilities из module
bootstrap. Query capability не выдаёт command rights.

Route становится selectable только при одновременном выполнении двух условий:

1. Gateway bootstrap сообщил `available` для exact wire surface;
2. application bundle содержит exact compiled adapter ID.

Список compiled adapters принадлежит `frontend/src/app`. Kernel и provider
runtime не могут зарегистрировать remote frontend code. Несовпадение fail
closed с `client_route_adapter_unavailable`.

### Communications page

Canonical Communications page использует только generated
`CommunicationsQueryService` через общий browser Gateway transport. Она
показывает provider-neutral evidence и canonical search/read states.

Mail, Telegram, WhatsApp и Zulip operational screens не являются режимами
Communications domain component. `app` размещает их рядом в product navigation,
но монтирует exact integration adapter. Переход между ними не передаёт domain
store, provider DTO или cache object.

Provider-neutral action, требующий provider execution, не вызывается domain
page напрямую. Для него требуется отдельный application workflow с typed
command/result и evidence lineage.

### Settings composition

Settings является app-level workbench, а не business domain:

- System Control остаётся platform-owned recovery surface;
- public module settings читаются из sanitized bootstrap projection;
- provider-specific semantics, validation и controls живут только у owning
  integration;
- `app` составляет navigation и монтирует выбранную owner panel;
- secret values, provider sessions, cursors, checkpoints и runtime state не
  становятся settings;
- generic settings panel не интерпретирует provider keys и не вызывает
  provider REST.

Mail accounts/sync/content-egress/resource mappings принадлежат Mail. Telegram
read receipts принадлежат Telegram. Historical sensitive forwarding,
cross-channel policy, AI, maintenance и background-job controls не переносятся
в Communications/Settings без отдельного owner/workflow ADR и public contract.

### Transport и cache boundaries

Business access разрешён только через generated ConnectRPC clients и единый
browser Gateway transport. В scoped production source запрещены handwritten
`/api/v1/communications/*`, `/api/v1/integrations/*`, aliases, proxy, fallback,
dual-read и dual-write.

Canonical Communications и каждая integration имеют отдельные query/cache
roots. Realtime frame маршрутизируется к exact owner adapter; app не создаёт
общий provider DTO union и не патчит domain cache provider-specific payload.

### SRP

SRP оценивается по ответственности и причине изменения:

- route catalog знает wire metadata, но не Vue components;
- compiled adapter registry знает только bundled adapter inventory;
- page composition выбирает owner surface, но не выполняет owner query;
- owner controller выполняет один use case;
- presentation component получает typed view model и emits typed intent;
- generated client unit создаёт один service-specific client;
- mapper переводит один generated response в один owner view model.

Большой компонент разделяется, если имеет несколько владельцев или причин
изменения. Маленький facade, который скрывает cross-owner coupling, запрещён
независимо от числа строк.

## Атомарные gates

### `frontend_surface_admission_v1`

Открывается при наличии:

1. отдельных stable wire IDs и exact capability mapping для Communications,
   Mail, Telegram, WhatsApp и Zulip;
2. generated browser contracts, включая WhatsApp;
3. app-owned compiled adapter registry;
4. fail-closed navigation/bootstrap tests без Communications-to-provider fanout;
5. architecture evidence и backend/frontend type/test gates.

### `communications_frontend_owner_v1`

Открывается при наличии active canonical route, generated owner query,
loading/empty/error/result states, separate cache root, accessibility tests и
отсутствии provider imports/legacy REST.

### `<provider>_frontend_operational_v1`

Открывается отдельно для каждого provider при наличии exact compiled adapter,
generated service clients, capability-aware controls, provider-owned settings
и удаления соответствующего scoped legacy REST/query/realtime surface.

### `settings_frontend_composition_v1`

Открыт. Historical `domains/settings` orchestrator и mixed Communications
settings panel удалены; app-level composition монтирует System Control и exact
integration panels, а ownership/SRP проверяются executable tests.

### `communications_frontend_clean_room_v1`

Финальный gate требует открытия всех принятых slices, отсутствия scoped legacy
runtime source, typecheck/unit/visual/build evidence и проверки, что dormant
historical surfaces не входят в production bundle.

## Последствия

- Communications capability больше не включает provider UI.
- Integration остаётся отдельной единицей admission, сборки и failure.
- Settings отражает владельцев, а не собирает скрытый monolith.
- Неподдержанная historical функция видна как gap, а не как fake control.
- App остаётся единственным cross-owner composition root.

## Отклонённые варианты

### Один Communications surface ID для всех provider routes

Отклонено: capability домена не является разрешением integration UI.

### Общий Communications controller с provider switch

Отклонено: domain начинает выбирать provider behavior и владеть integration
cache/API.

### Generic settings service с provider keys

Отклонено: Kernel/app становятся business facade и интерпретируют чужую
семантику.

### Сохранить REST до завершения UI

Отклонено: fallback создаёт второй authority path и скрывает незакрытый gate.
