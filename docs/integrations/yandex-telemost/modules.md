# Yandex Telemost Modules

## Backend

```text
backend/src/integrations/yandex_telemost/
├── mod.rs
├── runtime.rs
└── client/
    ├── mod.rs
    ├── errors.rs
    ├── models.rs
    ├── store.rs
    └── validation.rs
```

Responsibilities:

```text
client/models.rs      DTOs, provider kind constants, capability contract
client/errors.rs      typed provider runtime errors
client/validation.rs  URL/token/payload validation and sanitizer
client/store.rs       provider account setup, HostVault binding, API calls, event publication
runtime.rs            runtime-facing reexports
```

## App routes

```text
backend/src/app/provider_runtime_handlers/yandex_telemost.rs
backend/src/app/handlers/yandex_telemost.rs
```

Routes are provider runtime/setup routes only. They are not Calendar or Calls
business routes.

## Desktop runtime

```text
frontend/src-tauri/src/yandex_telemost_companion.rs
```

Responsibilities:

```text
visible WebView open
allowed-origin navigation guard
WebView active-speaker heuristic bridge
local virtual/loopback audio preparation
ffmpeg MP3 recording start/stop
speaker timeline JSONL/TXT append
```

## Frontend

```text
frontend/src/integrations/yandexTelemost/
├── api/yandexTelemost.ts
├── components/YandexTelemostSettingsPanel.vue
├── queries/yandexTelemostQueryKeys.ts
├── queries/useYandexTelemostRuntimeQuery.ts
└── types/yandexTelemost.ts
```

The settings panel may create conferences and launch the visible companion
WebView. Product-level meeting views must remain provider-neutral.
