# Persons — API Reference

Base: `/api/v2/`

## Core Profile

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons` | Список персон (?favorites_only, ?limit) |
| GET | `/persons/{id}` | Обогащённый профиль персоны |
| POST | `/persons/{id}/fingerprint` | Построить communication fingerprint |
| POST | `/persons/{id}/favorite` | Переключить избранное |
| PUT | `/persons/{id}/notes` | Сохранить заметки |
| GET | `/persons/search?q=...` | Поиск по имени/email |

## Identities

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/identities` | Список identities |
| POST | `/persons/{id}/identities` | Добавить identity |
| DELETE | `/persons/{id}/identities/{iid}` | Удалить identity |

## Identity Resolution

| Метод | Путь | Описание |
|---|---|---|
| GET | `/identity-candidates` | Кандидаты на merge/split |
| PUT | `/identity-candidates/{id}/review` | Подтвердить/отклонить |
| GET | `/persons/{id}/identity` | Подтверждённые связи |

## Roles

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/roles` | Список ролей |
| POST | `/persons/{id}/roles` | Назначить роль |
| DELETE | `/persons/{id}/roles/{role}` | Снять роль |

## Personas

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/personas` | Список персон |
| POST | `/persons/{id}/personas` | Создать персону |
| DELETE | `/persons/{id}/personas/{pid}` | Удалить персону |

## Memory

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/facts` | Список фактов |
| POST | `/persons/{id}/facts` | Добавить факт |
| GET | `/persons/{id}/memory-cards` | Карточки памяти |
| POST | `/persons/{id}/memory-cards` | Добавить карточку |
| GET | `/persons/{id}/preferences` | Предпочтения |
| POST | `/persons/{id}/preferences` | Сохранить предпочтение |
| GET | `/persons/{id}/snapshots` | Снимки состояния |
| GET | `/persons/{id}/history-diff?from=&to=` | Сравнить снимки |

## Timeline

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/timeline` | Таймлайн событий (?limit) |
| POST | `/persons/{id}/timeline` | Добавить событие |

## Enrichment

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/enrichment` | Результаты enrichment |
| POST | `/persons/{id}/enrichment/{rid}/apply` | Применить результат |
| POST | `/persons/{id}/enrichment/{rid}/reject` | Отклонить результат |

## Expertise

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/expertise` | Навыки персоны |
| GET | `/persons/search/expertise?skill=` | Поиск по навыку |

## Trust

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/promises` | Список обещаний |
| GET | `/persons/{id}/risks` | Список рисков |

## Health

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/health` | Здоровье отношений |
| GET | `/persons/health` | Все нездоровые персоны |
| GET | `/persons/watchlist` | Watchlist |
| POST | `/persons/{id}/watchlist` | Toggle watchlist |

## Investigator

| Метод | Путь | Описание |
|---|---|---|
| POST | `/persons/{id}/investigate` | Запустить investigator |
| GET | `/persons/{id}/dossier` | Полное досье |
| GET | `/persons/{id}/meeting-prep` | Подготовка к встрече |

## Analytics & Export

| Метод | Путь | Описание |
|---|---|---|
| GET | `/persons/{id}/analytics` | Вся аналитика |
| GET | `/persons/{id}/export?format=md\|json` | Экспорт досье |

## EnrichedPerson (GET /persons/{id})

```json
{
  "person_id": "person:v1:email:11:alice@ex.com",
  "display_name": "Alice",
  "email_address": "alice@example.com",
  "language": "en",
  "tone": "friendly",
  "trust_score": 72,
  "avg_response_hours": 4.5,
  "preferred_channel": "email",
  "last_interaction_at": "2026-06-07T10:00:00Z",
  "interaction_count": 42,
  "frequent_topics": ["finance", "project"],
  "writing_style": "concise",
  "person_metadata": {},
  "is_favorite": true,
  "notes": "CTO at Acme Corp",
  "linked_projects": ["hermes-hub"],
  "linked_documents": ["doc:1"]
}
```
