# ADR-0256: Owner-declared client RPC route admission

Статус: Принято
Дата: 2026-07-23
Состояние реализации: Не реализовано. ADR заменяет временную
Communications-specific route composition в Kernel Gateway; до перехода
внешняя owner query delivery не считается clean-room evidence.

Уточняет:

- [ADR-0205: Core Gateway](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0215: module admission](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221: ModuleDescriptorV1](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0251: client_gateway_v1](ADR-0251-client-gateway-v1-opening-for-owner-contracts.md);
- [ADR-0252: first_owner_v1 Communications admission](ADR-0252-first-owner-v1-communications-admission.md).

## Контекст

`ModuleDescriptorV1` уже описывает ClientRpc contract, однако не содержит
exact public Connect path. Временная реализация воспроизводит этот path,
owner, capability и schema digest внутри Kernel Gateway для Communications.
Такой hardcode делает core transport owner-specific facade и не допускает
второй owner без изменения Kernel.

## Решение

Каждая capability с `ProvidedSurfaceKindV1::ClientRpc` обязана объявить
`ClientRpcRouteV1` в signed `ModuleDescriptorV1`. Route содержит только:

- exact absolute Connect service-method path;
- `POST` как единственный method;
- exact `ContractReferenceV1` с owner/name/major/revision/schema digest;
- capability, в чьём provided surface находится route.

Path не является свободной строкой Gateway configuration. Descriptor
validation принимает только canonical Connect path, один route на capability
и exact correspondence с ClientRpc contract. Никакой route не содержит
provider identity, secret, business content, query text или arbitrary handler
name.

При registration Kernel extracts candidate routes, но route не монтируется.
Только approved effective GrantSet создаёт immutable client-route record в
private Control Store. Record содержит registration, owner/module IDs,
capability, contract, exact path, descriptor digest и grant epoch. Он
invalidates on suspension, revoke, descriptor replacement, grant epoch change
or managed runtime generation replacement.

Gateway получает bounded snapshot approved records и формирует generic
`ClientRpcRouter`. Для каждого request он:

1. проверяет session/device authority и Connect transport limits;
2. находит exact registered path, contract and currently approved capability;
3. creates opaque `ModuleClientRequestV1` from record metadata;
4. routes it through existing managed capability router with generation/grant
   fences;
5. returns opaque response bytes with generic sanitized Connect errors.

Gateway, Kernel and gateway packages neither import owner contract crates nor
decode owner request/response Protobuf. The owner runtime alone decodes its
payload. A missing, stale or ambiguous route is `not_found` or `unavailable`,
never a fallback to a handwritten endpoint.

## Communications transition

Communications declares its existing public Query service path and
`communications.query.v1` contract in its descriptor. The route is admitted
from that descriptor like any future owner route. The following temporary
Kernel symbols must be removed in the same implementation slice:

- Communications-specific owner/capability/contract constants;
- Communications schema dependency in Gateway composition;
- `find_communications_query_registration` and
  `encode_communications_query_module_request` helpers;
- `CommunicationsQueryRouter` and its fixed route branch in Gateway runtime.

The generated Communications client keeps its existing public service path;
this ADR changes admission/composition only, not its owner contract.

## Acceptance evidence

- descriptor validation rejects malformed, duplicate, non-ClientRpc and
  contract-mismatched routes;
- pending or revoked registration cannot mount or receive a route;
- an approved Communications descriptor mounts its generated route through
  generic Gateway transport and reaches only its managed runtime;
- wrong owner, capability, schema digest, grant epoch, runtime generation,
  method, path, content type, payload limit and deadline fail closed;
- Gateway/Kernal Cargo graphs and source guards contain no owner-specific
  client route, schema or generated owner contract dependency;
- a second synthetic owner can be admitted without changing Gateway code;
- no route metadata leaks private payloads or provider state.

## Rollback

Disabling the capability removes its route record and unmounts it on the next
Gateway configuration refresh. The owner runtime, data and durable records are
retained. Reverting this ADR must not restore a hardcoded owner route.
