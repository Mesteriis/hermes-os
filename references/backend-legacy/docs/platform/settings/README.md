# Application Settings

Status: code-aligned documentation package created from ADR-0054 and current
backend/frontend modules.

Application settings are allowlisted, typed, non-secret runtime and UI values.
They are a platform contract and Settings UI surface, not a product domain that
owns provider accounts or credentials.

ADR source of truth:

- [ADR-0054 Application Settings Store](../../archive/adr/ADR-0054-application-settings-store.md)
- [ADR-0081 Opt-In OmniRoute AI Runtime](../../archive/adr/ADR-0081-opt-in-omniroute-ai-runtime.md)
- [ADR-0082 AI Settings Control Center](../../archive/adr/ADR-0082-ai-settings-control-center.md)

## Current Implementation Evidence

Current backend files:

- `backend/src/platform/settings.rs`;
- `backend/src/platform/settings/store.rs`;
- `backend/src/platform/settings/models.rs`;
- `backend/src/platform/settings/definitions.rs`;
- `backend/src/app/router/routes/settings.rs`;
- `backend/src/app/handlers/settings/mod.rs`.

Current frontend package:

- `frontend/src/domains/settings`.

Current API routes include:

- `/api/v1/settings`;
- `/api/v1/settings/accounts`;
- `/api/v1/settings/{setting_key}`.

`ApplicationSettingsStore` lists declared settings, updates editable declared
values, derives AI runtime settings and repairs the declared settings table at
startup. Setting value kinds are boolean, integer, string and JSON.

## Boundary Rule

Settings store declared non-secret values only. Provider accounts remain
provider/account records, and credential material remains behind the vault or
secret boundary. Secret-like setting keys are rejected.

