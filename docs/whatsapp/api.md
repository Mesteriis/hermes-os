# WhatsApp API Reference

Status: runtime/setup foundation only.

## Ownership

WhatsApp runtime, session, setup, fixture, and provider-control APIs live under:

```text
/api/v1/integrations/whatsapp/*
```

Communications business state stays provider-neutral and lives under:

```text
/api/v1/communications/*
```

## Implemented Foundation Routes

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/integrations/whatsapp/capabilities` | Capability matrix |
| POST | `/api/v1/integrations/whatsapp/accounts/fixture` | Fixture account setup |
| GET | `/api/v1/integrations/whatsapp/sessions` | Session/runtime list |
| GET | `/api/v1/integrations/whatsapp/messages` | Projected WhatsApp integration message list |
| POST | `/api/v1/integrations/whatsapp/messages` | Fixture message ingest |

## Contract Notes

- There are no provider-scoped business routes under `/api/v1/communications/*`.
- Runtime/setup/provider session state belongs to `/api/v1/integrations/whatsapp/*`.
- Any future user-facing Communication reads or writes must use provider-neutral `/api/v1/communications/*` routes.
- Live WhatsApp runtime and outbound provider writes remain gated by explicit runtime and consent decisions.
