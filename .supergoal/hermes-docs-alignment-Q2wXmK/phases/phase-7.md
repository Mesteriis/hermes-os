SUPERGOAL_PHASE_START
Phase: 7 of 10 — Frontend: Persona Alignment
Task: Rename persons to personas in frontend — routes, components, stores, i18n. Sync with backend API.
Mandatory commands: cd frontend && pnpm build, cd frontend && pnpm lint
Acceptance criteria: 9
Evidence required: build output, route config, grep for remaining persons references
Depends on phases: Phase 2, Phase 6

## Why

После rename Persons to Personas в backend Phase 2 необходимо синхронизировать frontend. Пути Vue Router, компоненты, stores и i18n keys должны использовать каноническое Persona naming согласно ADR-0084.

## Work

1. **Vue Router route rename:**
   - `/persons` → redirect to `/personas`
   - `/personas` — новый canonical route
   - Добавить /personas route в `frontend/src/app/router.ts`
   - Route name: `personas` / `persona-detail`
   - Сохранить `/persons` как redirect для compatibility

2. **Component rename:**
   - `PersonsList.vue` → `PersonasList.vue` с re-export
   - `PersonsPage.vue` → `PersonasPage.vue` с re-export
   - `PersonDetail.vue` → `PersonaDetail.vue` с re-export
   - `PersonIdentityReview.vue` → `PersonaIdentityReview.vue`
   - `PersonRelationshipReview.vue` → `PersonaRelationshipReview.vue`
   - Все import paths обновлены

3. **Store rename:**
   - `frontend/src/domains/personas/stores/persons.ts` → `personas.ts`
   - Export compatibility alias: `export { usePersonasStore as usePersonsStore }`
   - Обновить все импорты store в компонентах

4. **API calls update:**
   - Все вызовы `/api/v1/persons/*` → `/api/v1/personas/*`
   - Сохранить fallback на `/api/v1/persons` для compatibility

5. **i18n keys update:**
   - `ru.json`: `persons` → `personas` keys (где применимо)
   - `en.json`: `persons` → `personas` keys
   - Не менять translation values — только keys

6. **Update other domain imports:**
   - Communications domain — импорты personas types/stores
   - Review domain — импорты personas types
   - Organizations domain — импорты personas types
   - Tasks/Projects domain — импорты personas types

7. **Verify:**
   - `grep -r "from '\.\./persons" frontend/src/domains/ — 0 matches кроме compatibility
   - `grep -r "/persons" frontend/src/app/router.ts` — только redirect
   - `cd frontend && pnpm build` — build pass
   - `/personas` route рендерит PersonasPage
   - `/persons` route редиректит на `/personas`

## Acceptance criteria (all must pass)

- [ ] AC1: `/personas` route существует и рендерит PersonasPage
- [ ] AC2: `/persons` route редиректит на `/personas` через redirect
- [ ] AC3: Personas store экспортируется как usePersonasStore с usePersonsStore alias
- [ ] AC4: PersonasList.vue существует и импортируется
- [ ] AC5: API calls используют `/api/v1/personas` endpoints
- [ ] AC6: `cd frontend && pnpm build` passes exit 0
- [ ] AC7: `cd frontend && pnpm lint` passes exit 0
- [ ] AC8: Нет прямых references к `persons` в новом коде кроме compatibility слоя
- [ ] AC9: i18n keys для persons/personas корректно обновлены

## Mandatory commands (run each, surface last ~10 lines + exit code)

- `cd frontend && pnpm build`
- `cd frontend && pnpm lint`

## Evidence required in transcript

- Build output — last 10 lines showing success
- Route config from `frontend/src/app/router.ts` — showing /personas route and /persons redirect
- grep results: `grep -rn "persons" frontend/src/domains/ --include="*.ts" --include="*.vue" | grep -v node_modules | grep -v "\.spec\|\.test"` — only compatibility aliases
- List of renamed component files

## Notes

- Domain directory `frontend/src/domains/personas/` уже использует Personas naming — не переименовывать
- Compatibility слой minimal: только alias export + route redirect
- Не менять backend API — только frontend calls
- Reference Phase 2 для consistency в naming
- Переименовывать файлы через git mv или create new + redirect old
