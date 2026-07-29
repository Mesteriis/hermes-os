# ADR-0328: Storage bootstrap quarantine for policy-invalid owner bundles

- Статус: принято
- Дата: 2026-07-29
- Состояние реализации: реализовано и подтверждено live generations 93-95
- Уточняет: ADR-0202, ADR-0203, ADR-0224

## Контекст

Storage Runtime получает Kernel-authorized desired bindings и exact immutable
bundles. PostgreSQL AST admission остаётся последним fail-closed gate перед
credentials, roles, migrations и pool configuration.

Если один ранее сохранённый bundle больше не проходит hard additive policy,
остановка всего Storage Runtime создаёт recovery deadlock:

- invalid owner не получает data-plane rights, что правильно;
- все остальные owners также теряют Storage, что нарушает isolation;
- owner не может заменить invalid binding через normal successor flow, потому
  что live apply требует работающий Storage Control.

Такой deadlock был подтверждён на immutable Attachment Security bundle revision
3: policy отклонила data `UPDATE`, а новый revision 4 не мог быть применён до
readiness старого desired plan.

## Решение

Перед любыми runtime credential, PostgreSQL role, migration или PgBouncer
операциями Storage Runtime повторно выполняет canonical AST admission каждого
desired bundle.

- binding, чей exact bundle проходит policy, остаётся в bootstrap plan;
- binding с отсутствующим или policy-invalid bundle исключается вместе с
  bundle из effective runtime configuration;
- quarantined binding не получает credential resolution, role reconciliation,
  migration execution или PgBouncer entry;
- остальные owners продолжают bootstrap независимо;
- Kernel Control Store остаётся authority desired state и не переписывается
  Storage Runtime;
- owner заменяет quarantined binding обычным revoke/successor/admit/apply flow;
- live apply нового binding по-прежнему atomic и fail-closed.

Quarantine не является автоматическим исправлением SQL, fallback на старый
digest или разрешением non-additive migration. Storage не интерпретирует
business data и не изменяет owner state ради recovery.

## Ownership и единицы сборки

- Storage migration policy владеет AST admission;
- Storage Runtime владеет effective bootstrap filtering;
- Kernel хранит desired bindings/bundles и не парсит owner SQL;
- owner persistence владеет своими additive migrations и data reconciliation;
- assembly только координирует owner-authorized successor plan.

Ни Kernel, ни integration, ни domain не получают доступ к чужим таблицам.

## Admission evidence

1. policy-invalid bundle удаляется из effective bootstrap configuration;
2. binding удаляется до credential/role/migration/pool stages;
3. valid bindings и bundles сохраняются exact;
4. live successor binding применяется после quarantined predecessor;
5. Storage и Gateway остаются available для остальных owners;
6. architecture/unit gates и `make pre-commit` проходят.

Live generation 93 исключил один policy-invalid binding до data-plane stages,
оставил Storage/Gateway доступными и применил valid successor revision 4.
Generation 95 затем штатно применил additive Attachment Security successor
revision 5 без reset или изменения quarantined bytes.

## Последствия

Ошибка одного owner bundle остаётся fail-closed для этого owner, но больше не
становится global Storage outage. Desired state остаётся наблюдаемым в Kernel,
а восстановление проходит через обычный immutable successor, без reset Control
Store, manual SQL и owner-specific platform исключений.
