# Event Tracing Gap Analysis

Status: living implementation gap list.

## Known Gaps

- legacy rows can have nullable `correlation_id`;
- realtime stored-event replay and in-memory bus payloads have different
  `recorded_at` availability;
- observation events are now root trace events in the canonical store path, but
  older observations remain legacy rows;
- raw provider/source signals are trace-linked in the current raw record paths,
  but older events remain disconnected;
- accepted Mail, Telegram and WhatsApp message signals now emit canonical
  `communication.message.recorded` or `communication.message.updated` events
  with inherited trace context after projection;
- Timeline Engine expects subject shapes and must not be used as Trace Engine;
- provider observation events can use idempotency key as root correlation id,
  but derived provider observations must prefer parent trace context;
- frontend trace UI exists as a shared platform surface; embedding links from
  domain detail pages remains follow-up work.

## Watchpoints

- Do not add Telegram-specific or WhatsApp-specific trace ownership.
- Do not make OpenTelemetry the trace source of truth.
- Do not infer missing trace links with AI.
- Do not store private content in trace-specific structures.
- The canonical builder guarantees a non-empty `correlation_id`, but it cannot
  infer whether a new event is semantically root or derived. New derived event
  paths must set `causation_id` explicitly or use `TraceContext` and add
  regression coverage for their causal chain.
