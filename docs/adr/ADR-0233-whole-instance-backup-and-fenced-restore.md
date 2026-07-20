# ADR-0233: Scoped local recovery export и отдельный PostgreSQL dump

Статус: Принято  
Дата: 2026-07-19  
Состояние реализации: Частично реализовано. Существуют отдельные offline
Control Store export/restore и encrypted authenticated Vault backup/restore.
Добавлена отдельная процедура PostgreSQL data export. Единого
whole-instance backup или restore coordinator нет.

Зависит от:

- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).

## Контекст

Текущий local recovery scope намеренно меньше whole-instance snapshot. Уже
существующие операции сохраняют private Kernel Control Store (включая
settings/control authority) и encrypted Vault storage. PostgreSQL business data
в них не входит и требует независимой выгрузки.

Blob data, JetStream retained records/configuration, Scheduler runtime state,
provider OS profiles/sessions, browser/device credentials, distribution bytes,
логи и caches не являются частью этой процедуры. Их нельзя молча назвать
сохранёнными.

## Решение

### Три независимых owner-scoped артефакта

| Артефакт | Владелец и назначение | Входит |
| --- | --- | --- |
| Control Store export | Kernel offline recovery | Control Store authority и settings; не PostgreSQL business rows |
| Vault backup | Vault offline recovery | encrypted Vault database, anchor и authenticated manifest |
| PostgreSQL custom dump | Storage-admin export | schema и data одной явно указанной PostgreSQL database, включая её migration ledger |

Эти артефакты не образуют атомарный глобальный snapshot. Оператор хранит их
вместе только как recovery material с отмеченным временем создания; порядок и
согласованность между ними не доказываются этой процедурой.

### PostgreSQL export

make -C backend export-postgres-backup вызывает repository-owned wrapper над
явно указанным pg_dump. Он принимает только:

- абсолютный путь к regular non-symlink pg_dump, который executable и не
  writable group/other;
- абсолютный private regular file 0600 с одной PostgreSQL URL;
- отсутствующий абсолютный output path.

Wrapper не передаёт URL или password в argv. Он создаёт private temporary
pg_service.conf и pgpass, запускает pg_dump с format=custom, no-owner и
no-privileges и публикует новый dump atomically without overwrite после digest
и fsync.

Итоговый dump имеет mode 0600; stdout содержит только path, byte size и
SHA-256. no-owner и no-privileges означают, что PostgreSQL global roles, grants
и platform credentials не экспортируются. Это data/schema export, а не portable
cluster image и не автоматическая процедура restore.

Пример запуска (значения путей задаёт operator; URL file не передаётся как
command-line value):

    make -C backend export-postgres-backup \
      PG_DUMP=/absolute/path/to/pg_dump \
      POSTGRES_CONNECTION_URL_FILE=/absolute/private/postgres-url \
      POSTGRES_BACKUP_OUTPUT=/absolute/private/hermes-postgres.dump

### Restore boundary

Проверенная автоматическая PostgreSQL restore procedure пока отсутствует.
Восстановление такого custom dump требует отдельного owner-approved Storage
Control procedure: stopped isolated cluster, explicit schema/ledger
verification, then fresh storage/runtime credentials. Нельзя выполнять restore
в работающий Kernel или считать dump доказательством fenced recovery.

## Последствия

Backup UI/CLI и документация должны говорить точно: доступны отдельные Control
Store/Vault recovery operations и PostgreSQL data export. Они не обещают
backup/restore Blob, JetStream, Scheduler, provider state или whole Hermes
instance.
