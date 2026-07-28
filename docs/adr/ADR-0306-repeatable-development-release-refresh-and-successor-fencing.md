# ADR-0306: Повторяемое обновление development release и successor fencing

Статус: Принято
Дата: 2026-07-27
Состояние реализации: реализовано для
`loopback_full_stack_dev_assembly_v1`. Gate закрыт двумя последовательными
live `make dev` на одном default data directory с signed generations `7` и
`8`, неизменным набором registration IDs и без удаления owner/provider state.
Crash recovery также завершает durable reservation предыдущей generation,
атомарно сохраняет её state и только затем начинает requested successor; это
подтверждено восстановлением незавершённой generation `18` перед successful
generation `20` без удаления Control Store, PostgreSQL или provider state.

Уточняет:

- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0288: Managed successor quiesce и Storage fence order](ADR-0288-managed-successor-quiesce-and-storage-fence-order.md);
- [ADR-0300: Loopback full-stack development assembly](ADR-0300-loopback-full-stack-development-assembly.md);
- [ADR-0301: Bundled module discovery и development admission](ADR-0301-bundled-module-discovery-and-development-admission.md).

## Контекст

`make dev` materialize-ит новый locally signed distribution перед каждым
запуском, но прежняя реализация всегда записывала
`distribution_generation=1`. Development assembly сохраняла только generation
и registrations первого admission. После изменения executable bytes следующий
запуск:

1. устанавливал новую signed release с прежней generation;
2. видел существующий assembly state как `admitted`;
3. не выполнял explicit rebind;
4. пытался запустить новый artifact через старый durable binding;
5. корректно получал fail-closed ошибку
   `installed managed launch artifact does not match its durable binding`.

Удаление `.local/kernel-dev`, создание нового default data directory или
повторная регистрация modules скрыли бы дефект ценой потери owner settings,
provider accounts, sessions и технической истории. Автоматическое ослабление
integrity проверки также запрещено.

## Решение

Вводится development-only contract:

```text
repeatable_development_release_refresh_v1
```

Явный вызов `make dev` является owner-authorized host operation для exact
locally signed development distribution. Он не даёт Kernel права самостоятельно
обновлять, скачивать, выбирать rollback либо менять module inventory.

### Monotonic signed generation

Materializer:

- читает generation только из private metadata текущей project-local
  development release;
- для pristine release использует generation `1`;
- для следующей целой release использует checked `N + 1`;
- передаёт generation в signed `DistributionManifestV1`;
- кладёт ту же generation в `0600` host metadata внутри atomically staged
  release root;
- atomically заменяет release вместе с metadata.

Metadata не является launch authority. Kernel повторно проверяет signature,
manifest generation и exact artifact bytes. Missing metadata у прежней
реализации однозначно мигрируется как installed generation `1`; malformed,
symlink либо non-private metadata fail closed.

### Reconcile вместо повторной регистрации

Development assembly различает:

```text
missing | current | stale
```

- `missing` выполняет первый proposal/approval/bind/admission;
- `current` ничего не меняет;
- `stale` допускается только для того же distribution ID и строго большей
  generation.

При `stale` assembly переиспользует exact существующие registration IDs и
owner/provider settings. Новые registrations и settings reset запрещены.
Если signed `ModuleDescriptorV1` изменился, fresh owner operation атомарно
заменяет descriptor requests той же registration, увеличивает grant epoch и
одобряет exact capability set текущего bundled artifact. Module/owner identity
меняться не могут; descriptor берётся только из verified installed
distribution. Это explicit reapproval в рамках `make dev`, а не implicit
Kernel upgrade.

Для каждого plan item update выполняется через generic owner-control contracts:

1. fresh owner-private query читает exact current managed Storage binding
   revision, role epoch и credential lease revision из Kernel Control Store;
2. durable reserve текущего Storage binding для revocation;
3. quiesce и physical fence predecessor по ADR-0288;
4. reconcile exact signed descriptor на той же registration и increment grant
   epoch только при descriptor drift;
5. bind exact artifact из текущей installed signed release;
6. admit exact current Storage bundle;
7. reserve новый managed runtime generation;
8. issue successor Storage binding с monotonically increased role epoch и
   credential lease revision;
9. atomically commit updated assembly state;
10. start successor только из committed current state.

Ни development assembly, ни Kernel не импортируют Communications,
Attachment Security, Mail, Telegram, WhatsApp или Zulip implementation.
Assembly координирует только generic typed owner-control contracts и exact
development plan.

### Crash recovery

До выдачи successor Storage binding assembly сохраняет owner-private durable
reservation с:

- target distribution ID/generation;
- exact registration/capability IDs;
- reserved runtime instance/generation;
- Storage bundle revision/digest;
- target role epoch и credential lease revision.

Повтор:

- принимает exact already-`revoking` binding;
- повторяет exact successor binding idempotently;
- не начинает другую generation, пока reservation не завершена;
- если `make dev` уже materialize-ил следующую signed generation, сначала
  завершает predecessor reservation, атомарно сохраняет её assembly state и
  удаляет только завершённую reservation, затем начинает requested successor;
- пишет assembly state atomic rename и только после этого удаляет reservation.

Legacy state format version `2` мигрируется без reset: его единственная
реализованная initial issuance имела binding revision, role epoch и credential
lease revision `1`. Эти значения являются только migration diagnostics:
provider Settings могли уже выполнить отдельную successor rotation, поэтому
refresh всегда использует fresh Kernel query. Новая запись использует version
`3` и сохраняет last-reconciled fences явно, но также не превращает их в
authority.

После restart Storage runtime может не иметь predecessor binding в своём
in-memory active set. Durable binding со state `revoking` всё равно должен
закончить physical fencing через exact reserved binding. Подтверждённое
отсутствие PgBouncer pool либо PostgreSQL role означает, что соответствующий
predecessor уже физически fenced; ошибка catalog/admin query остаётся
fail-closed и не приравнивается к отсутствию.

Если signed bundled release продвигает Settings schema, Kernel строит
provider-neutral successor snapshot:

- сохраняет только значения с тем же setting ID и тем же value type;
- применяет declarative defaults новой schema, если они есть;
- не выполняет provider-specific alias или semantic conversion;
- переводит неполную обязательную configuration в `blocked_config` с
  sanitized reason `required_settings_missing`;
- атомарно меняет schema binding, schema artifact и desired snapshot.

Поэтому Zulip `bot_email` не переносится в новый `account_email`: это разные
provider semantics по ADR-0304, и владелец должен явно заполнить новое поле.

Rollback на меньшую либо ту же неподтверждённую generation, смена distribution
ID и partially matching plan fail closed. Автоматического удаления state,
Control Store, Vault, PostgreSQL, Blob либо provider session data нет.

## SRP и единицы сборки

- release materializer владеет только построением и atomic installation
  development distribution;
- development assembly владеет exact plan reconciliation;
- Kernel владеет verification, durable authority и process lifecycle;
- Storage Control владеет physical database fencing;
- Vault владеет credential lease invalidation;
- каждый domain, engine и integration остаётся отдельной build/runtime unit;
- provider account и QR authorization остаются integration-owned и не
  становятся частью release update.

## Проверка

Gate закрывается только при наличии:

1. builder tests на explicit positive generation и rejection invalid input;
2. assembly tests на `missing/current/stale`, legacy state migration,
   monotonically increased fences и rollback rejection;
3. owner-control coverage для fresh managed Storage status и revocation;
4. architecture contract, запрещающего reset/new registration в refresh path;
5. live первого и повторного default `make dev` на одном
   `<repository>/.local/kernel-dev`;
6. evidence, что повторный запуск использует те же registrations, новые
   release/runtime generations и сохраняет Settings/provider state;
7. browser evidence, что Communications Settings и provider-specific QR
   authorization по-прежнему доступны только через реальные integration
   capabilities.

Реализационное evidence для закрытия gate:

- unit/integration coverage подтверждает descriptor upgrade на той же
  registration, atomic Settings schema successor, restart-safe Storage
  revocation и уже отсутствующие PgBouncer/PostgreSQL predecessor resources;
- первый live запуск записал state version `3`, distribution generation `7` и
  шесть module registrations;
- второй default live запуск записал generation `8`, сохранил exact SHA-256
  отсортированного набора registration IDs
  `ecceff02f88b5cbb77063ea6858706bb5f658d3cebad07db94c9ec89feff4ed8`
  и не оставил durable reservation;
- `http://127.0.0.1:5173/` и authenticated Gateway readiness через Vite
  development proxy ответили `200`;
- live Telegram Settings показывают только `Telegram QR login`, объясняют
  short-lived TDLib login link, не содержат bot UI и держат QR action
  недоступным до сохранения реального provider account.

## Последствия

- `make dev` становится повторяемым после изменения backend/frontend bytes.
- Integrity mismatch исправляется explicit successor lifecycle, а не bypass.
- Локальные аккаунты и sessions сохраняются между development releases.
- Каждый запуск materialize-ит новую signed generation и поэтому выполняет
  bounded owner-authorized rebind существующего exact plan.
- Наличие ADR без tests и двух последовательных live запусков не закрывает
  gate.
