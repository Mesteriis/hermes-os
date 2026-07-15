# Hermes frontend

Статус: существующая product-surface и migration reference

Vue 3, Vite и Tauri source остаются в `frontend/`, чтобы сохранить реальные
экраны, interaction patterns и единственного потребителя будущего backend API.
Этот код ещё не переключён на clean-room Core Gateway и не является
доказательством работающего end-to-end приложения.

Предыдущая frontend documentation с legacy full-stack commands, API/auth
contract, sidecar packaging и transport client перенесена в
[`references/backend-legacy/frontend/README.md`](../references/backend-legacy/frontend/README.md).

## Текущие правила

- Не добавлять новый business API поверх legacy routes.
- Не считать существующие transports и DTO будущим контрактом.
- Provider screens используются как product behavior evidence.
- Новый client contract определяется ADR-0204 и ADR-0205 и переключается
  вертикальными clean-room slices.
- Host/Tauri bridge не является business API.

Для scoped frontend validation используйте только scripts, реально объявленные
в `package.json`, например:

```sh
pnpm lint
pnpm typecheck
pnpm test:unit
pnpm build
```

Успешная frontend-команда не подтверждает наличие clean-room backend.
