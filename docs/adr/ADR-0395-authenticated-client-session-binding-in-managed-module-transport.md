# ADR-0395: Authenticated client session binding in managed module transport

Статус: Принято

Дата: 2026-08-04

Состояние реализации: transport field, atomic validation, Gateway propagation
и actor/session-bound ticket redemption реализованы в слайсе
`call_transcription_runtime_v1`; inventory gate `call_transcription_v1`
остаётся `planned` до release assembly и browser conformance.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0232: Browser client device identity and same-origin session](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0390: Call recording custody and Speech-to-Text boundary](ADR-0390-call-recording-custody-and-speech-to-text-boundary.md).

## Контекст

Core Gateway уже аутентифицирует exact browser session, logical owner и device,
но `ModuleClientRequestV1` передаёт managed owner только owner/device. Поэтому
one-use Blob ticket невозможно честно связать с session: подстановка device ID
разрешила бы использовать ticket из другой активной session того же device.

## Решение

`ModuleClientRequestV1` получает additive protobuf field
`authenticated_client_session_id`. Значение берётся только из успешно
авторизованной `BrowserSession`; client payload не может его задать или
переопределить.

Authenticated owner, device and session являются атомарным transport context:
либо все три заданы и валидны, либо все три пусты для внутренних callers,
которые не могут получить client Blob authority. Core Gateway передаёт session
ID одинаково для ClientRpc и ClientBlob. Managed owner хэширует его с
owner-specific domain separation и хранит только digest в one-use ticket.

Session завершение, замена cookie или запрос из другой session делает ticket
недействительным независимо от совпадения owner/device. Module transport не
выдаёт session ID в business response, SSE, logs или durable event.

## SRP и границы

- Gateway session service владеет browser session identity.
- Gateway transport добавляет authenticated context в internal envelope.
- Managed owner решает, какие операции требуют session-bound authority.
- Kernel router не интерпретирует business payload и не создаёт session ID.
- Frontend не передаёт session ID как поле business request.

## Проверка

1. protocol validation принимает только atomic owner/device/session context;
2. Gateway ClientRpc и ClientBlob используют `BrowserSession::session_id()`;
3. managed owner ticket хранит digest, не raw session ID;
4. другая session того же owner/device не может redeem ticket;
5. существующие internal callers остаются совместимы только с тремя пустыми
   authenticated fields;
6. full architecture, Rust and managed/browser gates проходят до открытия
   `call_transcription_v1`.
