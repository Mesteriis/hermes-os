# ADR-0288: Managed successor quiesce and Storage fence order

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Реализовано в Kernel managed-runtime supervisor,
Storage successor lifecycle и bootstrap control dispatch. Unit evidence
доказывает idempotent pre-fence quiesce, а live managed Telegram restart —
physical fence predecessor, generation `N + 1` и доступную owner projection
после successor launch. Production owner inventory не меняется.

## Контекст

Managed successor обязан одновременно выполнить два инварианта:

- старый process не может использовать прежний Storage/Vault fence после
  выдачи successor identity;
- ожидаемое физическое отключение старого PgBouncer alias не должно выглядеть
  для supervisor как обычный crash и запускать autorestart со stale binding.

Текущий порядок выполнял physical Storage fence до запрета autorestart exact
worker. Между этими шагами старый runtime мог получить ожидаемую database
ошибку, завершиться и быть автоматически запущен повторно с прежним
runtime-generation configuration. Новый generation при этом ещё не получал
права, но predecessor lifecycle застревал в bounded restart exhaustion.

Это общий managed lifecycle defect. Он не принадлежит Telegram, Mail,
WhatsApp, Zulip или Communications и не может исправляться provider-specific
retry либо ослаблением Storage fence.

## Решение

Managed Storage successor использует один fail-closed порядок:

1. Kernel переводит exact predecessor binding из `active` в durable
   `revoking`.
2. Supervisor атомарно ставит exact worker `stop_requested`. С этого момента
   worker не имеет права autorestart-ить child.
3. Storage Control завершает exact Vault/PgBouncer/PostgreSQL physical fence
   для reserved binding.
4. Kernel join-ит или reap-ит exact predecessor worker.
5. Только после успешных шагов 1-4 Kernel резервирует следующий managed
   runtime generation и выдаёт его новый Storage binding.

`stop_requested` не является authority fence и не заменяет durable
`revoking`, Vault revoke, PgBouncer alias disable или PostgreSQL role/session
fence. Это supervisor-owned quiesce marker, запрещающий только автоматический
перезапуск predecessor во время физического fencing.

Если quiesce, physical fence или join не завершаются, successor reservation не
создаётся. Binding остаётся `revoking`, и повтор использует существующую exact
revocation reservation.

## Границы ответственности

- Kernel supervisor владеет worker quiesce, autorestart policy и join.
- Storage Control владеет physical Vault/PgBouncer/PostgreSQL fence.
- Control Store владеет durable binding state и generation high-watermark.
- Owner runtime не распознаёт provider-specific способ рестарта и не
  продолжает data plane после stale fence.
- Integration/domain/workflow packages не импортируются в эту platform unit.

## Отклонённые варианты

### Provider-specific retry database errors

Отклонено: retry скрывает Kernel race, оставляет stale worker активным и
размножает разное lifecycle-поведение между integrations.

### Остановить process и только потом записать `revoking`

Отклонено: process-control action не является durable authority fence, а crash
между stop и записью допускает неоднозначный recovery.

### Сначала physical fence, затем обычный `stop`

Отклонено: это прежний порядок, допускающий autorestart stale child в окне
между шагами.

## Проверка

Обязательное evidence:

- supervisor test на idempotent pre-fence `stop_requested`;
- successor test, где predecessor получает physical database fence, но не
  запускается повторно;
- live managed integration restart с generation `N + 1`, новым Storage alias
  и доступной owner query после predecessor fencing;
- stale generation и прежний alias остаются недоступны;
- Kernel/Storage architecture и SRP gates остаются зелёными.

## Последствия

- successor rotation остаётся fail closed и не ослабляет physical fence;
- expected predecessor fencing больше не загрязняет новый generation bounded
  restart budget;
- один platform contract применяется ко всем managed owner runtimes;
- наличие ADR не открывает новый domain/integration capability и не является
  доказательством live conformance.
