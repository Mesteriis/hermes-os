# ADR-0396: Recording-to-transcription client authority handoff

Статус: Принято

Дата: 2026-08-04

Состояние реализации: typed recording `Get` authority и managed/browser
conformance реализованы в слайсе `call_transcription_managed_conformance_v1`;
gate `call_transcription_v1` остаётся закрытым до полного `make pre-push`.

Уточняет:

- [ADR-0390: Call recording custody and Speech-to-Text boundary](ADR-0390-call-recording-custody-and-speech-to-text-boundary.md);
- [ADR-0394: Desktop call recording host capture and consent authority](ADR-0394-desktop-call-recording-host-capture-and-consent-authority.md).

## Контекст

`StartCallTranscriptionRequestV1` требует exact operation, call evidence,
recording evidence и consent receipt revisions. Recording integration владеет
этими значениями, но прежний public `Get` возвращал только recording ID,
lifecycle, revision, duration и error. Поэтому app не мог начать transcription
после подтверждённой записи без чтения integration storage, разбора внутреннего
durable event или создания недоказанной authority.

Ни тестовый SQL, ни frontend-generated receipt не допустимы: consent receipt
создаёт recording owner после подтверждённого native capture start.

## Решение

Recording public contract получает optional
`RecordingTranscriptionAuthorityV1` только в terminal `Ready` ответе `Get`.
Он содержит:

- stable recording operation ID;
- canonical call evidence ID и exact revision;
- recording evidence ID и exact terminal revision;
- opaque consent receipt ID и policy revision.

Authority отсутствует во всех non-ready/rejected ответах и в realtime SSE.
После pre-opened recording SSE app выполняет один `Get` на terminal Ready,
маппит exact authority в generated `StartCallTranscriptionRequestV1` и больше
не polling. App не генерирует и не изменяет IDs/revisions.

Receipt ID является opaque authority identifier, а не consent body или secret.
Contract не раскрывает challenge, device identity/hash, capture timestamps,
Blob reference/proof, audio metadata beyond existing bounded duration, path или
provider identity.

Recording integration не импортирует transcription implementation или
contract: handoff type принадлежит recording API и описывает только его
собственную доказанную authority. App является composition owner и переводит
typed recording output в typed workflow input.

## SRP и границы

- recording integration создаёт и выдаёт только собственную source authority;
- app сопоставляет два public contracts;
- transcription workflow повторно проверяет exact event evidence и revisions;
- Communications не импортирует recording или transcription implementation;
- Kernel/Gateway маршрутизируют opaque generated messages без business merge.

## Проверка

1. non-ready `Get` не содержит authority;
2. Ready `Get` содержит exact persisted IDs/revisions и opaque receipt;
3. SSE, logs, errors и durable client payload не содержат consent body/device;
4. app/conformance не читает recording storage и не синтезирует receipt;
5. stale receipt/revision и wrong actor/session fail closed;
6. architecture, managed/browser и full `make pre-push` gates проходят до
   открытия `call_transcription_v1`.
